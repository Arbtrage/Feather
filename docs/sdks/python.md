# Python SDK

Package: `feather-sdk` on PyPI (monorepo: `packages/sdk-python/`)

Requires Python 3.11+.

## Install

```bash
pip install feather-sdk
```

## Celery-style embedded mode (recommended)

```python
from feather import FeatherApp

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
from feather import FeatherClient

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

Async worker with the same surface area as the Node SDK:

```python
import asyncio
import json
from feather import Worker

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
- Long-poll dequeue with backoff
- Periodic heartbeat
- Graceful shutdown on SIGINT/SIGTERM

## Example

See `examples/python-worker/`:

```bash
cd examples/python-worker
python worker.py          # worker
python enqueue.py         # enqueue 10 jobs
```

## Cross-language

Python workers can process jobs enqueued by Node.js (and vice versa). The gRPC protocol and proto definitions are language-agnostic.

```bash
# Terminal 1: Python worker
python examples/python-worker/worker.py

# Terminal 2: Node enqueue
cd examples/node-worker && npm run enqueue
```
