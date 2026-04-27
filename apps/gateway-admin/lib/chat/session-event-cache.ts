'use client'

/**
 * Shared SSE event caches — single source of truth.
 *
 * Both `chat-session-provider.tsx` and `use-session-events.ts` import from
 * here so the two subscribers always read from the same Maps.
 *
 * LRU eviction: keeps the last 10 sessions in memory. Both maps are evicted
 * together on the same key when the limit is exceeded.
 */

import type { BridgeEvent } from '@/lib/acp/types'

const MAX_SESSIONS = 10

const _eventMap = new Map<string, BridgeEvent[]>()
const _seqMap = new Map<string, number>()
/** Access order — most-recently used at the end. */
const _accessOrder: string[] = []

function touchSession(sessionId: string) {
  const idx = _accessOrder.indexOf(sessionId)
  if (idx !== -1) {
    _accessOrder.splice(idx, 1)
  }
  _accessOrder.push(sessionId)

  // Evict when we exceed the limit
  while (_accessOrder.length > MAX_SESSIONS) {
    const evicted = _accessOrder.shift()
    if (evicted != null) {
      _eventMap.delete(evicted)
      _seqMap.delete(evicted)
    }
  }
}

/** In-memory event buffer keyed by session ID. */
export const sessionEventCache = {
  get(sessionId: string): BridgeEvent[] | undefined {
    const value = _eventMap.get(sessionId)
    if (value !== undefined) touchSession(sessionId)
    return value
  },
  set(sessionId: string, events: BridgeEvent[]): void {
    _eventMap.set(sessionId, events)
    touchSession(sessionId)
  },
}

/** Last seen sequence number keyed by session ID. */
export const sessionLastSeqCache = {
  get(sessionId: string): number | undefined {
    return _seqMap.get(sessionId)
  },
  set(sessionId: string, seq: number): void {
    _seqMap.set(sessionId, seq)
    // access order is already updated by sessionEventCache.set for the same session;
    // seq updates always accompany event writes so no separate touchSession needed.
  },
}
