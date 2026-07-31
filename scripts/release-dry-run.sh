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
  if [[ -x "$ROOT/.venv/bin/python" ]]; then
    PY="$ROOT/.venv/bin/python"
  elif [[ -x "$ROOT/.venv/bin/python3.12" ]]; then
    PY="$ROOT/.venv/bin/python3.12"
  else
    VENV="$ROOT/.venv-release-dry-run"
    if [[ ! -x "$VENV/bin/python" ]]; then
      rm -rf "$VENV"
      (python3.12 -m venv "$VENV" 2>/dev/null || python3 -m venv "$VENV")
    fi
    PY="$VENV/bin/python"
  fi
  "$PY" -m pip install "$@"
}

chmod +x scripts/bundle-protos.sh scripts/bundle-ui.sh scripts/sync-version.sh
./scripts/sync-version.sh "$VERSION"
ensure_python_tools grpcio-tools "build>=1.2" "hatchling>=1.27.0"
./scripts/bundle-protos.sh
./scripts/bundle-ui.sh
npm ci
npm run build -w @feather/sdk
test -d packages/sdk-node/ui-static/assets

ensure_python_tools "build>=1.2" "hatchling>=1.27.0" grpcio-tools "twine>=6.1.0"
./scripts/bundle-protos.sh
./scripts/bundle-ui.sh
test -d packages/sdk-python/feather/ui_static/assets
cd "$ROOT/packages/sdk-python"
"$PY" -m build
"$PY" -m twine check dist/*
cd "$ROOT"

node --test tests/contract/node_contract.test.mjs

echo ""
echo "Release validate passed for v$VERSION"
echo "Re-run CI: gh workflow run release.yml -f version=$VERSION -f dry_run=true"
