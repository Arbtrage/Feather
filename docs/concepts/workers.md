# Workers

Workers are long-running processes that poll Feather for jobs, execute user code, and report results.

## Lifecycle

```
1. Register  ──► server returns lease_duration_ms, heartbeat_interval_ms
2. Poll loop ──► Dequeue (blocking on server via Redis BRPOP, up to 30s)
3. Execute   ──► dispatch to handler by task name
4. Ack/Nack  ──► report outcome
5. Heartbeat ──► periodic keep-alive (every 10s default)
6. Deregister► clean shutdown on SIGINT
```

## Registration

Workers identify themselves and declare which queues they serve:

```python
worker = Worker(
    address="localhost:50051",
    worker_id="python-worker-1",
    queues=["default", "high-priority"],
)
```

Registration includes optional metadata (Phase 1):

| Field | Description |
|-------|-------------|
| `capabilities` | Feature tags for future routing |
| `labels` | Key-value metadata |
| `resource_profile` | CPU/memory/GPU hints |
| `metadata` | Arbitrary key-value pairs |

## Polling

Workers use pull-based long-polling:

- Send `Dequeue` with `wait_timeout_ms` (default 30,000 ms) and optional `max_jobs`
- Server returns jobs immediately if available (non-blocking RPOP first)
- If the queue is empty, the server blocks on Redis BRPOP until a job arrives or the timeout expires
- Empty responses include a `backoff_hint_ms` for client backoff

## Lease renewal

The Python worker renews leases at **50% of the registered lease TTL** while a job runs, via `ExtendLease`. This prevents false redelivery for tasks that exceed the default lease duration.

## Heartbeat

Workers send periodic heartbeats to remain visible to the server:

- Default interval: 10 seconds (returned at registration)
- Missed heartbeats beyond the offline threshold (30s) mark the worker as offline
- Phase 1 uses heartbeats for visibility only; lease expiry handles job recovery

## Handler dispatch

Register handlers by task name:

```python
async def echo(ctx):
    data = json.loads(ctx.payload.decode())
    print("received:", data)

worker.task("echo", echo)
worker.task("send-email", send_email)
```

Each handler receives a context with:

| Property | Description |
|----------|-------------|
| `ctx.id` | Job ID |
| `ctx.queue` | Queue name |
| `ctx.name` | Task name |
| `ctx.payload` | Raw payload bytes |
| `ctx.ack()` | Mark job completed |
| `ctx.nack(reason)` | Mark job failed |

## Graceful shutdown

Press `Ctrl+C` (SIGINT) to stop the worker:

1. Stop polling for new jobs
2. Deregister from the server
3. Close gRPC connections

Jobs in progress should be acked or nacked before shutdown. Unfinished jobs are re-delivered after lease expiry.
