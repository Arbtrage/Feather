#!/usr/bin/env python3
"""Enqueue sample jobs."""

import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "packages", "sdk-python"))

from getfeather import FeatherClient


def main() -> None:
    client = FeatherClient()
    count = int(os.environ.get("COUNT", "10"))
    for i in range(count):
        job_id = client.enqueue(
            "echo",
            payload=json.dumps({"n": i + 1}).encode(),
        )
        print("enqueued", job_id)
    client.close()


if __name__ == "__main__":
    main()
