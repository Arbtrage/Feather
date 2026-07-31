# Contributing to Feather

## Development setup

1. Install Rust 1.78+, Node 20+, Python 3.11+, Docker.
2. Start Redis: `docker compose -f docker/docker-compose.yml up redis -d`
3. Run server: `cargo run --manifest-path packages/server/Cargo.toml`
4. Build SDKs:
   ```bash
   cd packages/sdk-node && npm install && npm run build
   pip install ./packages/sdk-python
   ```

## Tests

```bash
FEATHER_INTEGRATION=1 cargo test --manifest-path packages/server/Cargo.toml
node --test tests/contract/node_contract.test.mjs
cd apps/dashboard && npm run build
```

## Documentation

- **Public docs** — committed in `docs/`, GitBook-compatible (see `.gitbook.yaml`)
- **Internal design specs** — local `docs-local/` (gitignored)
- **Publishing** — [docs/operations/publishing.md](docs/operations/publishing.md)

When adding Phase 1 user-facing features, update `docs/` and `docs/SUMMARY.md`.

## Release SDKs

Create a GitHub Release with tag `v0.1.0` → CI publishes `@feather/sdk` and `feather-sdk`.

Required secrets: `NPM_TOKEN`, `PYPI_API_TOKEN`. See [docs/operations/publishing.md](docs/operations/publishing.md).

Local dry-run: `./scripts/release-dry-run.sh 0.1.0`

## Monorepo structure

| Path | Purpose |
|------|---------|
| `packages/proto/` | Protobuf definitions |
| `packages/server/` | Rust server crate |
| `packages/sdk-node/` | Node.js SDK (`@feather/sdk`) |
| `packages/sdk-python/` | Python SDK |
| `apps/dashboard/` | Next.js dashboard |
| `examples/` | Sample workers |
| `docker/` | Docker Compose stack |
