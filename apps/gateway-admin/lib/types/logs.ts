export const LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error'] as const

export const LOG_SUBSYSTEMS = [
  'gateway',
  'mcp_server',
  'mcp_client',
  'api',
  'web',
  'oauth_relay',
  'auth_webui',
  'auth_mcp',
  'auth_upstream',
  'core_runtime',
] as const

export type LogLevel = (typeof LOG_LEVELS)[number]
export type LogSubsystem = (typeof LOG_SUBSYSTEMS)[number]
export type LogSurface = 'cli' | 'mcp' | 'api' | 'web' | 'core_runtime'

export interface LogEvent {
  event_id: string
  ts: number
  level: LogLevel
  subsystem: LogSubsystem
  surface: LogSurface
  action: string | null
  message: string
  request_id: string | null
  session_id: string | null
  correlation_id: string | null
  trace_id: string | null
  span_id: string | null
  instance: string | null
  auth_flow: string | null
  outcome_kind: string | null
  fields_json: Record<string, unknown>
  source_kind: string | null
  source_node_id: string | null
  source_device_id: string | null
  ingest_path: string | null
  upstream_event_id: string | null
}

export interface LogSearchQuery {
  text?: string
  after_ts?: number
  before_ts?: number
  levels?: LogLevel[]
  subsystems?: LogSubsystem[]
  surfaces?: LogSurface[]
  action?: string
  request_id?: string
  session_id?: string
  correlation_id?: string
  limit?: number
}

export interface LogSearchResult {
  events: LogEvent[]
  next_cursor: string | null
}

export interface LogStoreStats {
  on_disk_bytes: number
  oldest_retained_ts: number | null
  newest_retained_ts: number | null
  total_event_count: number
  dropped_event_count: number
  retention: {
    max_age_days: number
    max_bytes: number
  }
}

export interface LogFilterState {
  text: string
  subsystems: LogSubsystem[]
  levels: LogLevel[]
  limit: number
}

export interface LogStreamStatus {
  connected: boolean
  paused: boolean
  atLiveEdge: boolean
  buffered: number
  lastEventTs: number | null
  error: string | null
}
