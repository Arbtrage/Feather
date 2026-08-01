use crate::config::AppConfig;
use crate::domain::Job;
use crate::storage::{ActivityQueueStore, StorageError};
use std::sync::Arc;
use uuid::Uuid;

pub struct QueueService {
    store: Arc<dyn ActivityQueueStore>,
    config: AppConfig,
}

impl QueueService {
    pub fn new(store: Arc<dyn ActivityQueueStore>, config: AppConfig) -> Self {
        Self { store, config }
    }

    pub fn validate_queue(name: &str) -> Result<(), String> {
        if name.is_empty() || name.len() > 64 {
            return Err("queue name must be 1-64 chars".into());
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("invalid queue name".into());
        }
        Ok(())
    }

    pub fn validate_task_name(name: &str) -> Result<(), String> {
        if name.is_empty() || name.len() > 128 {
            return Err("task name must be 1-128 chars".into());
        }
        Ok(())
    }

    pub async fn enqueue(
        &self,
        queue: &str,
        name: &str,
        payload: Vec<u8>,
        priority: i32,
    ) -> Result<Job, StorageError> {
        Self::validate_queue(queue).map_err(StorageError::Other)?;
        Self::validate_task_name(name).map_err(StorageError::Other)?;
        if payload.len() > self.config.max_payload_bytes {
            return Err(StorageError::Other(format!(
                "payload exceeds max {} bytes",
                self.config.max_payload_bytes
            )));
        }

        let queue = if queue.is_empty() {
            "default".to_string()
        } else {
            queue.to_string()
        };

        let id = Uuid::now_v7().to_string();
        let job = Job::new(id, queue, name.to_string(), payload, priority);
        self.store.enqueue(job.clone()).await?;
        Ok(job)
    }

    pub async fn dequeue(
        &self,
        worker_id: &str,
        queues: &[String],
        wait_timeout_ms: i32,
        max_jobs: i32,
    ) -> Result<Vec<Job>, StorageError> {
        let qs: Vec<String> = if queues.is_empty() {
            vec!["default".to_string()]
        } else {
            queues.to_vec()
        };
        let wait = if wait_timeout_ms <= 0 {
            0
        } else {
            wait_timeout_ms as u64
        };
        let max = if max_jobs <= 0 { 1 } else { max_jobs as u32 };
        self.store
            .dequeue(
                worker_id,
                &qs,
                self.config.default_lease_duration_ms,
                wait,
                max,
            )
            .await
    }

    pub async fn ack(&self, job_id: &str, worker_id: &str) -> Result<(), StorageError> {
        self.store.ack(job_id, worker_id).await
    }

    pub async fn nack(
        &self,
        job_id: &str,
        worker_id: &str,
        reason: &str,
    ) -> Result<(), StorageError> {
        self.store.nack(job_id, worker_id, reason).await
    }

    pub async fn extend_lease(
        &self,
        job_id: &str,
        worker_id: &str,
        extension_ms: u64,
    ) -> Result<chrono::DateTime<chrono::Utc>, StorageError> {
        self.store
            .extend_lease(
                job_id,
                worker_id,
                extension_ms,
                self.config.max_lease_renewals,
            )
            .await
    }

    pub async fn get_job(&self, job_id: &str) -> Result<Job, StorageError> {
        self.store.get_job(job_id).await
    }
}
