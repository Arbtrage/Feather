use crate::proto::feather::v1::queue_service_server::QueueService;
use crate::proto::feather::v1::worker_service_server::WorkerService;
use crate::proto::feather::v1::*;
use crate::services::{QueueService as DomainQueue, WorkerService as DomainWorker};
use crate::storage::StorageError;
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct QueueGrpc {
    pub inner: Arc<DomainQueue>,
}

fn job_to_proto(job: &crate::domain::Job) -> Job {
    Job {
        id: job.id.clone(),
        queue: job.queue.clone(),
        name: job.name.clone(),
        payload: job.payload.clone(),
        state: job_state_to_proto(job.state),
        priority: job.priority,
        attempt: job.attempt as i32,
        lease_expires_at: job.lease_expires_at.map(datetime_to_timestamp),
        created_at: Some(datetime_to_timestamp(job.created_at)),
        worker_id: job.worker_id.clone().unwrap_or_default(),
        workflow_run_id: job.workflow_run_id.clone(),
        activity_id: job.activity_id.clone(),
    }
}

fn job_state_to_proto(s: crate::domain::JobState) -> i32 {
    match s {
        crate::domain::JobState::Pending => 1,
        crate::domain::JobState::Leased => 2,
        crate::domain::JobState::Completed => 3,
        crate::domain::JobState::Failed => 4,
    }
}

fn datetime_to_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn map_storage_err(e: StorageError) -> Status {
    match e {
        StorageError::NotFound => Status::not_found("not found"),
        StorageError::PreconditionFailed(msg) => Status::failed_precondition(msg),
        StorageError::Redis(msg) => Status::unavailable(msg),
        StorageError::Other(msg) => Status::invalid_argument(msg),
    }
}

#[tonic::async_trait]
impl QueueService for QueueGrpc {
    async fn enqueue(
        &self,
        request: Request<EnqueueRequest>,
    ) -> Result<Response<EnqueueResponse>, Status> {
        let req = request.into_inner();
        let job = self
            .inner
            .enqueue(&req.queue, &req.name, req.payload, req.priority)
            .await
            .map_err(map_storage_err)?;
        Ok(Response::new(EnqueueResponse {
            job_id: job.id,
            created_at: Some(datetime_to_timestamp(job.created_at)),
        }))
    }

    async fn dequeue(
        &self,
        request: Request<DequeueRequest>,
    ) -> Result<Response<DequeueResponse>, Status> {
        let req = request.into_inner();
        let wait = if req.wait_timeout_ms <= 0 {
            30_000
        } else {
            req.wait_timeout_ms
        };
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(wait as u64);
        let mut empty_streak = 0u32;

        loop {
            match self.inner.dequeue(&req.worker_id, &req.queues, wait).await {
                Ok(Some(job)) => {
                    return Ok(Response::new(DequeueResponse {
                        job: Some(job_to_proto(&job)),
                        backoff_hint_ms: 0,
                        slow_down: false,
                    }));
                }
                Ok(None) => {
                    if tokio::time::Instant::now() >= deadline {
                        let backoff = std::cmp::min(1000 * empty_streak.max(1), 30000);
                        return Ok(Response::new(DequeueResponse {
                            job: None,
                            backoff_hint_ms: backoff as i32,
                            slow_down: false,
                        }));
                    }
                    empty_streak += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(e) => return Err(map_storage_err(e)),
            }
        }
    }

    async fn ack(&self, request: Request<AckRequest>) -> Result<Response<AckResponse>, Status> {
        let req = request.into_inner();
        self.inner
            .ack(&req.job_id, &req.worker_id)
            .await
            .map_err(map_storage_err)?;
        Ok(Response::new(AckResponse {}))
    }

    async fn nack(&self, request: Request<NackRequest>) -> Result<Response<NackResponse>, Status> {
        let req = request.into_inner();
        let reason = if req.reason.is_empty() {
            "nack"
        } else {
            &req.reason
        };
        self.inner
            .nack(&req.job_id, &req.worker_id, reason)
            .await
            .map_err(map_storage_err)?;
        Ok(Response::new(NackResponse {}))
    }

    async fn extend_lease(
        &self,
        request: Request<ExtendLeaseRequest>,
    ) -> Result<Response<ExtendLeaseResponse>, Status> {
        let req = request.into_inner();
        let ext = if req.extension_ms <= 0 {
            30_000u64
        } else {
            req.extension_ms as u64
        };
        let exp = self
            .inner
            .extend_lease(&req.job_id, &req.worker_id, ext)
            .await
            .map_err(map_storage_err)?;
        Ok(Response::new(ExtendLeaseResponse {
            lease_expires_at: Some(datetime_to_timestamp(exp)),
        }))
    }

    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<GetJobResponse>, Status> {
        let req = request.into_inner();
        let job = self
            .inner
            .get_job(&req.job_id)
            .await
            .map_err(map_storage_err)?;
        Ok(Response::new(GetJobResponse {
            job: Some(job_to_proto(&job)),
        }))
    }
}

pub struct WorkerGrpc {
    pub inner: Arc<DomainWorker>,
    pub lease_duration_ms: u64,
}

#[tonic::async_trait]
impl WorkerService for WorkerGrpc {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        self.inner
            .register(
                &req.worker_id,
                req.queues,
                req.capabilities,
                req.labels,
                req.metadata,
            )
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(RegisterResponse {
            lease_duration_ms: self.lease_duration_ms as i32,
            heartbeat_interval_ms: self.inner.heartbeat_interval_ms() as i32,
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        self.inner
            .heartbeat(&req.worker_id)
            .await
            .map_err(Status::not_found)?;
        Ok(Response::new(HeartbeatResponse {}))
    }

    async fn deregister(
        &self,
        request: Request<DeregisterRequest>,
    ) -> Result<Response<DeregisterResponse>, Status> {
        let req = request.into_inner();
        self.inner
            .deregister(&req.worker_id)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(DeregisterResponse {}))
    }
}
