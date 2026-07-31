import path from "node:path";
import { fileURLToPath } from "node:url";
import * as grpc from "@grpc/grpc-js";
import * as protoLoader from "@grpc/proto-loader";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// Bundled protos ship inside the npm package at proto/feather/v1/
const PROTO_ROOT = path.resolve(__dirname, "../proto");

export type LoadedClients = {
  queue: any;
  worker: any;
};

export function loadProto(): LoadedClients {
  const packageDefinition = protoLoader.loadSync(
    [
      path.join(PROTO_ROOT, "feather/v1/common.proto"),
      path.join(PROTO_ROOT, "feather/v1/job.proto"),
      path.join(PROTO_ROOT, "feather/v1/queue.proto"),
      path.join(PROTO_ROOT, "feather/v1/worker.proto"),
    ],
    {
      keepCase: false,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true,
      includeDirs: [PROTO_ROOT],
    }
  );
  const feather = grpc.loadPackageDefinition(packageDefinition) as any;
  return {
    queue: feather.feather.v1.QueueService,
    worker: feather.feather.v1.WorkerService,
  };
}

export function createChannel(address: string): grpc.Client {
  return new grpc.Client(address, grpc.credentials.createInsecure());
}

export function promisify<TReq, TRes>(
  fn: (req: TReq, cb: (err: grpc.ServiceError | null, res: TRes) => void) => void
): (req: TReq) => Promise<TRes> {
  return (req: TReq) =>
    new Promise((resolve, reject) => {
      fn(req, (err, res) => {
        if (err) reject(err);
        else resolve(res);
      });
    });
}
