import Link from "next/link";
import { getJobs, type JobRow } from "@/lib/api";

function badgeClass(state: string) {
  return `badge badge-${state}`;
}

export default async function JobsPage() {
  let jobs: JobRow[] = [];
  let error: string | null = null;
  try {
    jobs = await getJobs({ limit: 100 });
  } catch (e) {
    error = e instanceof Error ? e.message : "failed to load";
  }

  return (
    <div>
      <h1 style={{ fontSize: "1.75rem", marginBottom: "1rem" }}>Jobs</h1>
      {error && <p style={{ color: "var(--bad)" }}>{error}</p>}
      <div className="card" style={{ overflowX: "auto" }}>
        <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
          <thead>
            <tr style={{ textAlign: "left", color: "var(--muted)" }}>
              <th style={{ padding: "0.5rem" }}>ID</th>
              <th>Name</th>
              <th>State</th>
              <th>Queue</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((j) => (
              <tr key={j.id} style={{ borderTop: "1px solid #2a3544" }}>
                <td style={{ padding: "0.5rem" }}>
                  <Link href={`/jobs/${j.id}`}>{j.id.slice(0, 8)}…</Link>
                </td>
                <td>{j.name}</td>
                <td>
                  <span className={badgeClass(j.state)}>{j.state}</span>
                </td>
                <td>{j.queue}</td>
                <td style={{ color: "var(--muted)" }}>{new Date(j.created_at).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {!error && jobs.length === 0 && (
          <p style={{ color: "var(--muted)", padding: "1rem" }}>No jobs yet.</p>
        )}
      </div>
    </div>
  );
}
