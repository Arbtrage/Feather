import { useEffect, useState } from "react";
import { getQueues, type QueueInfo } from "../api";

export function OverviewPage() {
  const [queues, setQueues] = useState<QueueInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getQueues()
      .then(setQueues)
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load"));
  }, []);

  return (
    <div>
      <h1 style={{ fontSize: "1.75rem", marginBottom: "1rem" }}>Overview</h1>
      {error && <p className="error">{error}</p>}
      <div className="grid">
        {queues.map((q) => (
          <div key={q.name} className="card">
            <h2 style={{ marginBottom: "0.75rem" }}>{q.name}</h2>
            <ul style={{ listStyle: "none", lineHeight: 1.8 }} className="muted">
              <li>Pending: {q.pending}</li>
              <li>Leased: {q.leased}</li>
              <li>Completed: {q.completed}</li>
              <li>Failed: {q.failed}</li>
            </ul>
          </div>
        ))}
        {!error && queues.length === 0 && (
          <p className="muted">No queue data yet.</p>
        )}
      </div>
    </div>
  );
}
