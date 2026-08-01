use crate::domain::{Job, JobState};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("not found")]
    NotFound,
    #[error("precondition failed: {0}")]
    PreconditionFailed(String),
    #[error("redis error: {0}")]
    Redis(String),
    #[error("{0}")]
    Other(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending: u64,
    pub leased: u64,
    pub completed: u64,
    pub failed: u64,
}

#[async_trait]
pub trait ActivityQueueStore: Send + Sync {
    async fn enqueue(&self, job: Job) -> StorageResult<()>;
    async fn dequeue(
        &self,
        worker_id: &str,
        queues: &[String],
        lease_duration_ms: u64,
        wait_timeout_ms: u64,
        max_jobs: u32,
    ) -> StorageResult<Vec<Job>>;
    async fn ack(&self, job_id: &str, worker_id: &str) -> StorageResult<()>;
    async fn nack(&self, job_id: &str, worker_id: &str, reason: &str) -> StorageResult<()>;
    async fn extend_lease(
        &self,
        job_id: &str,
        worker_id: &str,
        extension_ms: u64,
        max_renewals: u32,
    ) -> StorageResult<DateTime<Utc>>;
    async fn get_job(&self, job_id: &str) -> StorageResult<Job>;
    async fn release_expired_leases(&self) -> StorageResult<u64>;
    async fn list_jobs(
        &self,
        queue: Option<&str>,
        state: Option<JobState>,
        limit: usize,
    ) -> StorageResult<Vec<Job>>;
    async fn queue_stats(&self, queue: &str) -> StorageResult<QueueStats>;
    async fn ping(&self) -> StorageResult<()>;
}
