use crate::grpc::{QueueGrpc, WorkerGrpc};
use crate::proto::feather::v1::queue_service_server::QueueServiceServer;
use crate::proto::feather::v1::worker_service_server::WorkerServiceServer;
use crate::services::{QueueService, WorkerService};
use std::sync::Arc;
use tonic::transport::Server;

pub async fn serve_grpc(
    addr: std::net::SocketAddr,
    queue: Arc<QueueService>,
    worker: Arc<WorkerService>,
    lease_duration_ms: u64,
) -> anyhow::Result<()> {
    let queue_svc = QueueGrpc { inner: queue };
    let worker_svc = WorkerGrpc {
        inner: worker,
        lease_duration_ms,
    };

    Server::builder()
        .add_service(QueueServiceServer::new(queue_svc))
        .add_service(WorkerServiceServer::new(worker_svc))
        .serve(addr)
        .await?;
    Ok(())
}
