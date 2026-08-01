#!/usr/bin/env python3
"""Contract checks for the Python SDK package layout."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> None:
    pyproject = ROOT / "packages/sdk-python/pyproject.toml"
    text = pyproject.read_text()
    match = re.search(r'^name = "(.+)"$', text, re.M)
    assert match, "pyproject.toml missing name"
    assert match.group(1) == "getfeather", f"unexpected package name: {match.group(1)}"

    queue_proto = ROOT / "packages/proto/feather/v1/queue.proto"
    proto = queue_proto.read_text()
    assert "package feather.v1;" in proto
    assert "service QueueService" in proto

    keys = (ROOT / "packages/server/src/storage/keys.rs").read_text()
    assert "fe:" in keys
    assert "queue_registry" in keys

    ui_static = ROOT / "packages/sdk-python/getfeather/ui_static"
    assert ui_static.is_dir(), "ui_static bundle missing — run scripts/bundle-ui.sh"

    print("python contract checks passed")


if __name__ == "__main__":
    try:
        main()
    except AssertionError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        sys.exit(1)
