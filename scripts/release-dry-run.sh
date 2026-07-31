#!/usr/bin/env bash
# Local release dry-run (same steps as CI, no publish).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo "usage: release-dry-run.sh <version>" >&2
  echo "  example: release-dry-run.sh 0.1.0" >&2
  exit 1
fi

"$ROOT/scripts/sync-version.sh" "$VERSION"
pip install grpcio-tools build hatchling 2>/dev/null || true
"$ROOT/scripts/bundle-protos.sh"

cd "$ROOT/packages/sdk-node"
npm ci
npm run build
echo "✓ @feather/sdk $VERSION ready (npm pack --dry-run):"
npm pack --dry-run

cd "$ROOT/packages/sdk-python"
python3 -m build
echo "✓ feather-sdk $VERSION ready in dist/"
ls -la dist/

echo ""
echo "To publish via CI: create a GitHub Release with tag v$VERSION"
