#!/usr/bin/env bash
# Mirror .github/workflows/ci.yml locally before pushing.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== rust: fmt ==="
cargo fmt --check --manifest-path packages/server/Cargo.toml

echo "=== rust: clippy ==="
cargo clippy --manifest-path packages/server/Cargo.toml -- -D warnings

echo "=== rust: test ==="
if redis-cli -u "${FEATHER_REDIS_URL:-redis://127.0.0.1:6379}" ping 2>/dev/null | grep -q PONG; then
  FEATHER_INTEGRATION=1 FEATHER_REDIS_URL="${FEATHER_REDIS_URL:-redis://127.0.0.1:6379}" \
    cargo test --manifest-path packages/server/Cargo.toml
else
  echo "Redis not reachable — compiling tests only (CI runs integration tests with Redis)."
  cargo test --manifest-path packages/server/Cargo.toml --no-run
fi

echo "=== ui package ==="
cd packages/ui && npm install && npm run build
cd "$ROOT"
chmod +x scripts/bundle-ui.sh
./scripts/bundle-ui.sh

echo "=== node sdk ==="
cd packages/sdk-node && npm install && npm run build
cd "$ROOT"
node --test tests/contract/node_contract.test.mjs

echo "=== python sdk ==="
pip install grpcio-tools
chmod +x scripts/bundle-protos.sh
./scripts/bundle-protos.sh
./scripts/bundle-ui.sh
pip install ./packages/sdk-python
python -c "from arbitrage.feather import FeatherApp, FeatherClient, Worker"

echo "=== docs site ==="
cd apps/docs && npm install && npm run build
cd "$ROOT"

echo "=== dashboard (optional standalone) ==="
cd apps/dashboard && npm install && npm run build

echo ""
echo "All CI checks passed."
