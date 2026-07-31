# gRPC API

Feather exposes two gRPC services on port 50051 (default). Protocol package: `feather.v1`.

## QueueService

### Enqueue

Submit a new job to a queue.

```
rpc Enqueue(EnqueueRequest) returns (EnqueueResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `queue` | string | Target queue (default: `default`) |
| `name` | string | Task type for handler dispatch |
| `payload` | bytes | Opaque job data |
| `priority` | int32 | Priority (reserved, use 0) |

Returns: `job_id`, `created_at`

### Dequeue

Long-poll for the next available job.

```
rpc Dequeue(DequeueRequest) returns (DequeueResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `worker_id` | string | Calling worker ID |
| `queues` | string[] | Queues to poll |
| `wait_timeout_ms` | int32 | Max wait time (default 30,000) |
| `max_jobs` | int32 | Jobs per response (always 1 in Phase 1) |

Returns: `job` (or empty), `backoff_hint_ms`, `slow_down`

### Ack

Mark a leased job as completed.

```
rpc Ack(AckRequest) returns (AckResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `job_id` | string | Job to acknowledge |
| `worker_id` | string | Must match leasing worker |

### Nack

Mark a leased job as failed.

```
rpc Nack(NackRequest) returns (NackResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `job_id` | string | Job to nack |
| `worker_id` | string | Must match leasing worker |
| `reason` | string | Failure description |
| `retryable` | bool | Reserved for Phase 2 |
| `failure_class` | string | Reserved for Phase 2 DLQ |

### ExtendLease

Renew the lease on a job in progress.

```
rpc ExtendLease(ExtendLeaseRequest) returns (ExtendLeaseResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `job_id` | string | Job to extend |
| `worker_id` | string | Must match leasing worker |
| `extension_ms` | int32 | Extension duration (default 30,000) |

Returns: `lease_expires_at`

### GetJob

Retrieve job details by ID.

```
rpc GetJob(GetJobRequest) returns (GetJobResponse)
```

## WorkerService

### Register

Register a worker with the server.

```
rpc Register(RegisterRequest) returns (RegisterResponse)
```

| Field | Type | Description |
|-------|------|-------------|
| `worker_id` | string | Unique worker identifier |
| `queues` | string[] | Queues to serve |
| `capabilities` | string[] | Feature tags |
| `labels` | map | Key-value metadata |
| `resource_profile` | ResourceProfile | CPU/memory/GPU hints |
| `metadata` | map | Arbitrary metadata |

Returns: `lease_duration_ms`, `heartbeat_interval_ms`

### Heartbeat

Send a keep-alive signal.

```
rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse)
```

### Deregister

Remove a worker from the active set.

```
rpc Deregister(DeregisterRequest) returns (DeregisterResponse)
```

## Testing with grpcurl

```bash
# Enqueue
grpcurl -plaintext -d '{"queue":"default","name":"echo","payload":"aGVsbG8="}' \
  localhost:50051 feather.v1.QueueService/Enqueue

# Get job
grpcurl -plaintext -d '{"job_id":"<id>"}' \
  localhost:50051 feather.v1.QueueService/GetJob
```
