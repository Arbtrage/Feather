use crate::domain::WorkerRecord;
use crate::storage::keys::KeyBuilder;
use chrono::Utc;
use deadpool_redis::Pool;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::sync::Arc;

pub struct WorkerService {
    pool: Pool,
    keys: KeyBuilder,
    heartbeat_interval_ms: u64,
}

impl WorkerService {
    pub fn new(pool: Pool, namespace: &str, heartbeat_interval_ms: u64) -> Self {
        Self {
            pool,
            keys: KeyBuilder::new(namespace),
            heartbeat_interval_ms,
        }
    }

    pub fn heartbeat_interval_ms(&self) -> u64 {
        self.heartbeat_interval_ms
    }

    pub async fn register(
        &self,
        worker_id: &str,
        queues: Vec<String>,
        capabilities: Vec<String>,
        labels: HashMap<String, String>,
        metadata: HashMap<String, String>,
    ) -> Result<(), String> {
        let now = Utc::now();
        let record = WorkerRecord {
            id: worker_id.to_string(),
            queues,
            capabilities,
            labels,
            registered_at: now,
            last_heartbeat_at: now,
            status: "active".into(),
            metadata,
        };
        let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let key = self.keys.worker(worker_id);
        let active = self.keys.workers_active();
        let _: () = conn.hset(&key, "data", json).await.map_err(|e| e.to_string())?;
        let _: () = conn.sadd(&active, worker_id).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn heartbeat(&self, worker_id: &str) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let key = self.keys.worker(worker_id);
        let data: Option<String> = conn.hget(&key, "data").await.map_err(|e| e.to_string())?;
        let Some(data) = data else {
            return Err("worker not registered".into());
        };
        let mut record: WorkerRecord = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        record.last_heartbeat_at = Utc::now();
        let json = serde_json::to_string(&record).map_err(|e| e.to_string())?;
        let _: () = conn.hset(&key, "data", json).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn deregister(&self, worker_id: &str) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let key = self.keys.worker(worker_id);
        let active = self.keys.workers_active();
        let _: () = conn.del(&key).await.map_err(|e| e.to_string())?;
        let _: () = conn.srem(&active, worker_id).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub async fn run_lease_sweeper(store: Arc<dyn crate::storage::ActivityQueueStore>, interval_ms: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
    loop {
        interval.tick().await;
        match store.release_expired_leases().await {
            Ok(n) if n > 0 => tracing::info!(released = n, "expired leases released"),
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "lease sweeper error"),
        }
    }
}
