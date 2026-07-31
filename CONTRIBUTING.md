# Contributing to Feather

## Development setup

1. Install Rust 1.78+, Node 20+, Python 3.11+, Docker, `protoc`.
2. Start Redis: `docker compose -f docker/docker-compose.yml up redis -d`
3. Run server: `cargo run --manifest-path packages/server/Cargo.toml`
4. Build SDKs (bundles protos + UI):
   ```bash
   cd packages/sdk-node && npm install && npm run build
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
node --test tests/contract/node_contract.test.mjs
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

## Release SDKs

Actions → **Release** → **Run workflow** with a semver (e.g. `0.1.0`). CI creates the GitHub Release, publishes `@OWNER/sdk` to GitHub Packages, and attaches the Python wheel to the release.

No external registry secrets required. See [docs/operations/publishing.md](docs/operations/publishing.md).

Local dry-run: `./scripts/release-dry-run.sh 0.1.0`

## Monorepo structure

| Path | Purpose |
|------|---------|
| `packages/proto/` | Protobuf definitions |
| `packages/server/` | Rust server crate |
| `packages/ui/` | Monitoring SPA (bundled into SDKs) |
| `packages/sdk-node/` | Node.js SDK (`@arbitrage/sdk`) |
| `packages/sdk-python/` | Python SDK |
| `apps/docs/` | Fumadocs site (Vercel) |
| `apps/dashboard/` | Optional standalone dashboard (Docker) |
| `examples/` | Sample workers |
| `docker/` | Docker Compose stack |
