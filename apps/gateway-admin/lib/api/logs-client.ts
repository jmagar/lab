import { normalizeGatewayApiBase } from './gateway-config.ts'
import { gatewayHeaders } from './gateway-request.ts'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'
import type { LogSearchQuery, LogSearchResult, LogStoreStats } from '../types/logs.ts'

export function logsActionUrl(baseUrl?: string) {
  return `${normalizeGatewayApiBase(baseUrl)}/logs`
}

export function logsRequestInit(
  action: string,
  params: object,
  token = process.env.NEXT_PUBLIC_API_TOKEN,
  signal?: AbortSignal,
  standaloneBearerAuth = isStandaloneBearerAuthMode(token),
): RequestInit {
  return {
    method: 'POST',
    headers: gatewayHeaders(token, standaloneBearerAuth),
    body: JSON.stringify({ action, params }),
    cache: 'no-store',
    credentials: standaloneBearerAuth ? 'omit' : 'include',
    signal,
  }
}

async function parseJsonResponse<T>(response: Response): Promise<T> {
  const payload = await response.json()
  if (!response.ok) {
    const message =
      typeof payload?.message === 'string'
        ? payload.message
        : `request failed with status ${response.status}`
    throw new Error(message)
  }
  return payload as T
}

export async function fetchLogs(
  query: LogSearchQuery,
  options?: {
    baseUrl?: string
    token?: string
    signal?: AbortSignal
    standaloneBearerAuth?: boolean
  },
): Promise<LogSearchResult> {
  const response = await fetch(
    logsActionUrl(options?.baseUrl),
    logsRequestInit(
      'logs.search',
      { query },
      options?.token,
      options?.signal,
      options?.standaloneBearerAuth,
    ),
  )

  return parseJsonResponse<LogSearchResult>(response)
}

export async function fetchLogStats(options?: {
  baseUrl?: string
  token?: string
  signal?: AbortSignal
  standaloneBearerAuth?: boolean
}): Promise<LogStoreStats> {
  const response = await fetch(
    logsActionUrl(options?.baseUrl),
    logsRequestInit(
      'logs.stats',
      {},
      options?.token,
      options?.signal,
      options?.standaloneBearerAuth,
    ),
  )

  return parseJsonResponse<LogStoreStats>(response)
}
