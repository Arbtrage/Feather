use super::keys::KeyBuilder;
use super::traits::{ActivityQueueStore, QueueStats, StorageError, StorageResult};
use crate::domain::{Job, JobState};
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use std::collections::HashMap;

pub struct RedisStore {
    pool: Pool,
    keys: KeyBuilder,
    recent_history_limit: usize,
}

impl RedisStore {
    pub fn new(redis_url: &str, namespace: &str, recent_history_limit: usize) -> StorageResult<Self> {
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
            ("created_at".into(), job.created_at.timestamp_millis().to_string()),
            ("updated_at".into(), job.updated_at.timestamp_millis().to_string()),
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
            state: JobState::from_str(map.get("state")?)?,
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
            lease_renewals: map.get("lease_renewals").and_then(|s| s.parse().ok()).unwrap_or(0),
        })
    }

    async fn append_event(&self, job_id: &str, event: &str) -> StorageResult<()> {
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
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
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(&job.id);
        let pending = self.keys.queue_pending(&job.queue);
        let recent = self.keys.recent_jobs();

        let fields = Self::job_to_hash(&job);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset_multiple(&job_key, &fields);
        pipe.lpush(&pending, &job.id);
        pipe.zadd(&recent, job.id.as_str(), job.created_at.timestamp_millis() as f64);
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
    ) -> StorageResult<Option<Job>> {
        let lease_ms = (Utc::now() + chrono::Duration::milliseconds(lease_duration_ms as i64))
            .timestamp_millis();
        let updated_at = Utc::now().timestamp_millis();
        let job_prefix = self.keys.job_prefix();

        for queue in queues {
            let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
            let pending = self.keys.queue_pending(queue);
            let leased_key = self.keys.queue_leased(queue);

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

            let Some(job_id) = job_id else { continue };

            let job = self.get_job(&job_id).await?;
            self.append_event(&job_id, "leased").await?;
            return Ok(Some(job));
        }
        Ok(None)
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

        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let leased_key = self.keys.queue_leased(&job.queue);

        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&job_key, "state", "completed");
        pipe.hset(&job_key, "updated_at", Utc::now().timestamp_millis().to_string());
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

        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let leased_key = self.keys.queue_leased(&job.queue);

        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&job_key, "state", "failed");
        pipe.hset(&job_key, "failure_reason", reason);
        pipe.hset(&job_key, "updated_at", Utc::now().timestamp_millis().to_string());
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

        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
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
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let job_key = self.keys.job(job_id);
        let map: HashMap<String, String> = conn.hgetall(&job_key).await.map_err(|e| StorageError::Redis(e.to_string()))?;
        if map.is_empty() {
            return Err(StorageError::NotFound);
        }
        Self::hash_to_job(map).ok_or(StorageError::Other("corrupt job".into()))
    }

    async fn release_expired_leases(&self) -> StorageResult<u64> {
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let now_ms = Utc::now().timestamp_millis() as f64;
        let mut released = 0u64;

        // Scan all leased keys via recent jobs index queues - simplified: scan known queue "default"
        // Production: track queue registry; Phase 1 uses pattern from job hash queue field
        let recent: Vec<String> = conn
            .zrevrange(self.keys.recent_jobs(), 0, 999)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;

        for job_id in recent {
            let job = match self.get_job(&job_id).await {
                Ok(j) => j,
                Err(_) => continue,
            };
            if job.state != JobState::Leased {
                continue;
            }
            let Some(exp) = job.lease_expires_at else { continue };
            if exp.timestamp_millis() as f64 > now_ms {
                continue;
            }

            let pending = self.keys.queue_pending(&job.queue);
            let job_key = self.keys.job(&job_id);
            let leased_key = self.keys.queue_leased(&job.queue);

            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.hset(&job_key, "state", "pending");
            pipe.hset(&job_key, "worker_id", "");
            pipe.hset(&job_key, "lease_expires_at", "");
            pipe.hset(&job_key, "updated_at", Utc::now().timestamp_millis().to_string());
            pipe.zrem(&leased_key, &job_id);
            pipe.lpush(&pending, &job_id);
            pipe.query_async::<()>(&mut conn)
                .await
                .map_err(|e| StorageError::Redis(e.to_string()))?;

            self.append_event(&job_id, "lease_expired").await?;
            released += 1;
        }
        Ok(released)
    }

    async fn list_jobs(
        &self,
        queue: Option<&str>,
        state: Option<JobState>,
        limit: usize,
    ) -> StorageResult<Vec<Job>> {
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let ids: Vec<String> = conn
            .zrevrange(self.keys.recent_jobs(), 0, (limit as isize * 4).max(100) - 1)
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
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(())
    }
}

impl RedisStore {
    async fn trim_recent(&self) -> StorageResult<()> {
        let mut conn = self.pool.get().await.map_err(|e| StorageError::Redis(e.to_string()))?;
        let _: () = conn
            .zremrangebyrank(
                self.keys.recent_jobs(),
                0,
                -(self.recent_history_limit as i64 + 1),
            )
            .await
            .map_err(|e| StorageError::Redis(e.to_string()))?;
        Ok(())
    }
}
