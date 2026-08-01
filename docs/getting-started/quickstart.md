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

## 2. Install the Python SDK

```bash
pip install getfeather
# or from monorepo:
pip install ./packages/sdk-python
```

## 3. Run embedded worker + UI

In a new terminal:

```bash
cd examples/embedded-python
python app.py
```

This starts an in-process worker and the monitoring UI at [http://127.0.0.1:3001](http://127.0.0.1:3001) when `ui.enabled` is set.

## 4. Enqueue jobs

The embedded example enqueues a sample job on startup. For a dedicated enqueue script:

```bash
cd examples/python-worker
python enqueue.py
```

Watch the worker logs and refresh the UI — jobs move from pending → leased → completed.

## Dedicated worker (optional)

Scale workers independently from your API:

```bash
cd examples/python-worker
python worker.py          # worker
python enqueue.py         # enqueue jobs
```

## Optional standalone dashboard

```bash
docker compose -f docker/docker-compose.yml --profile dashboard up --build
```

Open [http://localhost:3000](http://localhost:3000).

## Next steps

- [Embedded UI](../sdks/ui.md)
- [Docker Compose details](docker-compose.md)
- [Python SDK](../sdks/python.md)
- [Configuration reference](../reference/configuration.md)
