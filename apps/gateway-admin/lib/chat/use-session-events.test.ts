import test from 'node:test'
import assert from 'node:assert/strict'

import type { BridgeEvent } from '@/lib/acp/types'
import { buildSessionEventsUrl, consumeSessionEventBuffer } from './use-session-events'

function event(seq: number, overrides: Partial<BridgeEvent> = {}): BridgeEvent {
  return {
    id: `evt-${seq}`,
    seq,
    sessionId: 'session-1',
    provider: 'codex',
    kind: 'message.chunk',
    createdAt: '2026-04-23T00:00:00Z',
    role: 'assistant',
    text: `chunk-${seq}`,
    ...overrides,
  }
}

function sseFrame(value: BridgeEvent) {
  return `data: ${JSON.stringify(value)}\n\n`
}

test('buildSessionEventsUrl includes the current resume checkpoint and subscribe ticket', () => {
  assert.equal(
    buildSessionEventsUrl('/v1/acp', 'session-1', 7, 'ticket:abc/123', 'http://127.0.0.1:3000'),
    'http://127.0.0.1:3000/v1/acp/sessions/session-1/events?since=7&ticket=ticket%3Aabc%2F123',
  )
})

test('consumeSessionEventBuffer appends only new events and preserves trailing partial frames', () => {
  const thirdFrame = sseFrame(event(3))
  const splitIndex = thirdFrame.length - 8
  const firstPass = consumeSessionEventBuffer(
    sseFrame(event(1)) + sseFrame(event(1, { text: 'duplicate' })) + sseFrame(event(2)) + thirdFrame.slice(0, splitIndex),
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

test('consumeSessionEventBuffer tolerates SSE metadata lines and canonical browser event kinds', () => {
  const payload = [
    ': keepalive',
    'event: acp-event',
    'id: evt-4',
    'data: {"id":"evt-4","seq":4,"sessionId":"session-1","provider":"codex",',
    'data: "kind":"status","createdAt":"2026-04-23T00:00:00Z","title":"Prompt completed","status":"completed"}',
    '',
    sseFrame(event(3, { text: 'stale duplicate' })).trimEnd(),
    '',
    'data: {"id":"evt-5","seq":5,"sessionId":"session-1","provider":"codex","kind":"error","createdAt":"2026-04-23T00:00:00Z","title":"Provider error","text":"Prompt failed","status":"failed"}',
    '',
    '',
  ].join('\n')

  const consumed = consumeSessionEventBuffer(payload, 3)

  assert.deepEqual(
    consumed.events.map((entry) => [entry.seq, entry.kind, entry.title, entry.status]),
    [
      [4, 'status', 'Prompt completed', 'completed'],
      [5, 'error', 'Provider error', 'failed'],
    ],
  )
  assert.equal(consumed.lastSeq, 5)
  assert.equal(consumed.buffer, '')
})

test('consumeSessionEventBuffer skips malformed and unknown payloads without appending undefined events', () => {
  const payload = [
    'data: not-json',
    '',
    'data: {"seq":6,"kind":"message_chunk"}',
    '',
    'data: {"id":"evt-7","seq":7,"session_id":"session-1","created_at":"2026-04-23T00:00:00Z","kind":"message_chunk","role":"assistant","text":"hello","message_id":"msg-1"}',
    '',
    '',
  ].join('\n')

  const consumed = consumeSessionEventBuffer(payload, 5)

  assert.deepEqual(consumed.events.map((entry) => [entry.seq, entry.kind, entry.text]), [
    [7, 'message.chunk', 'hello'],
  ])
  assert.equal(consumed.lastSeq, 7)
  assert.equal(consumed.buffer, '')
})
