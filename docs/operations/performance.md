# Performance

Feather Phase 1 optimizes the hot path: **enqueue**, **blocking dequeue**, **batch claim**, and **lease recovery**.

## Server (feather-server)

| Path | Behavior |
|------|----------|
| Dequeue | Non-blocking RPOP + lease claim; falls back to **BRPOP** until gRPC wait deadline |
| Batch dequeue | Up to `max_jobs` per Dequeue RPC (default 1) |
| Lease sweeper | **ZRANGEBYSCORE** on per-queue leased ZSETs via queue registry — not limited to recent job index |
| Enqueue | Pipelined HSET + LPUSH + registry SADD |

## Benchmarks

Run with Redis available:

```bash
FEATHER_INTEGRATION=1 cargo test --test enqueue_bench -- --ignored --nocapture
```

| Benchmark | Target |
|-----------|--------|
| `enqueue_bench_500_per_sec` | ≥ 500 enqueues/s (1000 jobs) |
| `dequeue_latency_bench` | p99 claim latency < 500ms (20 samples; typically sub-100ms with BRPOP) |

Integration tests in `packages/server/tests/lease_expiry.rs` also assert sub-second dequeue for pre-enqueued jobs and batch claim of 3 jobs.

## Python worker

- **Long-poll dequeue** — single gRPC call blocks on the server (no client-side 500ms poll loop)
- **Auto lease renewal** — `ExtendLease` at 50% of registered lease TTL while a job runs
- **Batch processing** — optional `max_jobs` on dequeue; concurrent handlers via `asyncio.TaskGroup` (bounded by `max_concurrency`)

```python
worker = Worker(max_jobs=8, max_concurrency=4)
```

## Deferred (next milestone)

Phase 2 reliability work (retries, DLQ, idempotency keys, delayed jobs) is intentionally deferred until throughput and lease correctness are proven in production-like loads.

See [Roadmap](../roadmap.md).
