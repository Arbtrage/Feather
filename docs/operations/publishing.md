# Publishing the Python SDK

Feather publishes **getfeather** to PyPI.

## One-time setup

Add this secret in GitHub → Settings → Secrets and variables → Actions:

| Secret | Purpose |
|--------|---------|
| `PYPI_API_TOKEN` | PyPI API token for `getfeather` |

### PyPI token

1. [pypi.org](https://pypi.org) → Account → API tokens
2. Scope: project `getfeather` (or entire account for first publish)
3. Add as repository secret `PYPI_API_TOKEN`

Optional: configure a GitHub **environment** named `pypi` with required reviewers.

## Publish a release

1. Merge changes to `main`
2. GitHub → **Actions** → **Release** → **Run workflow**
3. Enter version (e.g. `0.2.0`) — no `v` prefix
4. Leave **dry_run** unchecked

The workflow will:

1. Sync version in `packages/sdk-python/pyproject.toml`
2. Bundle protos and UI into the Python wheel
3. Publish to PyPI
4. Create a GitHub Release tag for changelog

## Install

```bash
pip install getfeather
uv pip install getfeather
pipx install getfeather
```

In a [uv](https://docs.astral.sh/uv/) project:

```bash
uv add getfeather
```

Pin a version:

```bash
pip install getfeather==0.2.0
uv pip install getfeather==0.2.0
pipx install getfeather==0.2.0
uv add getfeather==0.2.0
```

```python
from getfeather import FeatherApp
```

## Manual dry-run

```bash
./scripts/release-dry-run.sh 0.2.0
```

## Troubleshooting

### PyPI: `400 Bad Request`

- Use `hatchling>=1.27.0` and `twine>=6.1.0` (handled in CI)
- Confirm `scripts/bundle-ui.sh` ran — wheel must include `getfeather/ui_static/`

### Version already published

Bump the version — PyPI does not allow republishing the same version.

## Breaking changes in v0.2.0

- The Node.js SDK was removed — Feather is Python-only.
- PyPI package renamed from `arbitrage-feather` to **`getfeather`**.

```bash
pip install getfeather
```

```python
from getfeather import FeatherApp
```
