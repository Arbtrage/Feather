#!/usr/bin/env bash
# Copy feather.v1 protos into SDK packages for registry publishing.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/packages/proto/feather"
NODE_DST="$ROOT/packages/sdk-node/proto/feather"
PY_DST="$ROOT/packages/sdk-python/arbtrage/feather/proto/feather"

rm -rf "$NODE_DST" "$PY_DST"
mkdir -p "$NODE_DST" "$PY_DST"
cp -R "$SRC/v1" "$NODE_DST/"
cp -R "$SRC/v1" "$PY_DST/"

echo "bundled protos to sdk-node and sdk-python"

# Pre-generate Python gRPC stubs for PyPI wheels
PY_SDK="$ROOT/packages/sdk-python"
GEN="$PY_SDK/arbtrage/feather/_gen"
rm -rf "$GEN"
mkdir -p "$GEN"

if python3 -c "import grpc_tools" 2>/dev/null; then
  python3 -m grpc_tools.protoc \
    -I"$PY_SDK/arbtrage/feather/proto" \
    --python_out="$GEN" \
    --grpc_python_out="$GEN" \
    "$PY_SDK/arbtrage/feather/proto/feather/v1/common.proto" \
    "$PY_SDK/arbtrage/feather/proto/feather/v1/job.proto" \
    "$PY_SDK/arbtrage/feather/proto/feather/v1/queue.proto" \
    "$PY_SDK/arbtrage/feather/proto/feather/v1/worker.proto"
  # Fix relative imports in generated grpc files
  find "$GEN" -name '*_pb2_grpc.py' -exec sed -i '' 's/import \(.*\)_pb2 as/from . import \1_pb2 as/g' {} + 2>/dev/null || \
  find "$GEN" -name '*_pb2_grpc.py' -exec sed -i 's/import \(.*\)_pb2 as/from . import \1_pb2 as/g' {} +
  touch "$GEN/feather/__init__.py" "$GEN/feather/v1/__init__.py"
  echo "generated Python gRPC stubs"
else
  echo "grpcio-tools not installed; Python stubs generated on first import"
fi
