#!/usr/bin/env bash
# Cross-language e2e: Node enqueue → Python worker
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export FEATHER_ADDRESS="${FEATHER_ADDRESS:-localhost:50051}"

echo "Ensure feather-server and Redis are running (docker compose up)."
echo "Starting Python worker in background..."
python3 "$ROOT/examples/python-worker/worker.py" &
WORKER_PID=$!
trap 'kill $WORKER_PID 2>/dev/null || true' EXIT
sleep 2

echo "Enqueueing jobs via Node..."
cd "$ROOT/examples/node-worker"
npm install --silent 2>/dev/null || npm install
COUNT=3 npm run enqueue

echo "Waiting for worker..."
sleep 5
echo "Cross-language smoke complete."
