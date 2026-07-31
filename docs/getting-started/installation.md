# Installation

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Docker & Docker Compose | Latest | Full local stack |
| Rust | 1.78+ | Server development |
| Node.js | 20+ | Node SDK and examples |
| Python | 3.11+ | Python SDK and examples |
| Redis | 7+ | Required if running server outside Docker |

## Clone the repository

```bash
git clone https://github.com/your-org/feather.git
cd feather
```

## Install SDK dependencies

### Node.js

```bash
cd packages/sdk-node
npm install
npm run build
```

### Python

```bash
pip install ./packages/sdk-python
```

Or with uv:

```bash
uv pip install -e packages/sdk-python
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
