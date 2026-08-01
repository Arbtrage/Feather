# Embedded monitoring UI

Enable Feather's read-only monitoring UI from the SDK — no separate dashboard container required.

## Overview

When `ui.enabled` is set, the SDK serves a local web UI (queue stats, jobs list, job detail) from static assets bundled in `getfeather`. The UI reads from your **feather-server admin HTTP API** (default `http://localhost:8080`).

You still run feather-server + Redis yourself. The UI runs inside your application process, similar to embedded workers.

## Python

```python
from getfeather import FeatherApp

app = FeatherApp(
    address="localhost:50051",
    ui={"enabled": True, "port": 3001, "admin_url": "http://localhost:8080"},
)

app.task("echo", lambda ctx: None)

await app.start_embedded()  # worker + UI when ui.enabled
# UI: http://127.0.0.1:3001
```

## Configuration

| Option | Env var | Default | Description |
|--------|---------|---------|-------------|
| `ui.enabled` | — | `false` | Serve monitoring UI |
| `ui.port` | — | `3001` | Local HTTP port |
| `ui.admin_url` | `FEATHER_ADMIN_URL` | `http://localhost:8080` | Admin API base URL |
| `ui.open_browser` | — | `false` | Open browser on start |

## CORS

The browser UI calls the admin API on a different port. Configure feather-server to allow your UI origin:

```bash
FEATHER_CORS_ORIGINS=http://localhost:3001,http://localhost:3000
```

Default server config already allows `localhost:3000` and `localhost:3001`.

## Standalone dashboard (optional)

You can still run the Next.js dashboard via Docker Compose (`apps/dashboard`) without SDK integration. See [Dashboard](../operations/dashboard.md).
