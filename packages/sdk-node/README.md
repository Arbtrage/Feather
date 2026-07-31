# @arbitrage/sdk

Node.js SDK for [Feather](https://github.com/Arbtrage/Feather) — enqueue tasks, run embedded workers, and optionally serve the monitoring UI.

## Install

```bash
npm install @arbitrage/sdk
```

Requires a running Feather server (`FEATHER_ADDRESS`, default `localhost:50051`).

## Celery-style (embedded — recommended)

```javascript
import { FeatherApp } from "@arbitrage/sdk";

const app = new FeatherApp({
  ui: { enabled: true, port: 3001 },
});

app.task("send-email", async (ctx) => {
  const { to } = JSON.parse(ctx.payload.toString());
  await sendEmail(to);
});

await app.startEmbedded(); // worker + UI at http://127.0.0.1:3001
await app.delay("send-email", { to: "user@example.com" });
```

## Dedicated worker (optional)

```javascript
import { Worker } from "@arbitrage/sdk";

const worker = new Worker({ queues: ["default"] });
worker.task("send-email", handler);
await worker.start();
```

## Publish

```bash
./scripts/bundle-protos.sh
./scripts/bundle-ui.sh
cd packages/sdk-node && npm publish --access public
```

Docs: https://docs.feather.dev
