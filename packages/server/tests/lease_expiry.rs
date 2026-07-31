use feather_server::domain::{Job, JobState};
use feather_server::storage::{ActivityQueueStore, RedisStore};
use std::sync::Arc;
use std::time::Duration;

fn redis_url() -> Option<String> {
    std::env::var("FEATHER_REDIS_URL").ok().or_else(|| {
        if std::env::var("FEATHER_INTEGRATION").ok().as_deref() == Some("1") {
            Some("redis://127.0.0.1:6379".into())
        } else {
            None
        }
    })
}

#[tokio::test]
async fn lease_expiry_redelivers_job() {
    let Some(url) = redis_url() else {
        return;
    };
    let ns = format!("lease-{}", uuid::Uuid::new_v4());
    let store = Arc::new(RedisStore::new(&url, &ns, 100).unwrap());
    let job = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "slow".into(),
        vec![],
        0,
    );
    let job_id = job.id.clone();
    store.enqueue(job).await.unwrap();
    store
        .dequeue("w1", &["default".into()], 500)
        .await
        .unwrap()
        .expect("leased");

    tokio::time::sleep(Duration::from_millis(1200)).await;
    store.release_expired_leases().await.unwrap();

    let redelivered = store
        .dequeue("w2", &["default".into()], 500)
        .await
        .unwrap()
        .expect("redelivered");
    assert_eq!(redelivered.id, job_id);
    assert_eq!(redelivered.state, JobState::Leased);
}
