# Publishing SDKs

Feather publishes **@feather/sdk** (npm) and **feather-sdk** (PyPI) automatically when you create a GitHub Release.

## One-time setup

Add these secrets in GitHub → Settings → Secrets and variables → Actions:

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | npm automation token with publish access to `@feather` scope |
| `PYPI_API_TOKEN` | PyPI API token for `feather-sdk` |

### npm token

1. [npmjs.com](https://www.npmjs.com) → Access Tokens → Generate **Granular Access Token**
2. Packages: read/write for `@feather/sdk`
3. Add as repository secret `NPM_TOKEN`

### PyPI token

1. [pypi.org](https://pypi.org) → Account → API tokens → Add token
2. Scope: entire account or project `feather-sdk`
3. Add as repository secret `PYPI_API_TOKEN`

Optional: configure a GitHub **environment** named `pypi` with required reviewers for production publishes.

## Publish a release

1. Merge changes to `main`
2. GitHub → **Releases** → **Draft a new release**
3. Create tag `v0.1.0` (must match semver: `vMAJOR.MINOR.PATCH`)
4. Publish release

The [Release workflow](../.github/workflows/release.yml) will:

1. Sync version to `packages/sdk-node/package.json` and `packages/sdk-python/pyproject.toml`
2. Bundle protos and generate Python gRPC stubs
3. Run validation (build + contract tests)
4. Publish to npm and PyPI

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
npm install @feather/sdk@0.1.0
pip install feather-sdk==0.1.0
```
