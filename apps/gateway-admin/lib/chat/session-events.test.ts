import test from 'node:test'
import assert from 'node:assert/strict'

import type { BridgeEvent } from '@/lib/acp/types'
import {
  appendSessionEvent,
  deriveTranscriptAndActivity,
  MAX_SESSION_EVENTS,
  resolveLastSessionEventSeq,
} from './session-events'

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

test('deriveTranscriptAndActivity preserves browser tool, permission, plan, and status semantics', () => {
  const derived = deriveTranscriptAndActivity([
    event(1, { role: 'user', messageId: 'user-1', text: 'Review the browser contract.' }),
    event(2, { role: 'thinking', text: 'Inspecting the active session stream.' }),
    event(3, { role: 'assistant', messageId: 'assistant-1', text: 'I am checking the browser stream.' }),
    event(4, {
      kind: 'tool.call',
      title: 'Read Cargo.toml',
      toolCallId: 'tool-1',
      toolKind: 'read' as never,
      status: 'pending',
      rawInput: { path: 'Cargo.toml' },
      locations: ['/home/jmagar/workspace/lab/Cargo.toml'],
    }),
    event(5, {
      kind: 'tool.update',
      title: 'Read Cargo.toml',
      toolCallId: 'tool-1',
      toolKind: 'read' as never,
      status: 'completed',
      rawOutput: { ok: true },
      locations: ['/home/jmagar/workspace/lab/Cargo.toml'],
    }),
    event(6, {
      kind: 'plan',
      title: 'Execution plan updated',
      plan: [{ content: 'Expand browser tests', priority: 'high', status: 'in_progress' }],
    }),
    event(7, {
      kind: 'permission.requested',
      title: 'Read ~/.ssh/config',
      toolCallId: 'tool-1',
      permissionOptions: [{ optionId: 'allow_once', name: 'Allow once', kind: 'allow_once' }],
      status: 'requested',
    }),
    event(8, {
      kind: 'permission.resolved',
      title: 'Read ~/.ssh/config',
      toolCallId: 'tool-1',
      permissionSelection: 'allow_once',
      status: 'resolved',
    }),
    event(9, {
      kind: 'usage',
      title: 'Usage updated',
      usage: { used: 12, size: 256 } as never,
    }),
    event(10, {
      kind: 'mode',
      title: 'Agent mode updated',
      currentMode: { slug: 'review' } as never,
    }),
    event(11, {
      kind: 'config',
      title: 'Configuration options updated',
      configUpdate: { configOptions: [{ key: 'temperature', value: 0.2 }] } as never,
    }),
    event(12, {
      kind: 'commands',
      title: 'Available commands updated',
      commands: [{ name: 'cancel' }] as never,
    }),
    event(13, {
      kind: 'session.info',
      title: 'Session info updated',
      sessionInfo: { title: 'Renamed session' } as never,
    }),
    event(14, {
      kind: 'status',
      title: 'Prompt started',
      status: 'running',
      text: 'Review the browser contract.',
    }),
    event(15, {
      kind: 'status',
      title: 'Prompt completed',
      status: 'completed',
    }),
    event(16, {
      kind: 'error',
      title: 'Provider error',
      status: 'failed',
      text: 'Prompt failed',
    }),
  ])

  assert.equal(derived.messages.length, 2)
  assert.equal(derived.messages[0]?.role, 'user')
  assert.equal(derived.messages[0]?.text, 'Review the browser contract.')

  const assistant = derived.messages[1]!
  assert.equal(assistant.role, 'assistant')
  assert.equal(assistant.text, 'I am checking the bridge.')
  assert.deepEqual(assistant.thoughts, ['Inspecting the active session stream.'])
  assert.equal(assistant.isStreaming, false)
  assert.deepEqual(assistant.toolCalls, [
    {
      id: 'tool-1',
      title: 'Read Cargo.toml',
      status: 'completed',
      kind: 'read',
      input: { path: 'Cargo.toml' },
      output: { ok: true },
      content: null,
      locations: ['/home/jmagar/workspace/lab/Cargo.toml'],
    },
  ])

  assert.deepEqual(
    derived.activity.map((entry) => entry.kind),
    [
      'message.chunk',
      'tool.call',
      'tool.update',
      'plan',
      'permission.requested',
      'permission.resolved',
      'usage',
      'mode',
      'config',
      'commands',
      'session.info',
      'status',
      'status',
      'error',
    ],
  )
  assert.equal(derived.activity.find((entry) => entry.kind === 'permission.resolved')?.permissionSelection, 'allow_once')
  assert.equal(derived.activity.find((entry) => entry.kind === 'session.info')?.sessionInfo?.title, 'Renamed session')
  assert.equal(derived.activity.find((entry) => entry.kind === 'usage')?.usage?.used, 12)
})
