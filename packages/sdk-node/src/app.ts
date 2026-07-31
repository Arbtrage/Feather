import { FeatherClient, type EnqueueOptions } from "./client.js";
import { startUiServer } from "./ui-server.js";
import { Worker, type Handler, type WorkerOptions } from "./worker.js";

export type FeatherUiOptions = {
  enabled?: boolean;
  port?: number;
  adminUrl?: string;
  openBrowser?: boolean;
};

export type FeatherAppOptions = {
  address?: string;
  queue?: string;
  worker?: WorkerOptions;
  ui?: FeatherUiOptions;
};

export type StartEmbeddedOptions = {
  ui?: boolean;
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
  private uiOpts: FeatherUiOptions;
  private handlers = new Map<string, Handler>();
  private worker?: Worker;
  private stopWorker?: () => void;
  private stopUi?: () => void;

  constructor(opts: FeatherAppOptions = {}) {
    this.address = opts.address ?? process.env.FEATHER_ADDRESS ?? "localhost:50051";
    this.client = new FeatherClient(this.address);
    this.defaultQueue = opts.queue ?? "default";
    this.workerOpts = opts.worker ?? {};
    this.uiOpts = opts.ui ?? {};
  }

  private adminUrl(): string {
    return (
      this.uiOpts.adminUrl ??
      process.env.FEATHER_ADMIN_URL ??
      "http://localhost:8080"
    );
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
   * Pass `{ ui: true }` to also start the monitoring UI when configured.
   */
  async startEmbedded(opts?: StartEmbeddedOptions): Promise<() => void> {
    if (this.stopWorker) {
      if (opts?.ui) await this.startUI();
      return () => this.shutdown();
    }
    this.worker = new Worker({
      address: this.address,
      queues: this.workerOpts.queues ?? [this.defaultQueue],
      ...this.workerOpts,
    });
    for (const [name, handler] of this.handlers) {
      this.worker.task(name, handler);
    }
    this.stopWorker = await this.worker.startBackground();

    const startUi = opts?.ui ?? this.uiOpts.enabled ?? false;
    if (startUi) {
      await this.startUI();
    }

    return () => this.shutdown();
  }

  /** Start the read-only monitoring UI (requires bundled ui-static assets). */
  async startUI(): Promise<{ url: string; stop: () => void }> {
    if (this.stopUi) {
      const port = this.uiOpts.port ?? 3001;
      return { url: `http://127.0.0.1:${port}`, stop: this.stopUi };
    }

    const port = this.uiOpts.port ?? 3001;
    const started = await startUiServer({
      port,
      adminUrl: this.adminUrl(),
    });
    this.stopUi = started.stop;

    if (this.uiOpts.openBrowser) {
      const { exec } = await import("node:child_process");
      const cmd =
        process.platform === "win32"
          ? `start ${started.url}`
          : process.platform === "darwin"
            ? `open ${started.url}`
            : `xdg-open ${started.url}`;
      exec(cmd);
    }

    return started;
  }

  /** Stop embedded worker, UI server, and close client connections. */
  shutdown(): void {
    this.stopUi?.();
    this.stopUi = undefined;
    this.stopWorker?.();
    this.stopWorker = undefined;
    this.client.close();
  }
}
