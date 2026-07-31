import { FeatherClient, type EnqueueOptions } from "./client.js";
import { Worker, type Handler, type WorkerOptions } from "./worker.js";

export type FeatherAppOptions = {
  address?: string;
  queue?: string;
  worker?: WorkerOptions;
};

/**
 * Celery-style app: define tasks, enqueue with .delay(), run worker in-process
 * via startEmbedded() — no separate worker deployment required.
 */
export class FeatherApp {
  private address: string;
  private client: FeatherClient;
  private defaultQueue: string;
  private workerOpts: WorkerOptions;
  private handlers = new Map<string, Handler>();
  private worker?: Worker;
  private stopWorker?: () => void;

  constructor(opts: FeatherAppOptions = {}) {
    this.address = opts.address ?? process.env.FEATHER_ADDRESS ?? "localhost:50051";
    this.client = new FeatherClient(this.address);
    this.defaultQueue = opts.queue ?? "default";
    this.workerOpts = opts.worker ?? {};
  }

  /** Register a task handler (Celery-style task definition). */
  task(name: string, handler: Handler): this {
    this.handlers.set(name, handler);
    return this;
  }

  /** Enqueue a task by name. */
  async enqueue(
    name: string,
    payload?: EnqueueOptions["payload"],
    opts?: { queue?: string; priority?: number }
  ): Promise<{ jobId: string }> {
    return this.client.enqueue({
      name,
      payload,
      queue: opts?.queue ?? this.defaultQueue,
      priority: opts?.priority ?? 0,
    });
  }

  /** Celery-style alias for enqueue. */
  delay(
    name: string,
    payload?: EnqueueOptions["payload"],
    opts?: { queue?: string; priority?: number }
  ): Promise<{ jobId: string }> {
    return this.enqueue(name, payload, opts);
  }

  /**
   * Start polling in the background inside this process.
   * Call once at app startup (e.g. Express listen callback).
   */
  async startEmbedded(): Promise<() => void> {
    if (this.stopWorker) return this.stopWorker;
    this.worker = new Worker({
      address: this.address,
      queues: this.workerOpts.queues ?? [this.defaultQueue],
      ...this.workerOpts,
    });
    for (const [name, handler] of this.handlers) {
      this.worker.task(name, handler);
    }
    this.stopWorker = await this.worker.startBackground();
    return this.stopWorker;
  }

  /** Stop embedded worker and close client connections. */
  shutdown(): void {
    this.stopWorker?.();
    this.stopWorker = undefined;
    this.client.close();
  }
}
