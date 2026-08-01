# getfeather

Python SDK for [Feather](https://github.com/Arbtrage/Feather) — enqueue tasks and run workers in-process (Celery-style) or as dedicated processes.

## Install

```bash
pip install getfeather
uv pip install getfeather
pipx install getfeather
```

Requires a running Feather server (`FEATHER_ADDRESS`, default `localhost:50051`).

## Celery-style (embedded — recommended)

```python
import asyncio
from getfeather import FeatherApp

app = FeatherApp()

async def send_email(ctx):
    data = json.loads(ctx.payload.decode())
    await mailer.send(data["to"])

app.task("send-email", send_email)

async def main():
    await app.start_embedded()
    app.delay("send-email", {"to": "user@example.com"})
    await asyncio.sleep(3600)  # keep app running

asyncio.run(main())
```

## FastAPI integration

```python
from contextlib import asynccontextmanager
from fastapi import FastAPI
from getfeather import FeatherApp

feather = FeatherApp()

@asynccontextmanager
async def lifespan(app: FastAPI):
    await feather.start_embedded()
    yield
    await feather.shutdown()

app = FastAPI(lifespan=lifespan)

@app.post("/send")
async def send(to: str):
    feather.delay("send-email", {"to": to})
    return {"ok": True}
```

## Publish

```bash
./scripts/bundle-protos.sh
cd packages/sdk-python && python -m build && twine upload dist/*
```
