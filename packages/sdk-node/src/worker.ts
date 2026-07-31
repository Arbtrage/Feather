import * as grpc from "@grpc/grpc-js";
import { createChannel, loadProto, promisify } from "./grpc.js";

export type JobContext = {
  id: string;
  queue: string;
  name: string;
  payload: Buffer;
  ack: () => Promise<void>;
  nack: (reason?: string) => Promise<void>;
};

export type Handler = (ctx: JobContext) => Promise<void>;

export type WorkerOptions = {
  address?: string;
  workerId?: string;
  queues?: string[];
  pollIntervalMs?: number;
  leaseRenewalRatio?: number;
};

type Middleware = (ctx: JobContext, next: () => Promise<void>) => Promise<void>;

export class Worker {
  private workerId: string;
  private queues: string[];
  private pollIntervalMs: number;
  private leaseRenewalRatio: number;
  private queueClient: any;
  private workerClient: any;
  private channel: grpc.Client;
  private handlers = new Map<string, Handler>();
  private middleware: Middleware[] = [];
  private running = false;
  private heartbeatTimer?: ReturnType<typeof setInterval>;
  private heartbeatIntervalMs = 10_000;
  private leaseDurationMs = 30_000;
  private registered = false;

  constructor(opts: WorkerOptions = {}) {
    const address = opts.address ?? process.env.FEATHER_ADDRESS ?? "localhost:50051";
    this.workerId = opts.workerId ?? `node-${process.pid}`;
    this.queues = opts.queues ?? ["default"];
    this.pollIntervalMs = opts.pollIntervalMs ?? 500;
    this.leaseRenewalRatio = opts.leaseRenewalRatio ?? 0.5;

    const { queue, worker } = loadProto();
    this.channel = createChannel(address);
    this.queueClient = new queue(this.channel, grpc.credentials.createInsecure());
    this.workerClient = new worker(this.channel, grpc.credentials.createInsecure());
  }

  task(name: string, handler: Handler): this {
    this.handlers.set(name, handler);
    return this;
  }

  use(mw: Middleware): this {
    this.middleware.push(mw);
    return this;
  }

  /** Block until stopped — use for dedicated worker processes. */
  async start(): Promise<void> {
    await this.register();
    this.running = true;
    process.on("SIGINT", () => this.stop());
    while (this.running) {
      try {
        await this.pollOnce();
      } catch {
        await sleep(this.pollIntervalMs * 2);
      }
    }
  }

  /**
   * Celery-style embedded mode: poll in the background without blocking your app.
   * Returns a stop function. Safe to call from Express/Fastify startup.
   */
  async startBackground(): Promise<() => void> {
    await this.register();
    this.running = true;
    const loop = (async () => {
      while (this.running) {
        try {
          await this.pollOnce();
        } catch {
          await sleep(this.pollIntervalMs * 2);
        }
      }
    })();
    loop.catch(() => {});
    process.on("SIGINT", () => this.stop());
    return () => this.stop();
  }

  stop(): void {
    this.running = false;
    if (this.heartbeatTimer) clearInterval(this.heartbeatTimer);
    if (this.registered) {
      const deregister = promisify<any, any>(this.workerClient.deregister.bind(this.workerClient));
      deregister({ workerId: this.workerId }).catch(() => {});
      this.registered = false;
    }
    this.queueClient.close?.();
    this.workerClient.close?.();
    this.channel.close();
  }

  private async register(): Promise<void> {
    if (this.registered) return;
    const register = promisify<any, any>(this.workerClient.register.bind(this.workerClient));
    const reg = await register({
      workerId: this.workerId,
      queues: this.queues,
      capabilities: [],
      labels: {},
      metadata: { mode: "embedded" },
    });
    this.leaseDurationMs = reg.leaseDurationMs ?? 30_000;
    this.heartbeatIntervalMs = reg.heartbeatIntervalMs ?? 10_000;

    const heartbeat = promisify<any, any>(this.workerClient.heartbeat.bind(this.workerClient));
    this.heartbeatTimer = setInterval(() => {
      heartbeat({ workerId: this.workerId, activeJobIds: [], status: "WORKER_STATUS_ACTIVE" }).catch(
        () => {}
      );
    }, this.heartbeatIntervalMs);
    this.registered = true;
  }

  private async pollOnce(): Promise<void> {
    const dequeue = promisify<any, any>(this.queueClient.dequeue.bind(this.queueClient));
    const res = await dequeue({
      workerId: this.workerId,
      queues: this.queues,
      waitTimeoutMs: 30_000,
      maxJobs: 1,
    });
    if (!res.job) {
      const backoff = res.backoffHintMs ?? this.pollIntervalMs;
      await sleep(backoff);
      return;
    }

    const job = res.job;
    const ackFn = promisify<any, any>(this.queueClient.ack.bind(this.queueClient));
    const nackFn = promisify<any, any>(this.queueClient.nack.bind(this.queueClient));
    const extendFn = promisify<any, any>(this.queueClient.extendLease.bind(this.queueClient));

    const renewMs = Math.floor(this.leaseDurationMs * this.leaseRenewalRatio);
    const renewTimer = setInterval(() => {
      extendFn({
        jobId: job.id,
        workerId: this.workerId,
        extensionMs: this.leaseDurationMs,
      }).catch(() => {});
    }, renewMs);

    const ctx: JobContext = {
      id: job.id,
      queue: job.queue,
      name: job.name,
      payload: Buffer.from(job.payload ?? []),
      ack: () => ackFn({ jobId: job.id, workerId: this.workerId }),
      nack: (reason = "error") =>
        nackFn({ jobId: job.id, workerId: this.workerId, reason, retryable: false, failureClass: "" }),
    };

    const handler = this.handlers.get(job.name);
    if (!handler) {
      await ctx.nack(`no handler for ${job.name}`);
      clearInterval(renewTimer);
      return;
    }

    const invoke = this.middleware.reduceRight<() => Promise<void>>(
      (next, mw) => () => mw(ctx, next),
      () => handler(ctx)
    );

    try {
      await invoke();
      await ctx.ack();
    } catch (err) {
      await ctx.nack(err instanceof Error ? err.message : "handler failed");
    } finally {
      clearInterval(renewTimer);
    }
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}
