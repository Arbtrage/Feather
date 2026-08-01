# Contributing to Feather

## Development setup

1. Install Rust 1.78+, Node 20+ (for UI/docs builds), Python 3.11+, Docker, `protoc`.
2. Start Redis: `docker compose -f docker/docker-compose.yml up redis -d`
3. Run server: `cargo run --manifest-path packages/server/Cargo.toml`
4. Build Python SDK (bundles protos + UI):
   ```bash
   chmod +x scripts/bundle-protos.sh scripts/bundle-ui.sh
   ./scripts/bundle-protos.sh
   ./scripts/bundle-ui.sh
   pip install ./packages/sdk-python
   ```

## Tests

Run the full CI mirror locally:

```bash
./scripts/ci-local.sh
```

Or individual targets:

```bash
FEATHER_INTEGRATION=1 cargo test --manifest-path packages/server/Cargo.toml
python tests/contract/python_contract.py
cd apps/docs && npm run build
cd packages/ui && npm run build
```

## Documentation

- **Public docs** — markdown in `docs/`, rendered by `apps/docs` (Fumadocs)
- **Sidebar** — edit `meta.json` in each `docs/` section folder
- **Internal design specs** — local `docs-local/` (gitignored)
- **Publishing** — [docs/operations/publishing.md](docs/operations/publishing.md)
- **Vercel** — [docs/operations/vercel.md](docs/operations/vercel.md)

When adding user-facing features, update `docs/` and the relevant `meta.json`.

## Release SDK

Actions → **Release** → **Run workflow** with a semver (e.g. `0.2.0`). CI publishes `getfeather` to PyPI.

Required secret: `PYPI_API_TOKEN`. See [docs/operations/publishing.md](docs/operations/publishing.md).

Local dry-run: `./scripts/release-dry-run.sh 0.2.0`

## Monorepo structure

| Path | Purpose |
|------|---------|
| `packages/proto/` | Protobuf definitions |
| `packages/server/` | Rust server crate |
| `packages/ui/` | Monitoring SPA (bundled into Python SDK) |
| `packages/sdk-python/` | Python SDK (`getfeather`) |
| `apps/docs/` | Fumadocs site (Vercel) |
| `apps/dashboard/` | Optional standalone dashboard (Docker) |
| `examples/` | Sample workers |
| `docker/` | Docker Compose stack |
