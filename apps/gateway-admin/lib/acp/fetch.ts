'use client'

/**
 * Standalone ACP fetch utility — derives acpBase and requestCredentials
 * per call so callers don't need to hold these as state.
 *
 * Internal to provider; surface hooks import directly.
 * NOT exported from context values.
 */

import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'

function getAcpBase(): string {
  return `${normalizeGatewayApiBase()}/acp`
}

function getRequestCredentials(): RequestCredentials {
  return isStandaloneBearerAuthMode() ? 'omit' : 'include'
}

export function createAcpFetcher() {
  const acpBase = getAcpBase()
  const requestCredentials = getRequestCredentials()

  return async function fetchAcp(path: string, init?: RequestInit): Promise<Response> {
    const headers = new Headers(init?.headers ?? {})
    const authHeaders = gatewayHeaders()
    for (const [key, value] of Object.entries(authHeaders)) {
      headers.set(key, value)
    }

    // Set Content-Type for requests with a body, unless the caller already set it.
    if (init?.body != null && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json')
    }

    return fetch(`${acpBase}${path}`, {
      ...init,
      headers,
      credentials: requestCredentials,
      cache: 'no-store',
    })
  }
}

export async function requestAcpSubscribeTicket(
  fetchAcp: ReturnType<typeof createAcpFetcher>,
  sessionId: string,
  signal?: AbortSignal,
) {
  const response = await fetchAcp(
    `/sessions/${encodeURIComponent(sessionId)}/subscribe_ticket`,
    {
      method: 'POST',
      signal,
    },
  )

  if (!response.ok) {
    throw new Error(`ACP subscribe ticket request failed with ${response.status}`)
  }

  const body = (await response.json()) as { ticket?: unknown }
  if (typeof body.ticket !== 'string' || body.ticket.length === 0) {
    throw new Error('ACP subscribe ticket response missing ticket')
  }

  return body.ticket
}
