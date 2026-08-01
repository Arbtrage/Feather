use feather_server::domain::{Job, JobState};
use feather_server::storage::{ActivityQueueStore, RedisStore};
use std::sync::Arc;

fn redis_url() -> Option<String> {
    std::env::var("FEATHER_REDIS_URL").ok().or_else(|| {
        if std::env::var("FEATHER_INTEGRATION").ok().as_deref() == Some("1") {
            Some("redis://127.0.0.1:6379".into())
        } else {
            None
        }
    })
}

fn make_store(url: &str) -> RedisStore {
    let ns = format!("test-{}", uuid::Uuid::new_v4());
    RedisStore::new(url, &ns, 100).expect("redis store")
}

#[tokio::test]
async fn enqueue_dequeue_ack_happy_path() {
    let Some(url) = redis_url() else {
        eprintln!("skip: set FEATHER_INTEGRATION=1 or FEATHER_REDIS_URL");
        return;
    };

    let store = Arc::new(make_store(&url));
    let job = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "echo".into(),
        br#"{"msg":"hi"}"#.to_vec(),
        0,
    );
    let job_id = job.id.clone();

    store.enqueue(job).await.expect("enqueue");
    let leased = store
        .dequeue("worker-1", &["default".into()], 30_000, 0, 1)
        .await
        .expect("dequeue")
        .into_iter()
        .next()
        .expect("job");
    assert_eq!(leased.id, job_id);
    assert_eq!(leased.state, JobState::Leased);

    store.ack(&job_id, "worker-1").await.expect("ack");
    let done = store.get_job(&job_id).await.expect("get");
    assert_eq!(done.state, JobState::Completed);
}

#[tokio::test]
async fn nack_marks_failed() {
    let Some(url) = redis_url() else {
        return;
    };
    let store = Arc::new(make_store(&url));
    let job = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "fail".into(),
        vec![],
        0,
    );
    let job_id = job.id.clone();
    store.enqueue(job).await.unwrap();
    store
        .dequeue("w", &["default".into()], 30_000, 0, 1)
        .await
        .unwrap();

    store.nack(&job_id, "w", "boom").await.unwrap();
    let failed = store.get_job(&job_id).await.unwrap();
    assert_eq!(failed.state, JobState::Failed);
}

#[tokio::test]
async fn concurrent_dequeue_one_winner() {
    let Some(url) = redis_url() else {
        return;
    };
    let store = Arc::new(make_store(&url));
    let job = Job::new(
        uuid::Uuid::now_v7().to_string(),
        "default".into(),
        "once".into(),
        vec![],
        0,
    );
    store.enqueue(job).await.unwrap();

    let s1 = store.clone();
    let s2 = store.clone();
    let queues = vec!["default".to_string()];
    let (a, b) = tokio::join!(
        s1.dequeue("w1", &queues, 30_000, 0, 1),
        s2.dequeue("w2", &queues, 30_000, 0, 1),
    );
    let total_claimed = a.unwrap().len() + b.unwrap().len();
    assert_eq!(total_claimed, 1);
}
