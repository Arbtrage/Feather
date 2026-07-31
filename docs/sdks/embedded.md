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

Use `FeatherApp` + `startEmbedded()` when:

- You run a web API (Express, FastAPI, Django) and want background jobs **in the same deployment**
- You don't want to manage a second worker service
- Job volume fits one process (most apps)

```javascript
// One Node process: HTTP server + worker loop
await app.startEmbedded();
await app.delay("send-email", { to: "..." });
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

```javascript
const app = new FeatherApp({ ui: { enabled: true, port: 3001 } });
await app.startEmbedded();
```

See [Embedded monitoring UI](ui.md).

## Node.js

```javascript
import { FeatherApp } from "@arbtrage/sdk";

const app = new FeatherApp();
app.task("my-job", async (ctx) => { /* ... */ });
await app.startEmbedded();
await app.delay("my-job", { data: 1 });
```

See [Node.js SDK](node.md) and `examples/embedded-node/`.

## Python

```python
from feather import FeatherApp

app = FeatherApp()
app.task("my-job", handler)
await app.start_embedded()
app.delay("my-job", {"data": 1})
```

See [Python SDK](python.md) and `examples/embedded-python/`.
