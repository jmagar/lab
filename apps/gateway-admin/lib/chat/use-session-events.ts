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

export function buildSessionEventsUrl(
  acpBase: string,
  sessionId: string,
  lastSeq: number,
  origin: string,
) {
  const url = new URL(`${acpBase}/sessions/${sessionId}/events`, origin)
  url.searchParams.set('since', String(lastSeq))
  return url.toString()
}

export function consumeSessionEventBuffer(buffer: string, lastSeq: number) {
  const events: BridgeEvent[] = []
  let nextBuffer = buffer
  let nextLastSeq = lastSeq

  while (true) {
    const boundary = nextBuffer.indexOf('\n\n')
    if (boundary < 0) {
      break
    }

    const rawEvent = nextBuffer.slice(0, boundary)
    nextBuffer = nextBuffer.slice(boundary + 2)

    const dataLines = rawEvent
      .split('\n')
      .filter((line) => line.startsWith('data:'))
      .map((line) => line.slice(5).trimStart())

    if (dataLines.length === 0) {
      continue
    }

    const event = JSON.parse(dataLines.join('\n')) as BridgeEvent
    if (event.seq <= nextLastSeq) {
      continue
    }

    nextLastSeq = event.seq
    events.push(event)
  }

  return {
    buffer: nextBuffer,
    events,
    lastSeq: nextLastSeq,
  }
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

    const cachedEvents = sessionEventCache.get(sessionId) ?? []
    const cachedLastSeq = resolveLastSessionEventSeq(
      cachedEvents,
      sessionLastSeqCache.get(sessionId) ?? 0,
    )

    setEvents(cachedEvents)
    setConnectionState('connecting')
    lastSeqRef.current = cachedLastSeq

    const abortController = new AbortController()
    const url = buildSessionEventsUrl(
      acpBase,
      sessionId,
      lastSeqRef.current,
      window.location.origin,
    )

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
        const applyBuffer = () => {
          const consumed = consumeSessionEventBuffer(buffer, lastSeqRef.current)
          buffer = consumed.buffer

          for (const event of consumed.events) {
            lastSeqRef.current = event.seq
            sessionLastSeqCache.set(sessionId, event.seq)
            setEvents((current) => {
              const next = appendSessionEvent(current, event)
              sessionEventCache.set(sessionId, next)
              return next
            })
          }
        }

        while (true) {
          const { done, value } = await reader.read()
          if (done) {
            break
          }

          buffer += decoder.decode(value, { stream: true })
          applyBuffer()
        }

        buffer += decoder.decode()
        applyBuffer()
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
