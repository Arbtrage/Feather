# Node.js SDK

Package: `@arbtrage/feather` on [GitHub Packages](https://github.com/Arbtrage/Feather/pkgs/npm/feather) (monorepo: `packages/sdk-node/`)

Requires Node.js 20+.

## Install

```bash
npm install @arbtrage/feather
```

If the scope is not resolved, add `.npmrc`:

```ini
@arbtrage:registry=https://npm.pkg.github.com
```

From monorepo during development:

```bash
cd packages/sdk-node && npm install && npm run build
```

## Celery-style embedded mode (recommended)

Run the worker **inside your app process** — no separate worker deployment:

```javascript
import { FeatherApp } from "@arbtrage/feather";

const app = new FeatherApp();

app.task("send-email", async (ctx) => {
  const { to } = JSON.parse(ctx.payload.toString());
  await sendEmail(to);
});

await app.startEmbedded(); // non-blocking background poll

// From your API route:
await app.delay("send-email", { to: "user@example.com" });
```

See [Embedded mode](embedded.md) for when to use embedded vs dedicated workers.

## FeatherClient (enqueue only)

Enqueue jobs and query status:

```javascript
import { FeatherClient } from "@arbtrage/feather";

const client = new FeatherClient("localhost:50051");

// Enqueue
const { jobId } = await client.enqueue({
  name: "echo",
  payload: JSON.stringify({ message: "hello" }),
  queue: "default",    // optional, defaults to "default"
  priority: 0,           // optional
});

// Get job
const job = await client.getJob(jobId);
console.log(job.state); // "pending", "leased", "completed", "failed"

client.close();
```

### Environment

Set `FEATHER_ADDRESS` to override the default server target (`localhost:50051`).

## Worker

Poll, execute, and acknowledge jobs:

```javascript
import { Worker } from "@arbtrage/feather";

const worker = new Worker({
  address: process.env.FEATHER_ADDRESS ?? "localhost:50051",
  workerId: `node-${process.pid}`,
  queues: ["default"],
});

// Middleware
worker.use(async (ctx, next) => {
  console.log(`job ${ctx.id} (${ctx.name})`);
  await next();
});

// Handler
worker.task("echo", async (ctx) => {
  const data = JSON.parse(ctx.payload.toString());
  console.log("payload:", data);
});

await worker.start(); // blocks until SIGINT
```

### Features

- Long-poll dequeue loop with adaptive backoff
- Auto lease renewal at 50% of TTL
- SIGINT graceful shutdown with deregistration
- Middleware pipeline (`use()`)

## Example

See `examples/node-worker/`:

```bash
cd examples/node-worker
npm install
FEATHER_ADDRESS=localhost:50051 npm start   # worker
npm run enqueue                                # enqueue 10 jobs
```

## Error handling

gRPC errors map to standard Node.js exceptions:

| gRPC Code | Meaning |
|-----------|---------|
| `NOT_FOUND` | Job or worker not found |
| `FAILED_PRECONDITION` | Wrong worker, job not leased, max renewals |
| `INVALID_ARGUMENT` | Bad queue name, oversized payload |
| `UNAVAILABLE` | Server or Redis unreachable |
