# Publishing SDKs

Feather publishes **@arbitrage/feather** (npm) and **arbitrage-feather** (PyPI).

## One-time setup

Add these secrets in GitHub → Settings → Secrets and variables → Actions:

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | npm automation token with publish access to `@arbitrage` |
| `PYPI_API_TOKEN` | PyPI API token for `arbitrage-feather` |

### npm organization (required)

The Node package is **`@arbitrage/feather`**. npm returns `404 Not Found` on first publish if the **`@arbitrage` organization does not exist**.

1. Sign in at [npmjs.com](https://www.npmjs.com)
2. Create an organization named **`arbitrage`**
3. Generate a **Granular Access Token** with read/write for the `arbitrage` org
4. Add as repository secret `NPM_TOKEN`

Verify:

```bash
npm whoami
npm access ls-packages @arbitrage
```

### PyPI token

1. [pypi.org](https://pypi.org) → Account → API tokens
2. Scope: project `arbitrage-feather` (or entire account for first publish)
3. Add as repository secret `PYPI_API_TOKEN`

Optional: configure a GitHub **environment** named `pypi` with required reviewers.

## Publish a release

1. Merge changes to `main`
2. GitHub → **Actions** → **Release** → **Run workflow**
3. Enter version (e.g. `0.1.2`) — no `v` prefix
4. Leave **dry_run** unchecked

The workflow will:

1. Sync version across SDK manifests
2. Build and validate both SDKs
3. Publish to npm and PyPI
4. Create a GitHub Release tag for changelog

## Install

```bash
npm install @arbitrage/feather
pip install arbitrage-feather
```

### Node.js

```javascript
import { FeatherApp } from "@arbitrage/feather";
```

### Python

```python
from arbitrage.feather import FeatherApp
```

No `.npmrc` or GitHub token required — packages live on the public registries.

## Manual dry-run

```bash
./scripts/release-dry-run.sh 0.1.2
```

## Troubleshooting

### npm: `404 Not Found`

- Create the **`@arbitrage`** npm organization
- Regenerate `NPM_TOKEN` with org publish permissions
- Re-run the release workflow with a new version

### PyPI: `400 Bad Request`

- Use `hatchling>=1.27.0` and `twine>=6.1.0` (handled in CI)
- Confirm `scripts/bundle-ui.sh` ran — wheel must include `arbitrage/feather/ui_static/`

### Version already published

Bump the version — npm and PyPI do not allow republishing the same version.
