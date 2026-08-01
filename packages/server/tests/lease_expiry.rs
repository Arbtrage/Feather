use feather_server::domain::{Job, JobState};
use feather_server::storage::{ActivityQueueStore, RedisStore};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
        .dequeue("w1", &["default".into()], 500, 0, 1)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(1200)).await;
    store.release_expired_leases().await.unwrap();

    let redelivered = store
        .dequeue("w2", &["default".into()], 500, 0, 1)
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("redelivered");
    assert_eq!(redelivered.id, job_id);
    assert_eq!(redelivered.state, JobState::Leased);
}

#[tokio::test]
async fn lease_expiry_finds_job_outside_recent_index() {
    let Some(url) = redis_url() else {
        return;
    };
    let ns = format!("lease-old-{}", uuid::Uuid::new_v4());
    let store = Arc::new(RedisStore::new(&url, &ns, 5).unwrap());

    let held = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "held".into(),
        vec![],
        0,
    );
    let held_id = held.id.clone();
    store.enqueue(held).await.unwrap();
    store
        .dequeue("w1", &["default".into()], 500, 0, 1)
        .await
        .unwrap();

    for _ in 0..20 {
        let filler = Job::new(
            uuid::Uuid::now_v7().to_string(),
            "default".into(),
            "filler".into(),
            vec![],
            0,
        );
        store.enqueue(filler).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(1200)).await;
    let released = store.release_expired_leases().await.unwrap();
    assert!(released >= 1, "expected held job lease to expire");

    let redelivered = store
        .dequeue("w2", &["default".into()], 500, 0, 1)
        .await
        .unwrap()
        .into_iter()
        .find(|j| j.id == held_id)
        .expect("held job redelivered");
    assert_eq!(redelivered.state, JobState::Leased);
}

#[tokio::test]
async fn dequeue_returns_pre_enqueued_job_without_polling_delay() {
    let Some(url) = redis_url() else {
        return;
    };
    let ns = format!("deq-lat-{}", uuid::Uuid::new_v4());
    let store = Arc::new(RedisStore::new(&url, &ns, 100).unwrap());
    let job = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "fast".into(),
        vec![],
        0,
    );
    store.enqueue(job).await.unwrap();

    let start = Instant::now();
    let jobs = store
        .dequeue("w1", &["default".into()], 30_000, 5_000, 1)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(jobs.len(), 1);
    assert!(
        elapsed < Duration::from_millis(500),
        "dequeue took {:?}, expected sub-second claim",
        elapsed
    );
}

#[tokio::test]
async fn batch_dequeue_claims_multiple_jobs() {
    let Some(url) = redis_url() else {
        return;
    };
    let ns = format!("batch-{}", uuid::Uuid::new_v4());
    let store = Arc::new(RedisStore::new(&url, &ns, 100).unwrap());

    for _ in 0..3 {
        let job = Job::new(
            uuid::Uuid::now_v7().to_string(),
            "default".into(),
            "batch".into(),
            vec![],
            0,
        );
        store.enqueue(job).await.unwrap();
    }

    let jobs = store
        .dequeue("w1", &["default".into()], 30_000, 0, 3)
        .await
        .unwrap();
    assert_eq!(jobs.len(), 3);
}
