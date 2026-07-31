# Configuration

Feather is configured via TOML file and environment variables. Environment variables take precedence.

## Config file

Default location: `packages/server/config/default.toml`

```toml
default_lease_duration_ms = 30000
max_payload_bytes = 262144
lease_sweep_interval_ms = 1000
recent_history_limit = 10000
heartbeat_interval_ms = 10000
offline_threshold_ms = 30000
max_lease_renewals = 100

[server]
grpc_addr = "0.0.0.0:50051"
http_addr = "0.0.0.0:8080"

[storage]
redis_url = "redis://127.0.0.1:6379"
namespace = "default"

[observability]
log_level = "info"
log_format = "json"
```

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FEATHER_GRPC_ADDR` | `0.0.0.0:50051` | gRPC listen address |
| `FEATHER_HTTP_ADDR` | `0.0.0.0:8080` | Admin HTTP listen address |
| `FEATHER_REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `FEATHER_NAMESPACE` | `default` | Redis key prefix namespace |
| `FEATHER_LEASE_MS` | `30000` | Default lease duration (ms) |
| `FEATHER_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
| `FEATHER_CORS_ORIGINS` | `http://localhost:3000,http://localhost:3001` | Comma-separated browser origins for admin API CORS |
| `FEATHER_ADMIN_URL` | `http://localhost:8080` | Admin API URL (SDK embedded UI) |
| `FEATHER_ADDRESS` | `localhost:50051` | SDK server target (client-side) |

## Server settings

| Setting | Default | Description |
|---------|---------|-------------|
| `default_lease_duration_ms` | 30000 | Lease TTL for dequeued jobs |
| `max_payload_bytes` | 262144 | Maximum job payload size (256 KB) |
| `lease_sweep_interval_ms` | 1000 | How often the lease sweeper runs |
| `recent_history_limit` | 10000 | Max jobs in recent index |
| `heartbeat_interval_ms` | 10000 | Expected worker heartbeat interval |
| `offline_threshold_ms` | 30000 | Worker considered offline after |
| `max_lease_renewals` | 100 | Max lease extensions per job |

## Redis key schema

All keys use the prefix `fe:{namespace}:`:

| Key pattern | Type | Purpose |
|-------------|------|---------|
| `fe:{ns}:queue:{name}:pending` | LIST | Pending job IDs |
| `fe:{ns}:queue:{name}:leased` | ZSET | Leased jobs (score = expiry ms) |
| `fe:{ns}:job:{id}` | HASH | Job data |
| `fe:{ns}:job:{id}:events` | LIST | State transition log |
| `fe:{ns}:index:jobs:recent` | ZSET | Recent jobs index |
| `fe:{ns}:workers:{id}` | HASH | Worker registration |
| `fe:{ns}:workers:active` | SET | Active worker IDs |
