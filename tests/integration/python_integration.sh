#!/usr/bin/env bash
# Python-only e2e: enqueue → worker smoke test
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export FEATHER_ADDRESS="${FEATHER_ADDRESS:-localhost:50051}"

echo "Ensure feather-server and Redis are running (docker compose up)."
echo "Starting Python worker in background..."
python3 "$ROOT/examples/python-worker/worker.py" &
WORKER_PID=$!
trap 'kill $WORKER_PID 2>/dev/null || true' EXIT
sleep 2

echo "Enqueueing jobs via Python..."
COUNT=3 python3 "$ROOT/examples/python-worker/enqueue.py"

echo "Waiting for worker..."
sleep 5
echo "Python integration smoke complete."
