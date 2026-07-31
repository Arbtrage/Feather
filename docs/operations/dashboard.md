# Dashboard

The Feather dashboard is a read-only Next.js application for operational visibility.

## Access

Default URL: [http://localhost:3000](http://localhost:3000) (when running via Docker Compose).

The dashboard reads from the server's HTTP admin API (`http://localhost:8080` by default).

## Pages

### Overview

Route: `/`

Shows queue statistics cards:

- **Pending** — jobs waiting for a worker
- **Leased** — jobs currently being processed
- **Completed** — successfully finished jobs
- **Failed** — jobs that were nacked or failed

### Jobs list

Route: `/jobs`

Table of recent jobs with:

| Column | Description |
|--------|-------------|
| ID | Truncated job ID (click for detail) |
| Name | Task type |
| State | Badge: pending, leased, completed, failed |
| Queue | Queue name |
| Created | Timestamp |

### Job detail

Route: `/jobs/:id`

Full job information:

- ID, state, queue, worker ID
- Created timestamp
- Failure reason (if failed)
- Payload (formatted JSON)

## Configuration

| Environment variable | Default | Description |
|---------------------|---------|-------------|
| `FEATHER_API_URL` | `http://localhost:8080` | Server-side API target (Docker internal) |
| `NEXT_PUBLIC_FEATHER_API_URL` | `http://localhost:8080` | Client-side API target |

In Docker Compose, the dashboard uses `FEATHER_API_URL=http://feather-server:8080` for server-side rendering and `NEXT_PUBLIC_FEATHER_API_URL=http://localhost:8080` for any client-side requests.

## Development

```bash
cd apps/dashboard
npm install
npm run dev     # http://localhost:3000
```

Requires the feather-server admin HTTP API to be running on port 8080.

## Limitations (Phase 1)

The dashboard is read-only. The following are not available:

- Enqueue or cancel jobs from the UI
- Retry failed jobs
- Worker management
- Real-time updates (pages refresh on navigation)

These features are planned for later phases.
