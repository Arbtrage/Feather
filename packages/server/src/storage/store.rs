use super::keys::KeyBuilder;
use super::traits::{ActivityQueueStore, QueueStats, StorageError, StorageResult};
use crate::domain::{Job, JobState};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RedisStore {
    pool: Pool,
    keys: KeyBuilder,
    recent_history_limit: usize,
}

impl RedisStore {
    pub fn new(
        redis_url: &str,
        namespace: &str,
        recent_history_limit: usize,
    ) -> StorageResult<Self> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(Self {
            pool,
            keys: KeyBuilder::new(namespace),
            recent_history_limit,
        })
    }

    fn job_to_hash(job: &Job) -> Vec<(String, String)> {
        vec![
            ("id".into(), job.id.clone()),
            ("queue".into(), job.queue.clone()),
            ("name".into(), job.name.clone()),
            ("payload".into(), base64_encode(&job.payload)),
            ("state".into(), job.state.as_str().into()),
            ("priority".into(), job.priority.to_string()),
            ("attempt".into(), job.attempt.to_string()),
            (
                "worker_id".into(),
                job.worker_id.clone().unwrap_or_default(),
            ),
            (
                "lease_expires_at".into(),
                job.lease_expires_at
                    .map(|t| t.timestamp_millis().to_string())
                    .unwrap_or_default(),
            ),
            (
                "created_at".into(),
                job.created_at.timestamp_millis().to_string(),
            ),
            (
                "updated_at".into(),
                job.updated_at.timestamp_millis().to_string(),
            ),
            (
                "failure_reason".into(),
                job.failure_reason.clone().unwrap_or_default(),
            ),
            ("workflow_run_id".into(), job.workflow_run_id.clone()),
            ("activity_id".into(), job.activity_id.clone()),
            ("lease_renewals".into(), job.lease_renewals.to_string()),
        ]
    }

    fn hash_to_job(map: HashMap<String, String>) -> Option<Job> {
        let id = map.get("id")?.clone();
        Some(Job {
            id,
            queue: map.get("queue")?.clone(),
            name: map.get("name")?.clone(),
            payload: base64_decode(map.get("payload").map(String::as_str).unwrap_or("")),
            state: map.get("state")?.parse().ok()?,
            priority: map.get("priority")?.parse().unwrap_or(0),
            attempt: map.get("attempt")?.parse().unwrap_or(1),
            worker_id: map.get("worker_id").filter(|s| !s.is_empty()).cloned(),
            lease_expires_at: map
                .get("lease_expires_at")
                .filter(|s| !s.is_empty())
                .and_then(|s| s.parse::<i64>().ok())
                .and_then(|ms| Utc.timestamp_millis_opt(ms).single()),
            created_at: map
                .get("created_at")
                .and_then(|s| s.parse::<i64>().ok())
                .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
                .unwrap_or_else(Utc::now),
            updated_at: map
                .get("updated_at")
                .and_then(|s| s.parse::<i64>().ok())
                .and_then(|ms| Utc.timestamp_millis_opt(ms).single())
                .unwrap_or_else(Utc::now),
            failure_reason: map.get("failure_reason").filter(|s| !s.is_empty()).cloned(),
            workflow_run_id: map.get("workflow_run_id").cloned().unwrap_or_default(),
            activity_id: map.get("activity_id").cloned().unwrap_or_default(),
            lease_renewals: map
                .get("lease_renewals")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
    }

    async fn append_event(&self, job_id: &str, event: &str) -> StorageResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let key = self.keys.job_events(job_id);
        let ts = Utc::now().timestamp_millis();
        let _: () = conn
            .lpush(&key, format!("{ts}:{event}"))
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(())
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Vec<u8> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(s).unwrap_or_default()
}

#[async_trait]
impl ActivityQueueStore for RedisStore {
    async fn enqueue(&self, job: Job) -> StorageResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(&job.id);
        let pending = self.keys.queue_pending(&job.queue);
        let recent = self.keys.recent_jobs();
        let registry = self.keys.queue_registry();

        let fields = Self::job_to_hash(&job);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset_multiple(&job_key, &fields);
        pipe.lpush(&pending, &job.id);
        pipe.sadd(&registry, &job.queue);
        pipe.zadd(
            &recent,
            job.id.as_str(),
            job.created_at.timestamp_millis() as f64,
        );
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        self.trim_recent().await?;
        self.append_event(&job.id, "created").await?;
        Ok(())
    }

    async fn dequeue(
        &self,
        worker_id: &str,
        queues: &[String],
        lease_duration_ms: u64,
        wait_timeout_ms: u64,
        max_jobs: u32,
    ) -> StorageResult<Vec<Job>> {
        let max_jobs = max_jobs.clamp(1, 64) as usize;
        let deadline = Instant::now() + Duration::from_millis(wait_timeout_ms);
        let mut jobs = Vec::new();

        while jobs.len() < max_jobs {
            if let Some(job) = self
                .try_claim_nonblocking(worker_id, queues, lease_duration_ms)
                .await?
            {
                jobs.push(job);
                continue;
            }

            if wait_timeout_ms == 0 || Instant::now() >= deadline {
                break;
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            let block_secs = remaining.as_secs().clamp(1, 30) as f64;
            let pending_keys: Vec<String> =
                queues.iter().map(|q| self.keys.queue_pending(q)).collect();

            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;
            let brpop_result: Option<(String, String)> = conn
                .brpop(&pending_keys, block_secs)
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;

            let Some((pending_key, job_id)) = brpop_result else {
                break;
            };

            let Some(queue) = queue_from_pending_key(&pending_key) else {
                continue;
            };

            if let Some(job) = self
                .claim_job_id(worker_id, &queue, &job_id, lease_duration_ms)
                .await?
            {
                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    async fn ack(&self, job_id: &str, worker_id: &str) -> StorageResult<()> {
        let job = self.get_job(job_id).await?;
        if job.state != JobState::Leased {
            if job.state == JobState::Completed {
                return Ok(());
            }
            return Err(StorageError::PreconditionFailed("job not leased".into()));
        }
        if job.worker_id.as_deref() != Some(worker_id) {
            return Err(StorageError::PreconditionFailed("wrong worker".into()));
        }

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let leased_key = self.keys.queue_leased(&job.queue);

        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&job_key, "state", "completed");
        pipe.hset(
            &job_key,
            "updated_at",
            Utc::now().timestamp_millis().to_string(),
        );
        pipe.zrem(&leased_key, job_id);
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        self.append_event(job_id, "completed").await?;
        Ok(())
    }

    async fn nack(&self, job_id: &str, worker_id: &str, reason: &str) -> StorageResult<()> {
        let job = self.get_job(job_id).await?;
        if job.worker_id.as_deref() != Some(worker_id) {
            return Err(StorageError::PreconditionFailed("wrong worker".into()));
        }

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let leased_key = self.keys.queue_leased(&job.queue);

        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&job_key, "state", "failed");
        pipe.hset(&job_key, "failure_reason", reason);
        pipe.hset(
            &job_key,
            "updated_at",
            Utc::now().timestamp_millis().to_string(),
        );
        pipe.zrem(&leased_key, job_id);
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        self.append_event(job_id, "failed").await?;
        Ok(())
    }

    async fn extend_lease(
        &self,
        job_id: &str,
        worker_id: &str,
        extension_ms: u64,
        max_renewals: u32,
    ) -> StorageResult<DateTime<Utc>> {
        let mut job = self.get_job(job_id).await?;
        if job.worker_id.as_deref() != Some(worker_id) {
            return Err(StorageError::PreconditionFailed("wrong worker".into()));
        }
        if job.lease_renewals >= max_renewals {
            return Err(StorageError::PreconditionFailed("max renewals".into()));
        }

        let new_expiry = Utc::now() + chrono::Duration::milliseconds(extension_ms as i64);
        job.lease_expires_at = Some(new_expiry);
        job.lease_renewals += 1;

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let leased_key = self.keys.queue_leased(&job.queue);
        let lease_ms = new_expiry.timestamp_millis();

        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&job_key, "lease_expires_at", lease_ms.to_string());
        pipe.hset(&job_key, "lease_renewals", job.lease_renewals.to_string());
        pipe.zadd(&leased_key, job_id, lease_ms as f64);
        pipe.query_async::<()>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        Ok(new_expiry)
    }

    async fn get_job(&self, job_id: &str) -> StorageResult<Job> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let map: HashMap<String, String> = conn
            .hgetall(&job_key)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        if map.is_empty() {
            return Err(StorageError::NotFound);
        }
        Self::hash_to_job(map).ok_or(StorageError::Other("corrupt job".into()))
    }

    async fn release_expired_leases(&self) -> StorageResult<u64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let now_ms = Utc::now().timestamp_millis();
        let updated_at = now_ms.to_string();
        let mut released = 0u64;

        let queues: Vec<String> = conn
            .smembers(self.keys.queue_registry())
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        let requeue_script = r#"
            local job_id = ARGV[1]
            local job_key = ARGV[2]
            local now_ms = tonumber(ARGV[3])
            local state = redis.call('HGET', job_key, 'state')
            if state ~= 'leased' then return 0 end
            local exp = redis.call('HGET', job_key, 'lease_expires_at')
            if (not exp) or exp == '' or tonumber(exp) > now_ms then return 0 end
            redis.call('HSET', job_key, 'state', 'pending', 'worker_id', '',
                'lease_expires_at', '', 'updated_at', ARGV[4])
            redis.call('ZREM', KEYS[1], job_id)
            redis.call('LPUSH', KEYS[2], job_id)
            return 1
        "#;

        for queue in queues {
            let leased_key = self.keys.queue_leased(&queue);
            let pending = self.keys.queue_pending(&queue);
            let expired: Vec<String> = conn
                .zrangebyscore(&leased_key, 0, now_ms as f64)
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;

            for job_id in expired {
                let job_key = self.keys.job(&job_id);
                let did: i32 = redis::Script::new(requeue_script)
                    .key(&leased_key)
                    .key(&pending)
                    .arg(&job_id)
                    .arg(&job_key)
                    .arg(now_ms.to_string())
                    .arg(&updated_at)
                    .invoke_async(&mut conn)
                    .await
                    .map_err(|e| StorageError::Redis(e.to_string()))?;

                if did == 1 {
                    self.append_event(&job_id, "lease_expired").await?;
                    released += 1;
                }
            }
        }
        Ok(released)
    }

    async fn list_jobs(
        &self,
        queue: Option<&str>,
        state: Option<JobState>,
        limit: usize,
    ) -> StorageResult<Vec<Job>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let ids: Vec<String> = conn
            .zrevrange(
                self.keys.recent_jobs(),
                0,
                (limit as isize * 4).max(100) - 1,
            )
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        let mut jobs = Vec::new();
        for id in ids {
            if jobs.len() >= limit {
                break;
            }
            let job = match self.get_job(&id).await {
                Ok(j) => j,
                Err(_) => continue,
            };
            if let Some(q) = queue {
                if job.queue != q {
                    continue;
                }
            }
            if let Some(s) = state {
                if job.state != s {
                    continue;
                }
            }
            jobs.push(job);
        }
        Ok(jobs)
    }

    async fn queue_stats(&self, queue: &str) -> StorageResult<QueueStats> {
        let jobs = self.list_jobs(Some(queue), None, 10_000).await?;
        let mut stats = QueueStats {
            pending: 0,
            leased: 0,
            completed: 0,
            failed: 0,
        };
        for j in jobs {
            match j.state {
                JobState::Pending => stats.pending += 1,
                JobState::Leased => stats.leased += 1,
                JobState::Completed => stats.completed += 1,
                JobState::Failed => stats.failed += 1,
            }
        }
        Ok(stats)
    }

    async fn ping(&self) -> StorageResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(())
    }
}

