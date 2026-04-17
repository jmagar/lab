import { normalizeGatewayApiBase } from './gateway-config.ts'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'
import type { LogEvent } from '../types/logs.ts'

export function logsStreamUrl(baseUrl?: string) {
  return `${normalizeGatewayApiBase(baseUrl)}/logs/stream`
}

export function connectLogStream(
  handlers: {
    onEvent: (event: LogEvent) => void
    onOpen?: () => void
    onError?: (message: string) => void
  },
  options?: {
    baseUrl?: string
    token?: string
    standaloneBearerAuth?: boolean
  },
) {
  if (isStandaloneBearerAuthMode(options?.token, options?.standaloneBearerAuth ? 'true' : undefined)) {
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
    handlers.onEvent(JSON.parse(message.data) as LogEvent)
  }
  source.onerror = () => {
    handlers.onError?.('live stream disconnected')
  }

  return () => {
    source.close()
  }
}
