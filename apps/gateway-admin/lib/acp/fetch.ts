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

    return fetch(`${acpBase}${path}`, {
      ...init,
      headers,
      credentials: requestCredentials,
      cache: 'no-store',
    })
  }
}
