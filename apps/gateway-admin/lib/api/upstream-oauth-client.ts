import type { StartResponse, UpstreamEntry, UpstreamOauthStatus } from '@/lib/types/upstream-oauth'

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, { credentials: 'include', ...init })
  if (!res.ok) {
    let body: { kind?: string; message?: string } = {}
    try { body = await res.json() } catch { /* ignore */ }
    throw Object.assign(new Error(body.message ?? `HTTP ${res.status}`), { kind: body.kind })
  }
  return res.json() as Promise<T>
}

export const upstreamOauthApi = {
  listUpstreams(signal?: AbortSignal): Promise<UpstreamEntry[]> {
    return apiFetch('/v1/gateway/oauth/upstreams', { signal })
  },

  status(upstream: string, signal?: AbortSignal): Promise<UpstreamOauthStatus> {
    return apiFetch(`/v1/gateway/oauth/status?upstream=${encodeURIComponent(upstream)}`, { signal })
  },

  start(upstream: string, signal?: AbortSignal): Promise<StartResponse> {
    return apiFetch('/v1/gateway/oauth/start', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ upstream }),
      signal,
    })
  },

  clear(upstream: string, signal?: AbortSignal): Promise<{ ok: boolean }> {
    return apiFetch(
      `/v1/gateway/oauth/clear?upstream=${encodeURIComponent(upstream)}&confirm=true`,
      { method: 'POST', signal },
    )
  },
}
