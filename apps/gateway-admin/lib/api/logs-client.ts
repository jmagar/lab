import { normalizeGatewayApiBase } from './gateway-config.ts'
import { gatewayRequestInit } from './gateway-request.ts'
import type { LogSearchQuery, LogSearchResult, LogStoreStats } from '../types/logs.ts'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

const MOCK_LOG_EVENTS = [
  {
    event_id: 'log_1',
    ts: Date.now() - 1000 * 60 * 18,
    level: 'info',
    subsystem: 'gateway',
    surface: 'web',
    action: 'gateway.reload',
    message: 'Gateway catalog refreshed from mock data.',
    request_id: 'req_mock_01',
    session_id: null,
    correlation_id: null,
    trace_id: null,
    span_id: null,
    instance: 'gw-2',
    auth_flow: null,
    outcome_kind: 'ok',
    fields_json: { route: '/gateways' },
    source_kind: 'mock',
    source_node_id: null,
    source_device_id: null,
    ingest_path: 'mock',
    upstream_event_id: null,
  },
  {
    event_id: 'log_2',
    ts: Date.now() - 1000 * 60 * 12,
    level: 'warn',
    subsystem: 'api',
    surface: 'web',
    action: 'registry.list',
    message: 'Registry route is using local mock data.',
    request_id: 'req_mock_02',
    session_id: null,
    correlation_id: null,
    trace_id: null,
    span_id: null,
    instance: null,
    auth_flow: null,
    outcome_kind: 'mock',
    fields_json: { route: '/registry' },
    source_kind: 'mock',
    source_node_id: null,
    source_device_id: null,
    ingest_path: 'mock',
    upstream_event_id: null,
  },
  {
    event_id: 'log_3',
    ts: Date.now() - 1000 * 60 * 5,
    level: 'error',
    subsystem: 'auth_webui',
    surface: 'web',
    action: 'session.refresh',
    message: 'Sample auth warning retained in mock timeline.',
    request_id: 'req_mock_03',
    session_id: null,
    correlation_id: null,
    trace_id: null,
    span_id: null,
    instance: null,
    auth_flow: 'session',
    outcome_kind: 'expired',
    fields_json: { route: '/settings' },
    source_kind: 'mock',
    source_node_id: null,
    source_device_id: null,
    ingest_path: 'mock',
    upstream_event_id: null,
  },
] as const

export type LogsRequestOptions = {
  baseUrl?: string
  token?: string
  signal?: AbortSignal
  standaloneBearerAuth?: boolean
}

export function logsActionUrl(baseUrl?: string) {
  return `${normalizeGatewayApiBase(baseUrl)}/logs`
}

export function logsRequestInit(
  action: string,
  params: object,
  token?: string,
  signal?: AbortSignal,
  standaloneBearerAuth = false,
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
  options?: LogsRequestOptions,
): Promise<LogSearchResult> {
  if (USE_MOCK_DATA) {
    options?.signal?.throwIfAborted?.()
    const filtered = MOCK_LOG_EVENTS
      .filter((event) => !query.after_ts || event.ts >= query.after_ts)
      .filter((event) => !query.before_ts || event.ts <= query.before_ts)
      .filter((event) => !query.levels?.length || query.levels.includes(event.level))
      .filter((event) => !query.subsystems?.length || query.subsystems.includes(event.subsystem))
      .filter((event) => !query.surfaces?.length || query.surfaces.includes(event.surface))
      .filter((event) => !query.action || event.action === query.action)
      .filter((event) => !query.text || `${event.message} ${JSON.stringify(event.fields_json)}`.toLowerCase().includes(query.text.toLowerCase()))
      .slice(0, query.limit ?? 200)
    return { events: structuredClone(filtered), next_cursor: null }
  }
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

export async function fetchLogStats(options?: LogsRequestOptions): Promise<LogStoreStats> {
  if (USE_MOCK_DATA) {
    options?.signal?.throwIfAborted?.()
    const timestamps = MOCK_LOG_EVENTS.map((event) => event.ts)
    return {
      on_disk_bytes: 48_320,
      oldest_retained_ts: Math.min(...timestamps),
      newest_retained_ts: Math.max(...timestamps),
      total_event_count: MOCK_LOG_EVENTS.length,
      dropped_event_count: 0,
      retention: {
        max_age_days: 7,
        max_bytes: 50 * 1024 * 1024,
      },
    }
  }
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
