#!/usr/bin/env bash
# Build @arbitrage/ui and copy static assets into the Python SDK package.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UI="$ROOT/packages/ui"
PY_DST="$ROOT/packages/sdk-python/getfeather/ui_static"

cd "$UI"
npm install
npm run build

rm -rf "$PY_DST"
mkdir -p "$PY_DST"
cp -R "$UI/dist/." "$PY_DST/"

echo "bundled UI to sdk-python"
