# Leases and Delivery

Feather guarantees **at-least-once delivery** using lease-based visibility timeouts.

## How leases work

1. A worker dequeues a job → server sets state to `leased` and starts a lease timer (default: 30 seconds).
2. While leased, no other worker can claim the same job.
3. On success, the worker calls **ack** → state becomes `completed`.
4. On failure, the worker calls **nack** → state becomes `failed`.
5. If the lease expires before ack/nack, the job returns to `pending` and is re-delivered.

```
Worker A dequeues job
        │
        ▼
   [leased, 30s TTL]
        │
   ┌────┴────┐
   │         │
  ack       nack
   │         │
completed  failed

   (lease expires)
        │
        ▼
    pending → re-delivered to Worker B
```

## Lease renewal

Long-running jobs can extend their lease before it expires:

```javascript
// SDK auto-renews at 50% of lease TTL
await client.extendLease({ jobId, workerId, extensionMs: 30000 });
```

The Node and Python SDKs renew leases automatically at 50% of the TTL. The server enforces a maximum renewal count (default: 100) to prevent infinite leases.

## Delivery semantics

| Scenario | Behavior |
|----------|----------|
| Worker crashes mid-job | Job re-delivered after lease expiry |
| Worker acks after crash | Idempotent ack — already completed jobs are no-ops |
| Network partition | At-least-once: job may be delivered again after lease expiry |
| Duplicate dequeue | Prevented by atomic Lua script (one winner per job) |

Phase 1 does **not** provide exactly-once delivery. Design handlers to be idempotent where possible. Automatic deduplication arrives in Phase 2.

## Nack and failure classification

```javascript
await ctx.nack("connection timeout");
```

Nack fields (Phase 1):

| Field | Purpose |
|-------|---------|
| `reason` | Human-readable failure message |
| `retryable` | Reserved for Phase 2 retry routing |
| `failure_class` | Reserved for poison-message classification (Phase 2 DLQ) |

Failed jobs are visible in the dashboard with their failure reason.
