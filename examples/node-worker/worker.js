import { Worker } from "@feather/sdk";

const worker = new Worker({
  address: process.env.FEATHER_ADDRESS ?? "localhost:50051",
  workerId: process.env.WORKER_ID ?? `node-${process.pid}`,
  queues: ["default"],
});

worker.use(async (ctx, next) => {
  console.log("job", ctx.id, ctx.name);
  await next();
});

worker.task("echo", async (ctx) => {
  const body = JSON.parse(ctx.payload.toString() || "{}");
  console.log("echo payload", body);
});

console.log("worker starting...");
await worker.start();
