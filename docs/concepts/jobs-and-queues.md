# Jobs and Queues

## Jobs

A **job** is a unit of work submitted to Feather. Each job has:

| Field | Description |
|-------|-------------|
| `id` | UUID v7 (time-ordered, globally unique) |
| `queue` | Target queue name (default: `default`) |
| `name` | Task type — used by workers to dispatch handlers |
| `payload` | Opaque bytes (typically JSON) |
| `state` | Current lifecycle state |
| `priority` | Integer priority (reserved for Phase 3) |
| `attempt` | Attempt number (always `1` in Phase 1) |

Reserved fields for future workflow support (empty in Phase 1):

- `workflow_run_id`
- `activity_id`

## Job states

```
pending ──► leased ──► completed
                │
                └──► failed
                │
                └──► pending  (lease expired → re-delivered)
```

| State | Meaning |
|-------|---------|
| `pending` | Enqueued, waiting for a worker |
| `leased` | Claimed by a worker, lease clock running |
| `completed` | Worker acknowledged success |
| `failed` | Worker nacked or terminal failure |

## Queues

Queues are named channels for job routing. Phase 1 supports unlimited queue names with a single logical namespace per server instance.

**Queue name rules:**

- 1–64 characters
- ASCII alphanumeric, hyphens, underscores
- Empty queue name defaults to `default`

Workers subscribe to one or more queues at registration time.

## Enqueueing

```javascript
const { jobId } = await client.enqueue({
  queue: "default",
  name: "send-email",
  payload: Buffer.from(JSON.stringify({ to: "user@example.com" })),
  priority: 0,
});
```

**Payload limits:** Maximum 256 KB per job (262,144 bytes). Oversized payloads are rejected with a clear error.

## Task names

The `name` field is the routing key. Workers register handlers by task name:

```javascript
worker.task("send-email", async (ctx) => { /* ... */ });
worker.task("process-image", async (ctx) => { /* ... */ });
```

If no handler matches the task name, the worker nacks the job.
