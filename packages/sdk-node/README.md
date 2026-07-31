# @feather/sdk

Node.js SDK for [Feather](https://github.com/your-org/feather) — enqueue tasks and run workers in-process (Celery-style) or as dedicated processes.

## Install

```bash
npm install @feather/sdk
```

Requires a running Feather server (`FEATHER_ADDRESS`, default `localhost:50051`).

## Celery-style (embedded — recommended)

Run tasks in the same process as your app — no separate worker deployment:

```javascript
import { FeatherApp } from "@feather/sdk";

const app = new FeatherApp();

app.task("send-email", async (ctx) => {
  const { to } = JSON.parse(ctx.payload.toString());
  await sendEmail(to);
});

// Start background polling (non-blocking)
await app.startEmbedded();

// Enqueue from your API
await app.delay("send-email", { to: "user@example.com" });
```

## Dedicated worker (optional)

For high throughput, run a separate worker process:

```javascript
import { Worker } from "@feather/sdk";

const worker = new Worker({ queues: ["default"] });
worker.task("send-email", handler);
await worker.start(); // blocks
```

## Publish

From monorepo root:

```bash
./scripts/bundle-protos.sh
cd packages/sdk-node && npm publish --access public
```
