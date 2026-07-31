const API = process.env.FEATHER_API_URL ?? process.env.NEXT_PUBLIC_FEATHER_API_URL ?? "http://localhost:8080";

export type QueueInfo = {
  name: string;
  pending: number;
  leased: number;
  completed: number;
  failed: number;
};

export type JobRow = {
  id: string;
  queue: string;
  name: string;
  payload: unknown;
  state: string;
  priority: number;
  attempt: number;
  worker_id?: string | null;
  created_at: string;
  lease_expires_at?: string | null;
  failure_reason?: string | null;
};

async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(`${API}${path}`, { cache: "no-store" });
  if (!res.ok) throw new Error(`${path} ${res.status}`);
  return res.json() as Promise<T>;
}

export async function getQueues(): Promise<QueueInfo[]> {
  const json = await fetchJson<{ data: QueueInfo[] }>("/api/v1/queues");
  return json.data;
}

export async function getJobs(params?: { queue?: string; state?: string; limit?: number }): Promise<JobRow[]> {
  const q = new URLSearchParams();
  if (params?.queue) q.set("queue", params.queue);
  if (params?.state) q.set("state", params.state);
  if (params?.limit) q.set("limit", String(params.limit));
  const suffix = q.toString() ? `?${q}` : "";
  const json = await fetchJson<{ data: JobRow[] }>(`/api/v1/jobs${suffix}`);
  return json.data;
}

export async function getJob(id: string): Promise<JobRow> {
  const json = await fetchJson<{ data: JobRow }>(`/api/v1/jobs/${id}`);
  return json.data;
}
