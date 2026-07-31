# Docker Compose

The local development stack is defined in `docker/docker-compose.yml`.

## Services

### redis

- Image: `redis:7-alpine`
- Port: `6379`
- Persistence: AOF enabled
- Policy: `maxmemory-policy noeviction`

### feather-server

- Ports: `50051` (gRPC), `8080` (HTTP admin)
- Depends on: Redis (health check)
- Built from: `docker/Dockerfile.server`

### dashboard

- Port: `3000`
- Depends on: feather-server
- Built from: `docker/Dockerfile.dashboard`
- Env: `FEATHER_API_URL=http://feather-server:8080`

## Commands

```bash
# Start all services
docker compose -f docker/docker-compose.yml up --build

# Start Redis only (for local server dev)
docker compose -f docker/docker-compose.yml up redis -d

# Stop and remove volumes
docker compose -f docker/docker-compose.yml down -v
```

## Environment variables

Server environment inside Compose:

| Variable | Value in Compose |
|----------|------------------|
| `FEATHER_REDIS_URL` | `redis://redis:6379` |
| `FEATHER_GRPC_ADDR` | `0.0.0.0:50051` |
| `FEATHER_HTTP_ADDR` | `0.0.0.0:8080` |

See [Configuration](../reference/configuration.md) for the full list.
