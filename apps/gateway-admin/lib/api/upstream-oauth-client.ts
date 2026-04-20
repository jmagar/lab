import type { ProbeResponse, StartResponse, UpstreamEntry, UpstreamOauthStatus } from '@/lib/types/upstream-oauth'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode'
import { normalizeGatewayApiBase } from './gateway-config'
import { gatewayHeaders } from './gateway-request'

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const token = process.env.NEXT_PUBLIC_API_TOKEN
  const standaloneBearerAuth = isStandaloneBearerAuthMode(token)
  const credentials: RequestCredentials = standaloneBearerAuth ? 'omit' : 'include'
  const base = normalizeGatewayApiBase()

  const res = await fetch(`${base}${path}`, {
    credentials,
    ...init,
    headers: { ...gatewayHeaders(token, standaloneBearerAuth), ...init?.headers },
  })
  if (!res.ok) {
    let body: { kind?: string; message?: string } = {}
    try { body = await res.json() } catch { /* ignore */ }
    throw Object.assign(new Error(body.message ?? `HTTP ${res.status}`), { kind: body.kind })
  }
  return res.json() as Promise<T>
}

export const upstreamOauthApi = {
  listUpstreams(signal?: AbortSignal): Promise<UpstreamEntry[]> {
    return apiFetch('/gateway/oauth/upstreams', { signal })
  },

  status(upstream: string, signal?: AbortSignal): Promise<UpstreamOauthStatus> {
    return apiFetch(`/gateway/oauth/status?upstream=${encodeURIComponent(upstream)}`, { signal })
  },

  start(upstream: string, signal?: AbortSignal): Promise<StartResponse> {
    return apiFetch('/gateway/oauth/start', {
      method: 'POST',
      body: JSON.stringify({ upstream }),
      signal,
    })
  },

  clear(upstream: string, signal?: AbortSignal): Promise<{ ok: boolean }> {
    return apiFetch(
      `/gateway/oauth/clear?upstream=${encodeURIComponent(upstream)}&confirm=true`,
      { method: 'POST', signal },
    )
  },

  probe(url: string, signal?: AbortSignal): Promise<ProbeResponse> {
    return apiFetch('/gateway/oauth/probe', {
      method: 'POST',
      body: JSON.stringify({ url }),
      headers: { 'Content-Type': 'application/json' },
      signal,
    })
  },
}
