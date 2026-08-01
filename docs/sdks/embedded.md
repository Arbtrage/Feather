# Embedded mode (Celery-style)

Feather supports two deployment models. **Embedded mode is recommended** for most apps — it works like Celery but without a separate worker process.

## Comparison

| | Celery | Feather embedded | Feather dedicated worker | Temporal |
|---|--------|------------------|--------------------------|----------|
| Separate worker process | Usually yes | **No** | Yes (optional) | Yes (required) |
| Separate server/broker | Redis/RabbitMQ | feather-server + Redis | Same | Temporal server |
| Define tasks in app code | `@app.task` | `app.task()` | `worker.task()` | `@workflow` + `@activity` |
| Enqueue from app | `.delay()` | `.delay()` | client.enqueue | client.start |
| Durability | Broker-backed | Redis-backed | Same | Event history |

## When to use embedded mode

Use `FeatherApp` + `start_embedded()` when:

- You run a web API (FastAPI, Django) and want background jobs **in the same deployment**
- You don't want to manage a second worker service
- Job volume fits one process (most apps)

```python
# One Python process: HTTP handlers + worker loop
await app.start_embedded()
app.delay("send-email", {"to": "..."})
```

## When to use a dedicated worker

Use `Worker` + `start()` as a separate process when:

- You need to scale workers independently from your API
- Jobs are CPU-heavy and would starve HTTP handlers
- You want many worker replicas behind one queue

This is optional — not required like Temporal.

## What you still need

Feather is **not** an in-memory queue. You still run:

1. **feather-server** + **Redis** (via Docker Compose) — the durable broker
2. Your app with embedded worker OR a separate worker process

You do **not** need a separate worker **container** unless you choose dedicated mode for scale.

## Optional monitoring UI

Enable the bundled UI from SDK config — no separate dashboard service:

```python
app = FeatherApp(ui={"enabled": True, "port": 3001})
await app.start_embedded()
```

See [Embedded monitoring UI](ui.md).

## Python

```python
from getfeather import FeatherApp

app = FeatherApp()
app.task("my-job", handler)
await app.start_embedded()
app.delay("my-job", {"data": 1})
```

See [Python SDK](python.md) and `examples/embedded-python/`.
