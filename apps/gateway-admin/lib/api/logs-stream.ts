import { normalizeGatewayApiBase } from './gateway-config.ts'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'
import type { LogEvent } from '../types/logs.ts'

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
  const modeOverride =
    typeof options?.standaloneBearerAuth === 'boolean'
      ? (options.standaloneBearerAuth ? 'true' : 'false')
      : undefined

  if (isStandaloneBearerAuthMode(options?.token, modeOverride)) {
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
    }
  }

  return () => {
    source.close()
  }
}
