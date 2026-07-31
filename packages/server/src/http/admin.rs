use crate::domain::JobState;
use crate::storage::{ActivityQueueStore, QueueStats};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AdminState {
    pub store: Arc<dyn ActivityQueueStore>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct JobJson {
    id: String,
    queue: String,
    name: String,
    payload: serde_json::Value,
    state: String,
    priority: i32,
    attempt: u32,
    worker_id: Option<String>,
    created_at: String,
    lease_expires_at: Option<String>,
    failure_reason: Option<String>,
}

#[derive(Serialize)]
struct QueueInfo {
    name: String,
    pending: u64,
    leased: u64,
    completed: u64,
    failed: u64,
}

#[derive(Deserialize)]
struct JobsQuery {
    queue: Option<String>,
    state: Option<String>,
    limit: Option<usize>,
}

fn job_json(job: &crate::domain::Job) -> JobJson {
    let payload: serde_json::Value =
        serde_json::from_slice(&job.payload).unwrap_or(serde_json::Value::Null);
    JobJson {
        id: job.id.clone(),
        queue: job.queue.clone(),
        name: job.name.clone(),
        payload,
        state: job.state.as_str().into(),
        priority: job.priority,
        attempt: job.attempt,
        worker_id: job.worker_id.clone(),
        created_at: job.created_at.to_rfc3339(),
        lease_expires_at: job.lease_expires_at.map(|t| t.to_rfc3339()),
        failure_reason: job.failure_reason.clone(),
    }
}

pub fn router(state: AdminState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/queues", get(list_queues))
        .route("/api/v1/queues/:name", get(get_queue))
        .route("/api/v1/jobs", get(list_jobs))
        .route("/api/v1/jobs/:id", get(get_job))
        .with_state(state)
}

pub fn router_with_cors(state: AdminState, cors_origins: &[String]) -> Router {
    use axum::http::{HeaderValue, Method};
    use tower_http::cors::{AllowOrigin, Any, CorsLayer};

    let allowed: Vec<HeaderValue> = cors_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_headers(Any);

    router(state).layer(cors)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn ready(State(state): State<AdminState>) -> impl IntoResponse {
    match state.store.ping().await {
        Ok(()) => (StatusCode::OK, Json(HealthResponse { status: "ready" })),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "not_ready",
            }),
        ),
    }
}

async fn list_queues(
    State(state): State<AdminState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state
        .store
        .queue_stats("default")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "data": [queue_info("default", stats)]
    })))
}

async fn get_queue(
    State(state): State<AdminState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state
        .store
        .queue_stats(&name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        serde_json::json!({ "data": queue_info(&name, stats) }),
    ))
}

fn queue_info(name: &str, stats: QueueStats) -> QueueInfo {
    QueueInfo {
        name: name.into(),
        pending: stats.pending,
        leased: stats.leased,
        completed: stats.completed,
        failed: stats.failed,
    }
}

async fn list_jobs(
    State(state): State<AdminState>,
    Query(q): Query<JobsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = q.limit.unwrap_or(50).min(200);
    let state_filter = q.state.as_deref().and_then(|s| match s {
        "pending" => Some(JobState::Pending),
        "leased" => Some(JobState::Leased),
        "completed" => Some(JobState::Completed),
        "failed" => Some(JobState::Failed),
        _ => None,
    });
    let jobs = state
        .store
        .list_jobs(q.queue.as_deref(), state_filter, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let data: Vec<JobJson> = jobs.iter().map(job_json).collect();
    Ok(Json(serde_json::json!({ "data": data })))
}

async fn get_job(
    State(state): State<AdminState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let job = state
        .store
        .get_job(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({ "data": job_json(&job) })))
}

pub async fn serve_http(
    addr: std::net::SocketAddr,
    state: AdminState,
    cors_origins: &[String],
) -> anyhow::Result<()> {
    let app = router_with_cors(state, cors_origins);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
