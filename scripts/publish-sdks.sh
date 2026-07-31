#!/usr/bin/env bash
# Local helper — CI publishes on GitHub Release. See docs/operations/publishing.md
set -euo pipefail
echo "SDK publishing is handled by .github/workflows/release.yml"
echo ""
echo "  1. Add secrets: NPM_TOKEN, PYPI_API_TOKEN"
echo "  2. Create GitHub Release with tag v0.1.0"
echo ""
echo "Local dry-run:"
echo "  ./scripts/release-dry-run.sh 0.1.0"
echo ""
echo "Manual CI trigger: Actions → Release → Run workflow"
