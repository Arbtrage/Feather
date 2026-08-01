# Python SDK

Package: `getfeather` on [PyPI](https://pypi.org/project/getfeather/) (monorepo: `packages/sdk-python/`)

Requires Python 3.11+.

## Install

```bash
pip install getfeather
uv pip install getfeather
pipx install getfeather
```

In a uv-managed project:

```bash
uv add getfeather
```

## Celery-style embedded mode (recommended)

```python
from getfeather import FeatherApp

app = FeatherApp()

async def send_email(ctx):
    ...

app.task("send-email", send_email)
await app.start_embedded()
app.delay("send-email", {"to": "user@example.com"})
```

See [Embedded mode](embedded.md).

## FeatherClient (enqueue only)

```python
from getfeather import FeatherClient

client = FeatherClient()  # uses FEATHER_ADDRESS env var

job_id = client.enqueue(
    "echo",
    queue="default",
    payload=b'{"message": "hello"}',
    priority=0,
)

job = client.get_job(job_id)
print(job["state"])

client.close()
```

### Environment

Set `FEATHER_ADDRESS` to override the default (`localhost:50051`).

## Worker

Async worker for dedicated processes:

```python
import asyncio
import json
from getfeather import Worker

async def echo(ctx):
    data = json.loads(ctx.payload.decode())
    print("payload:", data)

async def main():
    worker = Worker()
    worker.task("echo", echo)
    await worker.start()  # blocks until SIGINT/SIGTERM

asyncio.run(main())
```

### Features

- Async gRPC via `grpc.aio`
- Server-side blocking dequeue (Redis BRPOP) — no client poll loop
- Auto lease renewal at 50% TTL via `ExtendLease`
- Optional batch dequeue (`max_jobs`) with bounded concurrency
- Periodic heartbeat
- Graceful shutdown on SIGINT/SIGTERM

```python
worker = Worker(max_jobs=8, max_concurrency=4)
```

## Example

See `examples/python-worker/`:

```bash
cd examples/python-worker
python worker.py          # worker
python enqueue.py         # enqueue 10 jobs
```

Also see `examples/embedded-python/` for Celery-style embedded mode.
