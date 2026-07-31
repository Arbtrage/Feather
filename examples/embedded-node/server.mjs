/**
 * Celery-style: one process — API + background worker.
 * No separate worker.js needed.
 */
import express from "express";
import { FeatherApp } from "@arbtrage/feather";

const app = new FeatherApp({
  ui: { enabled: true, port: 3001, adminUrl: "http://localhost:8080" },
});
const server = express();

app.task("echo", async (ctx) => {
  const body = JSON.parse(ctx.payload.toString() || "{}");
  console.log("echo:", body);
});

server.get("/enqueue", async (_req, res) => {
  const { jobId } = await app.delay("echo", { message: "hello from API" });
  res.json({ jobId });
});

const port = Number(process.env.PORT ?? 4000);

server.listen(port, async () => {
  await app.startEmbedded();
  console.log(`API on :${port} — GET /enqueue to submit a job`);
  console.log("Worker runs in-process (embedded mode)");
  console.log("Monitoring UI: http://127.0.0.1:3001");
});

process.on("SIGINT", () => {
  app.shutdown();
  process.exit(0);
});
