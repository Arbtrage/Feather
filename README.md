# Feather

A durable execution platform — Rust control plane, Redis-backed job queues, Node.js and Python worker SDKs.

Phase 1 delivers reliable **enqueue → lease → execute → ack** job processing over gRPC with at-least-once delivery.

## Quickstart

```bash
docker compose -f docker/docker-compose.yml up --build redis feather-server
```

```bash
cd examples/embedded-node && npm install
FEATHER_ADDRESS=localhost:50051 npm start
```

Embedded mode starts a worker and optional monitoring UI (`http://127.0.0.1:3001` when `ui.enabled`).

## Documentation

Public docs site: **https://docs.feather.dev** (Fumadocs on Vercel)

Source markdown in [`docs/`](docs/):

- [Quickstart](docs/getting-started/quickstart.md)
- [Embedded mode](docs/sdks/embedded.md) · [Embedded UI](docs/sdks/ui.md)
- [Node.js SDK](docs/sdks/node.md) · [Python SDK](docs/sdks/python.md)
- [Configuration](docs/reference/configuration.md)
- [Roadmap](docs/roadmap.md)
- [Vercel deploy guide](docs/operations/vercel.md)

## Repository layout

```
feather/
├── apps/docs/            Fumadocs site (deploy to Vercel)
├── apps/dashboard/       Optional standalone Next.js UI (Docker)
├── packages/
│   ├── proto/            feather.v1 protobuf definitions
│   ├── server/           Rust feather-server
│   ├── ui/               Monitoring SPA (bundled into SDKs)
│   ├── sdk-node/         @arbitrage/feather (npm)
│   └── sdk-python/       arbitrage-feather (PyPI)
├── docs/                 Documentation markdown source
├── examples/             Node & Python workers
├── docker/               Compose & Dockerfiles
└── tests/                Integration & contract tests
```

Internal design specs live locally in `docs-local/` (gitignored).

## Development

```bash
./scripts/ci-local.sh

# Docs site locally
cd apps/docs && npm install && npm run dev
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT — see [LICENSE](LICENSE).
