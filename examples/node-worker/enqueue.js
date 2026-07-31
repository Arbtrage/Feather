import { FeatherClient } from "@arbtrage/feather";

const client = new FeatherClient(process.env.FEATHER_ADDRESS ?? "localhost:50051");

async function main() {
  const count = Number(process.env.COUNT ?? 10);
  for (let i = 0; i < count; i++) {
    const { jobId } = await client.enqueue({
      name: "echo",
      payload: JSON.stringify({ n: i + 1 }),
    });
    console.log("enqueued", jobId);
  }
  client.close();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
