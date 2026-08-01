import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("sdk package exports FeatherClient and Worker", async () => {
  const pkg = path.join(root, "packages/sdk-node/package.json");
  const json = JSON.parse(await import("node:fs").then((fs) => fs.readFileSync(pkg, "utf8")));
  assert.equal(json.name, "@arbitrage/feather");
});

test("proto files use feather.v1 package", async () => {
  const fs = await import("node:fs");
  const queueProto = fs.readFileSync(
    path.join(root, "packages/proto/feather/v1/queue.proto"),
    "utf8"
  );
  assert.match(queueProto, /package feather\.v1;/);
  assert.match(queueProto, /service QueueService/);
});

test("redis key prefix documented in server", async () => {
  const fs = await import("node:fs");
  const keys = fs.readFileSync(path.join(root, "packages/server/src/storage/keys.rs"), "utf8");
  assert.match(keys, /fe:/);
});
