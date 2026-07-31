# Feather

A durable execution platform — Rust control plane, Redis-backed job queues, Node.js and Python worker SDKs.

Phase 1 delivers reliable **enqueue → lease → execute → ack** job processing over gRPC with at-least-once delivery.

## Quickstart

```bash
docker compose -f docker/docker-compose.yml up --build
```

```bash
cd examples/node-worker && npm install
FEATHER_ADDRESS=localhost:50051 npm start    # worker
npm run enqueue                               # enqueue jobs
```

Open the dashboard at [http://localhost:3000](http://localhost:3000).

## Documentation

Full documentation lives in [`docs/`](docs/):

- [Quickstart](docs/getting-started/quickstart.md)
- [Concepts](docs/concepts/jobs-and-queues.md)
- [Node.js SDK](docs/sdks/node.md) · [Python SDK](docs/sdks/python.md)
- [Configuration](docs/reference/configuration.md)
- [Roadmap](docs/roadmap.md)

Compatible with [GitBook Git Sync](https://gitbook.com/docs/getting-started/git-sync) via [`.gitbook.yaml`](.gitbook.yaml).

## Repository layout

```
feather/
├── apps/dashboard/       Next.js read-only UI
├── packages/
│   ├── proto/            feather.v1 protobuf definitions
│   ├── server/           Rust feather-server
│   ├── sdk-node/         @feather/sdk
│   └── sdk-python/       feather-sdk
├── docs/                 Public documentation (GitBook)
├── examples/             Node & Python workers
├── docker/               Compose & Dockerfiles
└── tests/                Integration & contract tests
```

Internal design specs live locally in `docs-local/` (gitignored).

## Development

```bash
# Server (requires Redis)
cargo run --manifest-path packages/server/Cargo.toml

# SDKs
cd packages/sdk-node && npm install && npm run build
pip install ./packages/sdk-python

# Tests
FEATHER_INTEGRATION=1 cargo test --manifest-path packages/server/Cargo.toml
node --test tests/contract/node_contract.test.mjs
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT — see [LICENSE](LICENSE).