impl RedisStore {
    async fn try_claim_nonblocking(
        &self,
        worker_id: &str,
        queues: &[String],
        lease_duration_ms: u64,
    ) -> StorageResult<Option<Job>> {
        for queue in queues {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;
            let pending = self.keys.queue_pending(queue);
            let leased_key = self.keys.queue_leased(queue);
            let lease_ms = (Utc::now() + chrono::Duration::milliseconds(lease_duration_ms as i64))
                .timestamp_millis();
            let updated_at = Utc::now().timestamp_millis();
            let job_prefix = self.keys.job_prefix();

            let script = r#"
                local job_id = redis.call('RPOP', KEYS[1])
                if not job_id then return nil end
                local job_key = ARGV[4] .. job_id
                redis.call('HSET', job_key, 'state', 'leased', 'worker_id', ARGV[1],
                    'lease_expires_at', ARGV[2], 'updated_at', ARGV[3])
                redis.call('ZADD', KEYS[2], ARGV[2], job_id)
                return job_id
            "#;

            let job_id: Option<String> = redis::Script::new(script)
                .key(&pending)
                .key(&leased_key)
                .arg(worker_id)
                .arg(lease_ms.to_string())
                .arg(updated_at.to_string())
                .arg(&job_prefix)
                .invoke_async(&mut conn)
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;

            let Some(job_id) = job_id else {
                continue;
            };

            let job = self.get_job(&job_id).await?;
            self.append_event(&job_id, "leased").await?;
            return Ok(Some(job));
        }
        Ok(None)
    }

