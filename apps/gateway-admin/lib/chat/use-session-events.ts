'use client'

import * as React from 'react'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import type { BridgeEvent } from '@/lib/acp/types'

export type SessionEventConnectionState = 'idle' | 'connecting' | 'open' | 'error'

function mergeEvents(current: BridgeEvent[], incoming: BridgeEvent) {
  if (current.some((event) => event.seq === incoming.seq)) {
    return current
  }
  return [...current, incoming].sort((left, right) => left.seq - right.seq)
}

export function useSessionEvents(sessionId: string | null) {
  const acpBase = React.useMemo(() => `${normalizeGatewayApiBase()}/acp`, [])
  const standaloneBearerAuth = React.useMemo(() => isStandaloneBearerAuthMode(), [])
  const requestCredentials = React.useMemo<RequestCredentials>(
    () => (standaloneBearerAuth ? 'omit' : 'include'),
    [standaloneBearerAuth],
  )
  const [events, setEvents] = React.useState<BridgeEvent[]>([])
  const [connectionState, setConnectionState] = React.useState<SessionEventConnectionState>('idle')
  const lastSeqRef = React.useRef(0)

  React.useEffect(() => {
    if (!sessionId) {
      setEvents([])
      setConnectionState('idle')
      lastSeqRef.current = 0
      return
    }

    setEvents([])
    setConnectionState('connecting')
    lastSeqRef.current = 0

    const abortController = new AbortController()
    const url = new URL(`${acpBase}/sessions/${sessionId}/events`, window.location.origin)
    url.searchParams.set('since', String(lastSeqRef.current))

    void (async () => {
      try {
        const response = await fetch(url, {
          method: 'GET',
          headers: gatewayHeaders(),
          credentials: requestCredentials,
          cache: 'no-store',
          signal: abortController.signal,
        })

        if (!response.ok || !response.body) {
          setConnectionState('error')
          return
        }

        setConnectionState('open')

        const reader = response.body.getReader()
        const decoder = new TextDecoder()
        let buffer = ''

        while (true) {
          const { done, value } = await reader.read()
          if (done) {
            break
          }

          buffer += decoder.decode(value, { stream: true })

          while (true) {
            const boundary = buffer.indexOf('\n\n')
            if (boundary < 0) {
              break
            }

            const rawEvent = buffer.slice(0, boundary)
            buffer = buffer.slice(boundary + 2)

            const dataLines = rawEvent
              .split('\n')
              .filter((line) => line.startsWith('data:'))
              .map((line) => line.slice(5).trimStart())

            if (dataLines.length === 0) {
              continue
            }

            try {
              const event = JSON.parse(dataLines.join('\n')) as BridgeEvent
              lastSeqRef.current = Math.max(lastSeqRef.current, event.seq)
              setEvents((current) => mergeEvents(current, event))
            } catch {
              setConnectionState('error')
              return
            }
          }
        }
      } catch {
        if (!abortController.signal.aborted) {
          setConnectionState('error')
        }
      }
    })()

    return () => {
      abortController.abort()
    }
  }, [acpBase, requestCredentials, sessionId])

  return { events, connectionState }
}
