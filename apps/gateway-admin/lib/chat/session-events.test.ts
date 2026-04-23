import test from 'node:test'
import assert from 'node:assert/strict'

import type { BridgeEvent } from '@/lib/acp/types'
import {
  appendSessionEvent,
  MAX_SESSION_EVENTS,
  resolveLastSessionEventSeq,
} from './session-events'

function event(seq: number): BridgeEvent {
  return {
    id: `evt-${seq}`,
    seq,
    sessionId: 'session-1',
    provider: 'codex',
    kind: 'message.chunk',
    createdAt: '2026-04-23T00:00:00Z',
    role: 'assistant',
    text: `chunk-${seq}`,
  }
}

test('appendSessionEvent ignores duplicate or out-of-order events', () => {
  const current = [event(1), event(2)]

  assert.equal(appendSessionEvent(current, event(2)), current)
  assert.equal(appendSessionEvent(current, event(1)), current)
})

test('appendSessionEvent retains only the newest bounded window', () => {
  let current: BridgeEvent[] = []

  for (let seq = 1; seq <= MAX_SESSION_EVENTS + 2; seq += 1) {
    current = appendSessionEvent(current, event(seq))
  }

  assert.equal(current.length, MAX_SESSION_EVENTS)
  assert.equal(current[0]?.seq, 3)
  assert.equal(current.at(-1)?.seq, MAX_SESSION_EVENTS + 2)
})

test('resolveLastSessionEventSeq prefers the newest cached sequence', () => {
  assert.equal(resolveLastSessionEventSeq([], 7), 7)
  assert.equal(resolveLastSessionEventSeq([event(3), event(9)], 4), 9)
  assert.equal(resolveLastSessionEventSeq([event(3), event(9)], 12), 12)
})
