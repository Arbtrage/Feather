use feather_server::config::AppConfig;
use feather_server::grpc::server::serve_grpc;
use feather_server::http::{serve_http, AdminState};
use feather_server::observability;
use feather_server::services::{run_lease_sweeper, QueueService, WorkerService};
use feather_server::storage::RedisStore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    observability::init_tracing(
        &config.observability.log_level,
        &config.observability.log_format,
    );

    tracing::info!("starting feather-server");

    let store = Arc::new(RedisStore::new(
        &config.storage.redis_url,
        &config.storage.namespace,
        config.recent_history_limit,
    )?);

    let store_for_sweeper = store.clone();
    let sweep_ms = config.lease_sweep_interval_ms;
    tokio::spawn(async move {
        run_lease_sweeper(store_for_sweeper, sweep_ms).await;
    });

    let worker_pool = deadpool_redis::Config::from_url(&config.storage.redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let queue_service = Arc::new(QueueService::new(store.clone(), config.clone()));
    let worker_service = Arc::new(WorkerService::new(
        worker_pool,
        &config.storage.namespace,
        config.heartbeat_interval_ms,
    ));

    let grpc_addr: std::net::SocketAddr = config.server.grpc_addr.parse()?;
    let http_addr: std::net::SocketAddr = config.server.http_addr.parse()?;

    let admin_state = AdminState {
        store: store.clone(),
    };
    let cors_origins = config.cors_origins.clone();
    let lease_ms = config.default_lease_duration_ms;

    let grpc = tokio::spawn(async move {
        serve_grpc(grpc_addr, queue_service, worker_service, lease_ms).await
    });
    let http = tokio::spawn(async move { serve_http(http_addr, admin_state, &cors_origins).await });

    tokio::select! {
        r = grpc => r??,
        r = http => r??,
    }

    Ok(())
}
