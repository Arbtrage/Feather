#!/usr/bin/env bash
# Set Python SDK package version from a git tag (e.g. v0.2.0 → 0.2.0).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: sync-version.sh <tag-or-version>" >&2
  echo "  examples: sync-version.sh v0.2.0  |  sync-version.sh 0.2.0" >&2
  exit 1
fi

RAW="$1"
VERSION="${RAW#v}"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
  echo "invalid semver: $VERSION (from $RAW)" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PY_PROJECT="$ROOT/packages/sdk-python/pyproject.toml"

python3 - <<PY
from pathlib import Path
import re

version = "$VERSION"
path = Path("$PY_PROJECT")
text = path.read_text()
text, n = re.subn(r'^version = ".*"$', f'version = "{version}"', text, count=1, flags=re.M)
if n != 1:
    raise SystemExit("failed to update pyproject.toml version")
path.write_text(text)
print(f"getfeather -> {version}")
PY

echo "version sync complete: $VERSION"
