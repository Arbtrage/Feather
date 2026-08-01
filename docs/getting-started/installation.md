# Installation

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Docker & Docker Compose | Latest | Full local stack |
| Rust | 1.78+ | Server development |
| Node.js | 20+ | UI and docs site builds (not required for SDK users) |
| Python | 3.11+ | Python SDK and examples |
| Redis | 7+ | Required if running server outside Docker |

## Clone the repository

```bash
git clone https://github.com/Arbtrage/Feather.git
cd Feather
```

## Install published SDK

### Python — `getfeather`

```bash
pip install getfeather
uv pip install getfeather
pipx install getfeather
uv add getfeather   # inside a uv project
```

## Install from monorepo (development)

```bash
chmod +x scripts/bundle-protos.sh scripts/bundle-ui.sh
./scripts/bundle-protos.sh
./scripts/bundle-ui.sh
pip install ./packages/sdk-python
```

## Install server (optional, for local dev)

```bash
cd packages/server
cargo build --release
```

## Verify Redis

If not using Docker Compose, ensure Redis is running:

```bash
redis-cli ping
# PONG
```

Next: [Quickstart](quickstart.md)
