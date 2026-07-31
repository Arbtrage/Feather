#!/usr/bin/env python3
"""Celery-style embedded worker + enqueue in one process."""

import asyncio
import json

from feather import FeatherApp

app = FeatherApp()


async def echo(ctx) -> None:
    body = json.loads(ctx.payload.decode() or "{}")
    print("echo:", body)


async def main() -> None:
    app.task("echo", echo)
    await app.start_embedded()
    print("embedded worker running — enqueueing sample job")
    job_id = app.delay("echo", {"from": "embedded-python"})
    print("enqueued", job_id)
    await asyncio.sleep(5)
    await app.shutdown()


if __name__ == "__main__":
    asyncio.run(main())
