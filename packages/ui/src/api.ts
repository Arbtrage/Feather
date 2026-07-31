export type FeatherUiConfig = {
  adminUrl: string;
};

declare global {
  interface Window {
    __FEATHER_CONFIG__?: FeatherUiConfig;
  }
}

export function getAdminUrl(): string {
  return (
    window.__FEATHER_CONFIG__?.adminUrl ??
    import.meta.env.VITE_ADMIN_URL ??
    "http://localhost:8080"
  );
}

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
  const res = await fetch(`${getAdminUrl()}${path}`);
  if (!res.ok) throw new Error(`${path} ${res.status}`);
  return res.json() as Promise<T>;
}

export async function getQueues(): Promise<QueueInfo[]> {
  const json = await fetchJson<{ data: QueueInfo[] }>("/api/v1/queues");
  return json.data;
}

export async function getJobs(params?: {
  queue?: string;
  state?: string;
  limit?: number;
}): Promise<JobRow[]> {
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
