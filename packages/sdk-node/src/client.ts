import * as grpc from "@grpc/grpc-js";
import { createChannel, loadProto, promisify } from "./grpc.js";

export type EnqueueOptions = {
  queue?: string;
  name: string;
  payload?: Buffer | Uint8Array | string;
  priority?: number;
};

export type Job = {
  id: string;
  queue: string;
  name: string;
  payload: Buffer;
  state: string;
};

export class FeatherClient {
  private queueClient: any;
  private channel: grpc.Client;

  constructor(address = process.env.FEATHER_ADDRESS ?? "localhost:50051") {
    const { queue } = loadProto();
    this.channel = createChannel(address);
    this.queueClient = new queue(this.channel, grpc.credentials.createInsecure());
  }

  async enqueue(opts: EnqueueOptions): Promise<{ jobId: string }> {
    const payload =
      typeof opts.payload === "string"
        ? Buffer.from(opts.payload)
        : Buffer.from(opts.payload ?? []);
    const enqueue = promisify<any, any>(this.queueClient.enqueue.bind(this.queueClient));
    const res = await enqueue({
      queue: opts.queue ?? "default",
      name: opts.name,
      payload,
      priority: opts.priority ?? 0,
    });
    return { jobId: res.jobId };
  }

  async getJob(jobId: string): Promise<Job | null> {
    const getJob = promisify<any, any>(this.queueClient.getJob.bind(this.queueClient));
    const res = await getJob({ jobId });
    if (!res.job) return null;
    const j = res.job;
    return {
      id: j.id,
      queue: j.queue,
      name: j.name,
      payload: Buffer.from(j.payload ?? []),
      state: String(j.state),
    };
  }

  close(): void {
    this.queueClient.close?.();
    this.channel.close();
  }
}
