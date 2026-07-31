# Publishing SDKs

Feather publishes **@arbitrage/sdk** (npm) and **arbitrage-feather-sdk** (PyPI) automatically when you create a GitHub Release.

## One-time setup

Add these secrets in GitHub → Settings → Secrets and variables → Actions:

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | npm automation token with publish access to `@arbitrage` scope |
| `PYPI_API_TOKEN` | PyPI API token for `arbitrage-feather-sdk` |

### npm organization (required for first publish)

The Node package is scoped as **`@arbitrage/sdk`**. npm returns `404 Not Found` on first publish if the **`@arbitrage` organization does not exist** or your token cannot publish to that scope.

1. Sign in at [npmjs.com](https://www.npmjs.com)
2. Create an organization named **`arbitrage`** (free plan is fine)
3. Confirm you are an owner of `@arbitrage`

Alternatively, rename the package to a scope you already own (for example `@your-npm-user/sdk`) before releasing.

### npm token

1. [npmjs.com](https://www.npmjs.com) → Access Tokens → Generate **Granular Access Token**
2. Organizations: read/write for **`arbitrage`**
3. Packages: read/write for `@arbitrage/sdk` (or all packages in the org)
4. Add as repository secret `NPM_TOKEN`

Verify locally (optional):

```bash
npm whoami
npm access ls-packages @arbitrage
```

### PyPI token

1. [pypi.org](https://pypi.org) → Account → API tokens → Add token
2. Scope: entire account or project `arbitrage-feather-sdk`
3. Add as repository secret `PYPI_API_TOKEN`

Optional: configure a GitHub **environment** named `pypi` with required reviewers for production publishes.

## Publish a release

1. Merge changes to `main`
2. GitHub → **Releases** → **Draft a new release**
3. Create tag `v0.1.0` (must match semver: `vMAJOR.MINOR.PATCH`)
4. Publish release

The [Release workflow](../.github/workflows/release.yml) will:

1. Sync version to `packages/sdk-node/package.json` and `packages/sdk-python/pyproject.toml`
2. Bundle protos, embedded UI static assets, and Python gRPC stubs
3. Run validation (build + contract tests + `twine check`)
4. Publish to npm and PyPI

## Troubleshooting

### npm: `404 Not Found` on `@arbitrage/sdk`

- Create the **`@arbitrage` npm organization** (see above)
- Regenerate `NPM_TOKEN` with org publish permissions
- Re-run the release workflow

### PyPI: `400 Bad Request`

- Ensure the workflow builds with `hatchling>=1.27.0` and uploads with `twine>=6.1.0` (license metadata must match Metadata-Version 2.4)
- Confirm `scripts/bundle-ui.sh` ran — the wheel should include `feather/ui_static/`
- Run locally: `./scripts/release-dry-run.sh 0.1.0` then inspect `packages/sdk-python/dist/*.whl`

## Manual dry-run (local)

```bash
./scripts/release-dry-run.sh 0.1.0
```

## Manual CI trigger

Actions → **Release** → **Run workflow**

- Enter version: `0.1.0`
- Enable **dry_run** to build without publishing

## Install published packages

```bash
npm install @arbitrage/sdk@0.1.0
pip install arbitrage-feather-sdk==0.1.0
```
