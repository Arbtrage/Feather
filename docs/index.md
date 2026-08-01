# Feather

Feather is a durable execution platform — a Rust control plane with Redis-backed job queues, a Python worker SDK, and a read-only operations dashboard.

Phase 1 delivers reliable **enqueue → lease → execute → ack** job processing over gRPC with at-least-once delivery semantics.

## Architecture

```
Python clients (getfeather)
        │
        ▼ gRPC
┌───────────────────────────┐
│  feather-server (Rust)    │
│  QueueService             │
│  WorkerService            │
│  Admin HTTP (:8080)       │
└───────────┬───────────────┘
            │
            ▼
       Redis 7+
            ▲
            │ HTTP /api/v1
     Dashboard (Next.js)
```

## Phase 1 scope

**Included:**

- Single queue with lease-based job delivery
- Blocking dequeue (Redis BRPOP) and batch claim
- Ack, nack, and lease extension with auto-renewal in the Python worker
- Worker registration and heartbeat
- Read-only dashboard (queue depth, jobs list, job detail)
- Docker Compose dev stack

**Not yet available (later phases):**

- Automatic retries and dead-letter queues
- Cron and delayed jobs
- Workflow orchestration
- Authentication and multi-tenancy
- PostgreSQL storage backend

## Quick links

- [Quickstart](getting-started/quickstart.md) — running locally in minutes
- [Python SDK](sdks/python.md) — enqueue and run workers
- [Performance](operations/performance.md) — throughput and latency notes
- [Configuration](reference/configuration.md) — environment variables
