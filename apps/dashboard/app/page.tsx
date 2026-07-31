import { getQueues, type QueueInfo } from "@/lib/api";

export default async function OverviewPage() {
  let queues: QueueInfo[] = [];
  let error: string | null = null;
  try {
    queues = await getQueues();
  } catch (e) {
    error = e instanceof Error ? e.message : "failed to load";
  }

  return (
    <div>
      <h1 style={{ fontSize: "1.75rem", marginBottom: "1rem" }}>Overview</h1>
      {error && <p style={{ color: "var(--bad)" }}>{error}</p>}
      <div style={{ display: "grid", gap: "1rem", gridTemplateColumns: "repeat(auto-fit,minmax(220px,1fr))" }}>
        {queues.map((q) => (
          <div key={q.name} className="card">
            <h2 style={{ marginBottom: "0.75rem" }}>{q.name}</h2>
            <ul style={{ listStyle: "none", color: "var(--muted)", lineHeight: 1.8 }}>
              <li>Pending: {q.pending}</li>
              <li>Leased: {q.leased}</li>
              <li>Completed: {q.completed}</li>
              <li>Failed: {q.failed}</li>
            </ul>
          </div>
        ))}
        {!error && queues.length === 0 && <p style={{ color: "var(--muted)" }}>No queue data yet.</p>}
      </div>
    </div>
  );
}
