#!/usr/bin/env python3
"""Python worker example."""

import asyncio
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "packages", "sdk-python"))

from getfeather import Worker


async def echo(ctx) -> None:
    body = json.loads(ctx.payload.decode() or "{}")
    print("echo payload", body)


async def main() -> None:
    worker = Worker()
    worker.task("echo", echo)
    print("worker starting...")
    await worker.start()


if __name__ == "__main__":
    asyncio.run(main())
