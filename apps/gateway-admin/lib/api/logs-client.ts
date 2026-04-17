import { normalizeGatewayApiBase } from './gateway-config.ts'
import { gatewayRequestInit } from './gateway-request.ts'
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
  return gatewayRequestInit(action, params, token, signal, standaloneBearerAuth)
}

async function parseJsonResponse<T>(response: Response): Promise<T> {
  const raw = await response.text()
  if (raw.length === 0) {
    if (!response.ok) {
      throw new Error(`request failed with status ${response.status} ${response.statusText}`.trim())
    }
    throw new Error('empty JSON response from gateway')
  }

  let payload: unknown
  try {
    payload = JSON.parse(raw)
  } catch (error) {
    if (!response.ok) {
      const snippet = raw.trim().slice(0, 120)
      const detail = snippet ? `: ${snippet}` : ''
      throw new Error(
        `request failed with status ${response.status} ${response.statusText}${detail}`.trim(),
        { cause: error },
      )
    }
    const message =
      error instanceof Error
        ? `invalid JSON response from gateway: ${error.message}`
        : 'invalid JSON response from gateway'
    throw new Error(message, { cause: error })
  }

  if (!response.ok) {
    const message =
      typeof (payload as { message?: unknown })?.message === 'string'
        ? (payload as { message: string }).message
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
