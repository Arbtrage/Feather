import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { getJob, type JobRow } from "../api";

export function JobDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [job, setJob] = useState<JobRow | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    getJob(id)
      .then(setJob)
      .catch((e) => setError(e instanceof Error ? e.message : "not found"));
  }, [id]);

  if (error || !job) {
    return (
      <div>
        <Link to="/jobs">← Jobs</Link>
        <p className="error" style={{ marginTop: "1rem" }}>
          {error ?? "Job not found"}
        </p>
      </div>
    );
  }

  return (
    <div>
      <Link to="/jobs">← Jobs</Link>
      <h1 style={{ fontSize: "1.75rem", margin: "1rem 0" }}>{job.name}</h1>
      <div className="card" style={{ marginBottom: "1rem" }}>
        <dl>
          <dt>ID</dt>
          <dd>{job.id}</dd>
          <dt>State</dt>
          <dd>
            <span className={`badge badge-${job.state}`}>{job.state}</span>
          </dd>
          <dt>Queue</dt>
          <dd>{job.queue}</dd>
          <dt>Worker</dt>
          <dd>{job.worker_id || "—"}</dd>
          <dt>Created</dt>
          <dd>{new Date(job.created_at).toLocaleString()}</dd>
          {job.failure_reason && (
            <>
              <dt>Failure</dt>
              <dd className="error">{job.failure_reason}</dd>
            </>
          )}
        </dl>
      </div>
      <div className="card">
        <h2 style={{ marginBottom: "0.75rem" }}>Payload</h2>
        <pre>{JSON.stringify(job.payload, null, 2)}</pre>
      </div>
    </div>
  );
}
