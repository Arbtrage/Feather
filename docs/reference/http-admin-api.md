# HTTP Admin API

The server exposes a read-only HTTP admin API on port 8080 (default). Used by the dashboard and for operational debugging.

## Health endpoints

### GET /health

Liveness check. Always returns 200.

```json
{ "status": "ok" }
```

### GET /ready

Readiness check. Returns 200 if Redis is reachable, 503 otherwise.

```json
{ "status": "ready" }
```

## Queue endpoints

### GET /api/v1/queues

List all queues with stats.

```json
{
  "data": [
    {
      "name": "default",
      "pending": 5,
      "leased": 2,
      "completed": 100,
      "failed": 1
    }
  ]
}
```

### GET /api/v1/queues/:name

Stats for a specific queue.

```json
{
  "data": {
    "name": "default",
    "pending": 5,
    "leased": 2,
    "completed": 100,
    "failed": 1
  }
}
```

## Job endpoints

### GET /api/v1/jobs

List recent jobs with optional filters.

Query parameters:

| Param | Type | Description |
|-------|------|-------------|
| `queue` | string | Filter by queue name |
| `state` | string | Filter by state (`pending`, `leased`, `completed`, `failed`) |
| `limit` | integer | Max results (default 50, max 200) |

```json
{
  "data": [
    {
      "id": "0195a1b2-...",
      "queue": "default",
      "name": "echo",
      "payload": { "n": 1 },
      "state": "completed",
      "priority": 0,
      "attempt": 1,
      "worker_id": "node-12345",
      "created_at": "2026-08-01T10:00:00Z",
      "lease_expires_at": null,
      "failure_reason": null
    }
  ]
}
```

### GET /api/v1/jobs/:id

Get a single job by ID.

```json
{
  "data": {
    "id": "0195a1b2-...",
    "queue": "default",
    "name": "echo",
    "payload": { "n": 1 },
    "state": "completed",
    "priority": 0,
    "attempt": 1,
    "worker_id": "node-12345",
    "created_at": "2026-08-01T10:00:00Z"
  }
}
```

Returns 404 if the job does not exist.

## Examples

```bash
# Queue stats
curl http://localhost:8080/api/v1/queues

# Recent completed jobs
curl "http://localhost:8080/api/v1/jobs?state=completed&limit=10"

# Job detail
curl http://localhost:8080/api/v1/jobs/0195a1b2-...
```