    async fn claim_job_id(
        &self,
        worker_id: &str,
        queue: &str,
        job_id: &str,
        lease_duration_ms: u64,
    ) -> StorageResult<Option<Job>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let leased_key = self.keys.queue_leased(queue);
        let lease_ms = (Utc::now() + chrono::Duration::milliseconds(lease_duration_ms as i64))
            .timestamp_millis();
        let updated_at = Utc::now().timestamp_millis();
        let job_prefix = self.keys.job_prefix();

        let script = r#"
            local job_id = ARGV[1]
            local job_key = ARGV[5] .. job_id
            local state = redis.call('HGET', job_key, 'state')
            if state ~= 'pending' then return nil end
            redis.call('HSET', job_key, 'state', 'leased', 'worker_id', ARGV[2],
                'lease_expires_at', ARGV[3], 'updated_at', ARGV[4])
            redis.call('ZADD', KEYS[1], ARGV[3], job_id)
            return job_id
        "#;

        let claimed: Option<String> = redis::Script::new(script)
            .key(&leased_key)
            .arg(job_id)
            .arg(worker_id)
            .arg(lease_ms.to_string())
            .arg(updated_at.to_string())
            .arg(&job_prefix)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        let Some(claimed_id) = claimed else {
            return Ok(None);
        };

        let job = self.get_job(&claimed_id).await?;
        self.append_event(&claimed_id, "leased").await?;
        Ok(Some(job))
    }

    async fn trim_recent(&self) -> StorageResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        let _: () = conn
            .zremrangebyrank(
                self.keys.recent_jobs(),
                0,
                -(self.recent_history_limit as isize + 1),
            )
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(())
    }
}

fn queue_from_pending_key(key: &str) -> Option<String> {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() >= 5 && parts[2] == "queue" && parts.last() == Some(&"pending") {
        Some(parts[3].to_string())
    } else {
        None
    }
}
