#!/usr/bin/env bash
# Copy feather.v1 protos into the Python SDK package for PyPI publishing.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/packages/proto/feather"
PY_DST="$ROOT/packages/sdk-python/getfeather/proto/feather"

rm -rf "$PY_DST"
mkdir -p "$PY_DST"
cp -R "$SRC/v1" "$PY_DST/"

echo "bundled protos to sdk-python"

# Pre-generate Python gRPC stubs for PyPI wheels
PY_SDK="$ROOT/packages/sdk-python"
GEN="$PY_SDK/getfeather/_gen"
rm -rf "$GEN"
mkdir -p "$GEN"

if python3 -c "import grpc_tools" 2>/dev/null; then
  python3 -m grpc_tools.protoc \
    -I"$PY_SDK/getfeather/proto" \
    --python_out="$GEN" \
    --grpc_python_out="$GEN" \
    "$PY_SDK/getfeather/proto/feather/v1/common.proto" \
    "$PY_SDK/getfeather/proto/feather/v1/job.proto" \
    "$PY_SDK/getfeather/proto/feather/v1/queue.proto" \
    "$PY_SDK/getfeather/proto/feather/v1/worker.proto"
  # Fix relative imports in generated grpc files
  find "$GEN" -name '*_pb2_grpc.py' -exec sed -i '' 's/import \(.*\)_pb2 as/from . import \1_pb2 as/g' {} + 2>/dev/null || \
  find "$GEN" -name '*_pb2_grpc.py' -exec sed -i 's/import \(.*\)_pb2 as/from . import \1_pb2 as/g' {} +
  touch "$GEN/feather/__init__.py" "$GEN/feather/v1/__init__.py"
  echo "generated Python gRPC stubs"
else
  echo "grpcio-tools not installed; Python stubs generated on first import"
fi
