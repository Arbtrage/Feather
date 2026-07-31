import Link from "next/link";
import { getJob } from "@/lib/api";

export default async function JobDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  let job = null;
  let error: string | null = null;
  try {
    job = await getJob(id);
  } catch (e) {
    error = e instanceof Error ? e.message : "not found";
  }

  if (error || !job) {
    return (
      <div>
        <Link href="/jobs">← Jobs</Link>
        <p style={{ color: "var(--bad)", marginTop: "1rem" }}>{error ?? "Job not found"}</p>
      </div>
    );
  }

  return (
    <div>
      <Link href="/jobs">← Jobs</Link>
      <h1 style={{ fontSize: "1.75rem", margin: "1rem 0" }}>{job.name}</h1>
      <div className="card" style={{ marginBottom: "1rem" }}>
        <dl style={{ display: "grid", gridTemplateColumns: "140px 1fr", gap: "0.5rem" }}>
          <dt style={{ color: "var(--muted)" }}>ID</dt>
          <dd>{job.id}</dd>
          <dt style={{ color: "var(--muted)" }}>State</dt>
          <dd>
            <span className={`badge badge-${job.state}`}>{job.state}</span>
          </dd>
          <dt style={{ color: "var(--muted)" }}>Queue</dt>
          <dd>{job.queue}</dd>
          <dt style={{ color: "var(--muted)" }}>Worker</dt>
          <dd>{job.worker_id || "—"}</dd>
          <dt style={{ color: "var(--muted)" }}>Created</dt>
          <dd>{new Date(job.created_at).toLocaleString()}</dd>
          {job.failure_reason && (
            <>
              <dt style={{ color: "var(--muted)" }}>Failure</dt>
              <dd style={{ color: "var(--bad)" }}>{job.failure_reason}</dd>
            </>
          )}
        </dl>
      </div>
      <div className="card">
        <h2 style={{ marginBottom: "0.75rem" }}>Payload</h2>
        <pre
          style={{
            background: "#0b1017",
            padding: "1rem",
            borderRadius: 8,
            overflow: "auto",
            fontSize: "0.85rem",
          }}
        >
          {JSON.stringify(job.payload, null, 2)}
        </pre>
      </div>
    </div>
  );
}
