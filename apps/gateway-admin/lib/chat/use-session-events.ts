'use client'

import * as React from 'react'
import { normalizeGatewayApiBase } from '@/lib/api/gateway-config'
import { gatewayHeaders } from '@/lib/api/gateway-request'
import { isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import type { AcpEvent, BridgeEvent } from '@/lib/acp/types'
import {
  appendSessionEvent,
  bridgeEventFromAcpEvent,
  deriveTranscriptAndActivity,
  resolveLastSessionEventSeq,
  resolveSessionStatusFromEvents,
} from './session-events'

export type SessionEventConnectionState = 'idle' | 'connecting' | 'open' | 'error'

const USE_MOCK_DATA = process.env.NEXT_PUBLIC_MOCK_DATA === 'true'

const sessionEventCache = new Map<string, BridgeEvent[]>()
const sessionLastSeqCache = new Map<string, number>()

function hasSequence(value: unknown): value is { seq: number } {
  return (
    typeof value === 'object' &&
    value !== null &&
    'seq' in value &&
    typeof (value as { seq?: unknown }).seq === 'number'
  )
}

function isAcpEvent(value: unknown): value is AcpEvent {
  return (
    hasSequence(value) &&
    'session_id' in value &&
    typeof (value as { session_id?: unknown }).session_id === 'string'
  )
}

function isBridgeEvent(value: unknown): value is BridgeEvent {
  return (
    hasSequence(value) &&
    'sessionId' in value &&
    typeof (value as { sessionId?: unknown }).sessionId === 'string'
  )
}

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

    let parsed: unknown
    try {
      parsed = JSON.parse(dataLines.join('\n'))
    } catch {
      continue
    }

    if (!isAcpEvent(parsed) && !isBridgeEvent(parsed)) {
      continue
    }

    const event = parsed
    if (event.seq <= nextLastSeq) {
      continue
    }

    nextLastSeq = event.seq
    if (isAcpEvent(event)) {
      events.push(bridgeEventFromAcpEvent(event))
    } else {
      events.push(event)
    }
  }

  return {
    buffer: nextBuffer,
    events,
    lastSeq: nextLastSeq,
  }
}

// ---------------------------------------------------------------------------
// rAF-batched event coalescing (C2)
//
// Without batching, 2000 terminal_output chunks/sec triggers 2000 React
// re-renders/sec — the thread cannot keep up; INP > 1s.
//
// The rAF coalescer accumulates incoming events in a queue and flushes
// the entire batch in a single setState call per animation frame (16ms).
//
// Backpressure (Codepath F — backgrounded-tab rAF starvation):
// If the queue grows beyond MAX_BATCH_QUEUE events (e.g., tab is backgrounded
// and rAF is throttled), oldest events are dropped with truncation flag.
// ---------------------------------------------------------------------------
const MAX_BATCH_QUEUE = 50_000

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

  // rAF batch queue: events waiting to be flushed.
  const batchQueueRef = React.useRef<BridgeEvent[]>([])
  const rafIdRef = React.useRef<number | null>(null)
  const sessionIdRef = React.useRef<string | null>(sessionId)

  // Keep sessionIdRef in sync for use inside rAF callback closure.
  React.useEffect(() => {
    sessionIdRef.current = sessionId
  }, [sessionId])

  const flushBatch = React.useCallback(() => {
    rafIdRef.current = null
    const batch = batchQueueRef.current
    batchQueueRef.current = []
    if (batch.length === 0) return

    const sid = sessionIdRef.current
    setEvents((current) => {
      let next = current
      for (const event of batch) {
        lastSeqRef.current = event.seq
        if (sid) sessionLastSeqCache.set(sid, event.seq)
        next = appendSessionEvent(next, event)
      }
      if (sid) sessionEventCache.set(sid, next)
      return next
    })
  }, [])

  const enqueueEvent = React.useCallback(
    (event: BridgeEvent) => {
      // Backpressure: drop oldest when queue is over budget (Codepath F).
      if (batchQueueRef.current.length >= MAX_BATCH_QUEUE) {
        batchQueueRef.current.shift()
      }
      batchQueueRef.current.push(event)

      // Schedule a single rAF flush if none is pending.
      if (rafIdRef.current == null) {
        rafIdRef.current = requestAnimationFrame(flushBatch)
      }
    },
    [flushBatch],
  )

  React.useEffect(() => {
    if (!sessionId) {
      setEvents([])
      setConnectionState('idle')
      lastSeqRef.current = 0
      batchQueueRef.current = []
      if (rafIdRef.current != null) {
        cancelAnimationFrame(rafIdRef.current)
        rafIdRef.current = null
      }
      return
    }

    if (USE_MOCK_DATA) {
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
            enqueueEvent(event)
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
      // Flush any pending batch on cleanup so state is consistent.
      if (rafIdRef.current != null) {
        cancelAnimationFrame(rafIdRef.current)
        rafIdRef.current = null
      }
      flushBatch()
    }
  }, [acpBase, enqueueEvent, flushBatch, requestCredentials, sessionId])

  const derived = React.useMemo(() => deriveTranscriptAndActivity(events), [events])

  return {
    events,
    connectionState,
    sessionStatus: resolveSessionStatusFromEvents(events),
    messages: derived.messages,
    activity: derived.activity,
  }
}
