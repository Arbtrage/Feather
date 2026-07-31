//! Load benchmark: enqueue throughput baseline.
//! Run: FEATHER_INTEGRATION=1 cargo test --test enqueue_bench -- --ignored --nocapture

use feather_server::domain::Job;
use feather_server::storage::RedisStore;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
#[ignore]
async fn enqueue_bench_500_per_sec() {
    let url = std::env::var("FEATHER_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let ns = format!("bench-{}", uuid::Uuid::new_v4());
    let store = Arc::new(RedisStore::new(&url, &ns, 50_000).expect("store"));

    let n = 1000usize;
    let start = Instant::now();
    for i in 0..n {
        let job = Job::new(
            uuid::Uuid::now_v7().to_string(),
            "default".into(),
            "bench".into(),
            format!("{{\"i\":{i}}}").into_bytes(),
            0,
        );
        store.enqueue(job).await.expect("enqueue");
    }
    let elapsed = start.elapsed().as_secs_f64();
    let rate = n as f64 / elapsed;
    println!("enqueue rate: {rate:.0}/s ({n} jobs in {elapsed:.2}s)");
    assert!(rate >= 500.0, "expected >= 500/s, got {rate:.0}/s");
}
