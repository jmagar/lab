'use client'

import * as React from 'react'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import type { BridgeEvent } from '@/lib/acp/types'
import { appendSessionEvent, resolveLastSessionEventSeq } from './session-events'

export type SessionEventConnectionState = 'idle' | 'connecting' | 'open' | 'error'

const sessionEventCache = new Map<string, BridgeEvent[]>()
const sessionLastSeqCache = new Map<string, number>()

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

    const cachedEvents = sessionEventCache.get(sessionId) ?? []
    const cachedLastSeq = resolveLastSessionEventSeq(
      cachedEvents,
      sessionLastSeqCache.get(sessionId) ?? 0,
    )

    setEvents(cachedEvents)
    setConnectionState('connecting')
    lastSeqRef.current = cachedLastSeq

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
              if (event.seq <= lastSeqRef.current) {
                continue
              }

              lastSeqRef.current = event.seq
              sessionLastSeqCache.set(sessionId, event.seq)
              setEvents((current) => {
                const next = appendSessionEvent(current, event)
                sessionEventCache.set(sessionId, next)
                return next
              })
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
