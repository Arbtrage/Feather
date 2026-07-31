#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/packages/proto"

if command -v buf >/dev/null 2>&1; then
  buf generate
  echo "buf generate complete"
else
  echo "buf not installed; Rust server uses tonic-build in packages/server/build.rs"
fi
