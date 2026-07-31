# Protocol

Feather uses Protocol Buffers (proto3) as the single source of truth for client-server communication. Package: `feather.v1`.

## Proto files

Located in `packages/proto/feather/v1/`:

| File | Contents |
|------|----------|
| `common.proto` | Shared types (`ResourceProfile`, `Failure`) |
| `job.proto` | `Job` message, `JobState` enum |
| `queue.proto` | `QueueService` and request/response messages |
| `worker.proto` | `WorkerService` and request/response messages |

## Job message

```protobuf
message Job {
  string id = 1;
  string queue = 2;
  string name = 3;
  bytes payload = 4;
  JobState state = 5;
  int32 priority = 6;
  int32 attempt = 7;
  google.protobuf.Timestamp lease_expires_at = 8;
  google.protobuf.Timestamp created_at = 9;
  string worker_id = 10;
  string workflow_run_id = 11;  // reserved, empty in Phase 1
  string activity_id = 12;      // reserved, empty in Phase 1
}
```

## JobState enum

| Value | Name | Description |
|-------|------|-------------|
| 0 | `JOB_STATE_UNSPECIFIED` | Default / unknown |
| 1 | `JOB_STATE_PENDING` | Waiting for worker |
| 2 | `JOB_STATE_LEASED` | Claimed by worker |
| 3 | `JOB_STATE_COMPLETED` | Successfully finished |
| 4 | `JOB_STATE_FAILED` | Terminal failure |

## Code generation

Proto definitions are compiled to:

| Language | Tool | Output |
|----------|------|--------|
| Rust | `tonic-build` | Compiled in `packages/server/build.rs` |
| Node.js | `@grpc/proto-loader` | Loaded at runtime from proto files |
| Python | `grpcio-tools` | Generated on first import to `feather/_gen/` |

Run codegen:

```bash
./scripts/proto-gen.sh
```

## Buf configuration

Linting and breaking-change detection via [Buf](https://buf.build):

```bash
cd packages/proto
buf lint
buf breaking --against '.git#branch=main'
```

## Versioning

Phase 1 uses a single proto package (`feather.v1`). Future phases will add new packages (e.g., `feather.v2`) following protobuf backward-compatibility rules. Reserved fields in job metadata prepare for workflow integration in Phase 4.
