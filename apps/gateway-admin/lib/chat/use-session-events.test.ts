import test from 'node:test'
import assert from 'node:assert/strict'

import type { BridgeEvent } from '@/lib/acp/types'
import { buildSessionEventsUrl, consumeSessionEventBuffer } from './use-session-events'

function event(seq: number, text = `chunk-${seq}`): BridgeEvent {
  return {
    id: `evt-${seq}`,
    seq,
    sessionId: 'session-1',
    provider: 'codex',
    kind: 'message.chunk',
    createdAt: '2026-04-23T00:00:00Z',
    role: 'assistant',
    text,
  }
}

function sseFrame(value: BridgeEvent) {
  return `data: ${JSON.stringify(value)}\n\n`
}

test('buildSessionEventsUrl includes the current resume checkpoint', () => {
  assert.equal(
    buildSessionEventsUrl('/v1/acp', 'session-1', 7, 'http://127.0.0.1:3000'),
    'http://127.0.0.1:3000/v1/acp/sessions/session-1/events?since=7',
  )
})

test('consumeSessionEventBuffer appends only new events and preserves trailing partial frames', () => {
  const thirdFrame = sseFrame(event(3))
  const splitIndex = thirdFrame.length - 8
  const firstPass = consumeSessionEventBuffer(
    sseFrame(event(1)) + sseFrame(event(1, 'duplicate')) + sseFrame(event(2)) + thirdFrame.slice(0, splitIndex),
    0,
  )

  assert.deepEqual(firstPass.events.map((entry) => entry.seq), [1, 2])
  assert.equal(firstPass.lastSeq, 2)
  assert.equal(firstPass.buffer, thirdFrame.slice(0, splitIndex))

  const secondPass = consumeSessionEventBuffer(
    firstPass.buffer + thirdFrame.slice(splitIndex),
    firstPass.lastSeq,
  )

  assert.deepEqual(secondPass.events.map((entry) => entry.seq), [3])
  assert.equal(secondPass.lastSeq, 3)
  assert.equal(secondPass.buffer, '')
})
