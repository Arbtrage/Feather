import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { getJobs, type JobRow } from "../api";

function badgeClass(state: string) {
  return `badge badge-${state}`;
}

export function JobsPage() {
  const [jobs, setJobs] = useState<JobRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getJobs({ limit: 100 })
      .then(setJobs)
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load"));
  }, []);

  return (
    <div>
      <h1 style={{ fontSize: "1.75rem", marginBottom: "1rem" }}>Jobs</h1>
      {error && <p className="error">{error}</p>}
      <div className="card" style={{ overflowX: "auto" }}>
        <table>
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>State</th>
              <th>Queue</th>
              <th>Created</th>
            </tr>
          </thead>
          <tbody>
            {jobs.map((j) => (
              <tr key={j.id}>
                <td>
                  <Link to={`/jobs/${j.id}`}>{j.id.slice(0, 8)}…</Link>
                </td>
                <td>{j.name}</td>
                <td>
                  <span className={badgeClass(j.state)}>{j.state}</span>
                </td>
                <td>{j.queue}</td>
                <td className="muted">
                  {new Date(j.created_at).toLocaleString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {!error && jobs.length === 0 && (
          <p className="muted" style={{ padding: "1rem" }}>
            No jobs yet.
          </p>
        )}
      </div>
    </div>
  );
}
