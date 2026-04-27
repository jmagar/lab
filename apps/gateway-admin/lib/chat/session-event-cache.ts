'use client'

/**
 * Shared SSE event caches — single source of truth.
 *
 * Both `chat-session-provider.tsx` and `use-session-events.ts` import from
 * here so the two subscribers always read from the same Maps.
 */

import type { BridgeEvent } from '@/lib/acp/types'

/** In-memory event buffer keyed by session ID. */
export const sessionEventCache = new Map<string, BridgeEvent[]>()

/** Last seen sequence number keyed by session ID. */
export const sessionLastSeqCache = new Map<string, number>()
