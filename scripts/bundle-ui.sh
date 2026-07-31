#!/usr/bin/env bash
# Build @arbitrage/ui and copy static assets into SDK packages.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
UI="$ROOT/packages/ui"
NODE_DST="$ROOT/packages/sdk-node/ui-static"
PY_DST="$ROOT/packages/sdk-python/feather/ui_static"

cd "$UI"
npm install
npm run build

rm -rf "$NODE_DST" "$PY_DST"
mkdir -p "$NODE_DST" "$PY_DST"
cp -R "$UI/dist/." "$NODE_DST/"
cp -R "$UI/dist/." "$PY_DST/"

echo "bundled UI to sdk-node and sdk-python"
