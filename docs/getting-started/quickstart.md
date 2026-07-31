# Quickstart

Get Feather running locally and process your first jobs in under five minutes.

## 1. Start the backend

```bash
docker compose -f docker/docker-compose.yml up --build redis feather-server
```

This starts:

| Service | URL |
|---------|-----|
| gRPC server | `localhost:50051` |
| Admin HTTP API | `http://localhost:8080` |

## 2. Run embedded worker + UI

In a new terminal:

```bash
cd examples/embedded-node
npm install
FEATHER_ADDRESS=localhost:50051 npm start
```

This starts an Express API, an in-process worker, and the monitoring UI at [http://127.0.0.1:3001](http://127.0.0.1:3001).

## 3. Enqueue a job

```bash
curl http://localhost:4000/enqueue
```

Watch the worker logs and refresh the UI — the job moves from pending → leased → completed.

## Alternative: dedicated worker

```bash
cd examples/node-worker
npm install
FEATHER_ADDRESS=localhost:50051 npm start   # worker
npm run enqueue                              # enqueue jobs
```

## Optional standalone dashboard

```bash
docker compose -f docker/docker-compose.yml --profile dashboard up --build
```

Open [http://localhost:3000](http://localhost:3000).

## Python worker (optional)

```bash
cd examples/python-worker
python worker.py
```

Workers are language-agnostic.

## Next steps

- [Embedded UI](../sdks/ui.md)
- [Docker Compose details](docker-compose.md)
- [Node.js SDK](../sdks/node.md)
- [Configuration reference](../reference/configuration.md)
