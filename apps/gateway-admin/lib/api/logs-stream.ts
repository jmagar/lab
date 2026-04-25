import { normalizeGatewayApiBase } from './gateway-config.ts'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'
import type { LogEvent } from '../types/logs.ts'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

export function logsStreamUrl(baseUrl?: string) {
  return `${normalizeGatewayApiBase(baseUrl)}/logs/stream`
}

export function connectLogStream(
  handlers: {
    onEvent: (event: LogEvent) => void
    onLag?: (skipped: number) => void
    onOpen?: () => void
    onError?: (message: string) => void
  },
  options?: {
    baseUrl?: string
    token?: string
    standaloneBearerAuth?: boolean
  },
) {
  if (USE_MOCK_DATA) {
    handlers.onOpen?.()
    const buildMockEvent = (): LogEvent => ({
      event_id: `mock-stream-${Date.now()}`,
      ts: Date.now(),
      level: 'info',
      subsystem: 'gateway',
      surface: 'web',
      action: 'logs.stream',
      message: 'Mock stream heartbeat.',
      request_id: null,
      session_id: null,
      correlation_id: null,
      trace_id: null,
      span_id: null,
      instance: null,
      auth_flow: null,
      outcome_kind: 'ok',
      fields_json: { source: 'mock-stream' },
      source_kind: 'mock',
      source_node_id: null,
      source_device_id: null,
      ingest_path: 'mock',
      upstream_event_id: null,
    })
    // Emit one immediate event so the mock stream feels responsive instead of
    // leaving the UI blank for the first 12s interval tick.
    const initialTimeoutId = window.setTimeout(() => {
      handlers.onEvent(buildMockEvent())
    }, 0)
    const intervalId = window.setInterval(() => {
      handlers.onEvent(buildMockEvent())
    }, 12000)

    return () => {
      window.clearTimeout(initialTimeoutId)
      window.clearInterval(intervalId)
    }
  }

  // EventSource cannot carry bearer headers — only same-origin session cookies.
  // Refuse explicitly so callers in standalone-bearer deployments fail fast
  // instead of silently sending unauthenticated stream requests.
  const standaloneBearerAuth =
    options?.standaloneBearerAuth ?? isStandaloneBearerAuthMode(options?.token)
  if (standaloneBearerAuth) {
    throw new Error(
      'live log streaming uses hosted same-origin session auth in v1; standalone bearer mode is not supported for EventSource',
    )
  }

  const source = new EventSource(logsStreamUrl(options?.baseUrl), {
    withCredentials: true,
  })

  source.onopen = () => {
    handlers.onOpen?.()
  }
  source.onmessage = (message) => {
    let event: LogEvent
    try {
      event = JSON.parse(message.data) as LogEvent
    } catch {
      handlers.onError?.('received malformed log event')
      return
    }
    handlers.onEvent(event)
  }
  source.addEventListener('lag', (message) => {
    const skipped = Number(message.data)
    if (Number.isInteger(skipped) && skipped > 0) {
      handlers.onLag?.(skipped)
      return
    }
    handlers.onError?.('received malformed lag event')
  })
  source.onerror = () => {
    if (source.readyState === EventSource.CLOSED) {
      handlers.onError?.('live stream disconnected')
    } else {
      // CONNECTING — browser is retrying automatically
      handlers.onError?.('live stream reconnecting…')
    }
  }

  return () => {
    source.close()
  }
}
