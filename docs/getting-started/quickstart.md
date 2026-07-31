# Quickstart

Get Feather running locally and process your first jobs in under five minutes.

## 1. Start the stack

```bash
docker compose -f docker/docker-compose.yml up --build
```

This starts:

| Service | URL |
|---------|-----|
| gRPC server | `localhost:50051` |
| Admin HTTP API | `http://localhost:8080` |
| Dashboard | `http://localhost:3000` |

## 2. Start a worker

In a new terminal:

```bash
cd examples/node-worker
npm install
FEATHER_ADDRESS=localhost:50051 npm start
```

The worker registers, polls for jobs, and handles tasks named `echo`.

## 3. Enqueue jobs

In another terminal:

```bash
cd examples/node-worker
npm run enqueue
```

This enqueues 10 jobs. Watch the worker terminal — each job is claimed, executed, and acknowledged.

## 4. View the dashboard

Open [http://localhost:3000](http://localhost:3000):

- **Overview** — queue depth (pending, leased, completed, failed)
- **Jobs** — list of recent jobs with state badges
- **Job detail** — click any job to see its JSON payload

## Python worker (optional)

```bash
cd examples/python-worker
python worker.py
```

Then enqueue from Node or Python — workers are language-agnostic.

## Next steps

- [Docker Compose details](docker-compose.md)
- [Node.js SDK](../sdks/node.md)
- [Configuration reference](../reference/configuration.md)
