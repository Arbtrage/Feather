#!/usr/bin/env bash
# Local release dry-run (mirrors .github/workflows/release.yml validate job).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo "usage: release-dry-run.sh <version>" >&2
  echo "  example: release-dry-run.sh 0.1.0" >&2
  exit 1
fi

cd "$ROOT"

if ! node -v | grep -qE '^v20\.'; then
  echo "warning: release CI uses Node 20; you are on $(node -v)" >&2
fi

ensure_python_tools() {
  if python3 -m pip install "$@" 2>/dev/null; then
    return 0
  fi
  VENV="$ROOT/.venv-release-dry-run"
  if [[ ! -d "$VENV" ]]; then
    python3 -m venv "$VENV"
  fi
  # shellcheck disable=SC1091
  source "$VENV/bin/activate"
  pip install "$@"
}

chmod +x scripts/bundle-protos.sh scripts/sync-version.sh
./scripts/sync-version.sh "$VERSION"
ensure_python_tools grpcio-tools build hatchling
./scripts/bundle-protos.sh
npm ci
npm run build -w @feather/sdk

ensure_python_tools build hatchling grpcio-tools
./scripts/bundle-protos.sh
cd "$ROOT/packages/sdk-python"
python3 -m build
cd "$ROOT"

node --test tests/contract/node_contract.test.mjs

echo ""
echo "Release validate passed for v$VERSION"
echo "Re-run CI: gh workflow run release.yml -f version=$VERSION -f dry_run=true"
