# Roadmap

Feather is being built in phases, each adding capabilities while keeping existing APIs stable.

## Phase 1 — Core Queue (current)

Reliable job processing with lease-based delivery.

- Enqueue, dequeue, ack, nack, extend lease
- Worker registration and heartbeat
- Redis-backed storage
- Python SDK (`getfeather`) with embedded worker + bundled UI
- Blocking dequeue (BRPOP), batch claim, ZSET lease sweeper
- Read-only dashboard
- Docker Compose dev stack

## Phase 2 — Reliability and Scheduling

- Automatic retries with exponential backoff
- Delayed jobs and cron scheduling
- Dead-letter queues (DLQ)
- Idempotency keys and deduplication
- Poison message classification

## Phase 3 — Scaling

- Multiple queue priorities and rate limits
- PostgreSQL storage backend
- Prometheus metrics and distributed tracing
- Server clustering with leader election
- gRPC connection multiplexing

## Phase 4 — Workflow Engine

- Durable workflow orchestration
- Activities (jobs evolve into activity tasks)
- Durable timers, signals, and queries
- Event history and deterministic replay
- Workflow SDK for Python

## Phase 5 — Enterprise

- Authentication and authorization
- Multi-tenancy and namespace isolation
- High-availability deployment patterns
- Audit logging

## Phase 6–10

- Cloud-native deployment (Kubernetes, Helm)
- Distributed systems internals (sharding, partitioning)
- Enterprise platform features
- Production hardening and compliance

Each phase builds on the previous one. Phase 1 APIs remain stable throughout.
