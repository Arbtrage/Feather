# Dashboard

Feather provides a read-only monitoring UI for queue stats and jobs.

## Recommended: SDK embedded UI

Enable the UI from your application when using `@arbitrage/sdk` or `feather-sdk`:

```javascript
const app = new FeatherApp({ ui: { enabled: true, port: 3001 } });
await app.startEmbedded();
// → http://127.0.0.1:3001
```

See [Embedded monitoring UI](../sdks/ui.md) for full configuration.

No separate dashboard container is required when using embedded UI.

## Optional: Docker Compose dashboard

The repo includes a standalone Next.js app at `apps/dashboard` for users who prefer a separate service:

```bash
docker compose -f docker/docker-compose.yml up feather-server redis dashboard
```

Default URL: [http://localhost:3000](http://localhost:3000)

The dashboard reads from the server's HTTP admin API (`http://localhost:8080` by default).

## Pages

### Overview

Shows queue statistics: pending, leased, completed, failed.

### Jobs list

Recent jobs with ID, name, state, queue, and created time.

### Job detail

Full job metadata and JSON payload.

## Configuration (standalone dashboard)

| Environment variable | Default | Description |
|---------------------|---------|-------------|
| `FEATHER_API_URL` | `http://localhost:8080` | Server-side API target (Docker internal) |
| `NEXT_PUBLIC_FEATHER_API_URL` | `http://localhost:8080` | Client-side API target |

## Limitations (Phase 1)

The dashboard is read-only:

- No enqueue or cancel from the UI
- No retry failed jobs
- No worker management
- No real-time updates (refresh on navigation)
