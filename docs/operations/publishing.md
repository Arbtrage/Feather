# Publishing SDKs

Feather publishes SDKs through **GitHub Releases** and **GitHub Packages** (free for public repositories). No npmjs.com or PyPI account is required.

| SDK | Where it lives |
|-----|----------------|
| Node.js `@arbtrage/feather` | [GitHub Packages](https://github.com/features/packages) (npm registry) |
| Python `feather-sdk` | `.whl` + `.tar.gz` attached to each [GitHub Release](https://docs.github.com/en/repositories/releasing-projects-on-github) |

GitHub Packages does not host a pip registry, so the Python wheel is uploaded as a release asset instead.

## Publish a release

1. Merge changes to `main`
2. GitHub → **Actions** → **Release** → **Run workflow**
3. Enter version (e.g. `0.1.0`) — do **not** include the `v` prefix
4. Leave **dry_run** unchecked to tag, publish, and create the release

The workflow will:

1. Sync version to `packages/sdk-node/package.json` and `packages/sdk-python/pyproject.toml`
2. Bundle protos, embedded UI static assets, and Python gRPC stubs
3. Run validation (build + contract tests + `twine check`)
4. Publish `@arbtrage/feather` to GitHub Packages
5. Create a GitHub Release tagged `v0.1.0` with Python wheel/sdist attached

No extra secrets are needed — the workflow uses the built-in `GITHUB_TOKEN`.

### npm scope note

GitHub Packages requires the npm scope to match your GitHub org/username. For repo `Arbtrage/Feather`, the published package is **`@arbtrage/feather`** (derived from the GitHub owner name). The workflow sets this automatically at publish time.

## Manual dry-run (local)

```bash
./scripts/release-dry-run.sh 0.1.0
```

## Manual CI dry-run

Actions → **Release** → **Run workflow** → enable **dry_run** (build + validate only).

## Install published packages

### Node.js (GitHub Packages)

Public packages can be installed without authentication:

```bash
npm install @arbtrage/feather@0.1.0
```

If npm cannot resolve the scope, add a project `.npmrc`:

```ini
@arbtrage:registry=https://npm.pkg.github.com
```

For private repos, use a [personal access token](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry) with `read:packages`.

### Python (GitHub Release asset)

Replace `v0.1.0` with your release tag:

```bash
pip install https://github.com/Arbtrage/Feather/releases/download/v0.1.0/feather_sdk-0.1.0-py3-none-any.whl
```

Or download the wheel from the release page and install locally:

```bash
pip install ./feather_sdk-0.1.0-py3-none-any.whl
```

## Troubleshooting

### npm publish fails: scope mismatch

The npm scope must match your GitHub owner (`Arbtrage` → `@arbtrage`). The release workflow sets this automatically; do not rename the scope manually unless you also rename the GitHub org.

### Python wheel missing UI assets

Confirm `scripts/bundle-ui.sh` ran — the wheel should include `arbtrage/feather/ui_static/`:

```bash
./scripts/release-dry-run.sh 0.1.0
unzip -l packages/sdk-python/dist/*.whl | grep ui_static
```

### Release tag already exists

Delete the tag in GitHub (Releases → delete, or `git push origin :refs/tags/v0.1.0`) before re-running the workflow for the same version.
