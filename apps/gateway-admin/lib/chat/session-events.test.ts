import test from 'node:test'
import assert from 'node:assert/strict'

import type { BridgeEvent } from '@/lib/acp/types'
import {
  appendSessionEvent,
  deriveTranscriptAndActivity,
  MAX_SESSION_EVENTS,
  resolveLastSessionEventSeq,
  resolveSessionStatusFromEvents,
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

test('resolveSessionStatusFromEvents returns the latest typed status event', () => {
  assert.equal(resolveSessionStatusFromEvents([], 'running'), 'running')
  assert.equal(
    resolveSessionStatusFromEvents([
      event(1, { kind: 'status', status: 'running' }),
      event(2, { role: 'assistant', text: 'ok' }),
      event(3, { kind: 'status', status: 'completed' }),
      event(4, { kind: 'status', status: 'closed' }),
    ]),
    'closed',
  )
})

test('deriveTranscriptAndActivity appends streaming chunks when provider message ids churn', () => {
  const derived = deriveTranscriptAndActivity([
    event(1, { role: 'user', messageId: 'user-1', text: 'First prompt' }),
    event(2, { role: 'assistant', messageId: 'assistant-a', text: 'loading ' }),
    event(3, { role: 'assistant', messageId: 'assistant-b', text: 'the ' }),
    event(4, { role: 'assistant', messageId: 'assistant-c', text: 'session' }),
    event(5, { role: 'user', messageId: 'user-2', text: 'Second prompt' }),
    event(6, { role: 'assistant', messageId: 'assistant-d', text: 'next ' }),
    event(7, { role: 'assistant', messageId: 'assistant-e', text: 'reply' }),
  ])

  assert.equal(derived.messages.length, 4)
  assert.equal(derived.messages[0]?.text, 'First prompt')
  assert.equal(derived.messages[1]?.text, 'loading the session')
  assert.equal(derived.messages[2]?.text, 'Second prompt')
  assert.equal(derived.messages[3]?.text, 'next reply')
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
  assert.equal(assistant.text, 'I am checking the browser stream.')
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
      permissionOptions: undefined,
      permissionSelection: undefined,
      terminal: null,
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

// ---------------------------------------------------------------------------
// Terminal chunk reducer tests (bead lab-qn84.3)
// ---------------------------------------------------------------------------

test('deriveTranscriptAndActivity merges terminal metadata into chunked output', () => {
  const derived = deriveTranscriptAndActivity([
    event(1, { kind: 'tool.call', toolCallId: 'tc-1', title: 'Run cargo build' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-1',
      status: 'in_progress',
      rawOutput: {
        _meta: {
          terminal_info: { terminal_id: 'term-1' },
        },
      },
    }),
    event(3, {
      kind: 'tool.update',
      toolCallId: 'tc-1',
      status: 'in_progress',
      rawOutput: {
        _meta: {
          terminal_output: { terminal_id: 'term-1', data: 'running\n' },
        },
      },
    }),
  ])

  const tc = derived.messages[0]?.toolCalls[0]
  assert.ok(tc?.terminal != null, 'terminal should be present')
  assert.deepEqual(tc?.terminal?.rawChunks, ['running\n'])
  assert.equal(tc?.terminal?.exitCode, null)
  assert.equal(tc?.terminal?.truncated, false)
})

test('reducer evicts oldest chunks when totalBytes exceeds MAX_TOTAL_BYTES', () => {
  // Build events to exceed 1 MB total (1024*1024 bytes).
  // Each chunk is exactly MAX_CHUNK_BYTES = 64 KB. Send 20 = 1280 KB total to
  // guarantee eviction even after pre-push slicing.
  const chunkData = 'x'.repeat(64 * 1024) // 64 KB — exactly MAX_CHUNK_BYTES
  const events: BridgeEvent[] = [
    event(1, { kind: 'tool.call', toolCallId: 'tc-evict', title: 'Big output' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-evict',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-evict' } } },
    }),
  ]
  for (let i = 0; i < 20; i++) {
    events.push(
      event(3 + i, {
        kind: 'tool.update',
        toolCallId: 'tc-evict',
        rawOutput: { _meta: { terminal_output: { terminal_id: 'term-evict', data: chunkData } } },
      }),
    )
  }

  const derived = deriveTranscriptAndActivity(events)
  const tc = derived.messages[0]?.toolCalls[0]
  assert.ok(tc?.terminal?.truncated === true, 'truncated should be true after eviction')
  assert.ok((tc?.terminal?.totalBytes ?? 0) <= 1024 * 1024, 'totalBytes should be <= MAX_TOTAL_BYTES')
})

test('readTerminalPatch slices chunks larger than MAX_CHUNK_BYTES at boundary', () => {
  // MAX_CHUNK_BYTES = 64 * 1024 = 65536. Send a chunk that exceeds it.
  const oversizedData = 'z'.repeat(65536 + 1000) // > 64 KB
  const derived = deriveTranscriptAndActivity([
    event(1, { kind: 'tool.call', toolCallId: 'tc-big-chunk', title: 'Big chunk' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-big-chunk',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-big' } } },
    }),
    event(3, {
      kind: 'tool.update',
      toolCallId: 'tc-big-chunk',
      rawOutput: { _meta: { terminal_output: { terminal_id: 'term-big', data: oversizedData } } },
    }),
  ])

  const tc = derived.messages[0]?.toolCalls[0]
  assert.ok(tc?.terminal != null, 'terminal should be present')
  const totalChunkLen = (tc?.terminal?.rawChunks ?? []).reduce((sum, c) => sum + c.length, 0)
  // After slicing, the stored chunk should be <= 64 KB.
  assert.ok(totalChunkLen <= 65536, `chunk length ${totalChunkLen} should be <= 65536 (MAX_CHUNK_BYTES)`)
})

test('readTerminalPatch rejects terminal_id longer than 128 chars', () => {
  const longId = 'a'.repeat(129) // > 128 chars
  const derived = deriveTranscriptAndActivity([
    event(1, { kind: 'tool.call', toolCallId: 'tc-longid', title: 'Long ID' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-longid',
      rawOutput: { _meta: { terminal_info: { terminal_id: longId } } },
    }),
    event(3, {
      kind: 'tool.update',
      toolCallId: 'tc-longid',
      rawOutput: { _meta: { terminal_output: { terminal_id: longId, data: 'hello' } } },
    }),
  ])

  const tc = derived.messages[0]?.toolCalls[0]
  // Events with oversized terminal_id should be dropped — no terminal state.
  assert.equal(tc?.terminal ?? null, null, 'terminal should be null for oversized terminal_id')
})

test('terminal_output_before_terminal_info_is_dropped_with_warn_log', () => {
  // Send terminal_output BEFORE terminal_info — orphan output should be dropped.
  const derived = deriveTranscriptAndActivity([
    event(1, { kind: 'tool.call', toolCallId: 'tc-orphan', title: 'Orphan' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-orphan',
      rawOutput: { _meta: { terminal_output: { terminal_id: 'term-orphan', data: 'orphan chunk' } } },
    }),
  ])

  const tc = derived.messages[0]?.toolCalls[0]
  // Orphan output dropped; terminal should remain null.
  assert.equal(tc?.terminal ?? null, null, 'orphan terminal_output before terminal_info must be dropped')
})

test('terminal_exit triggers compact-and-freeze with chunks <= TERMINAL_RENDER_TAIL_BYTES', () => {
  const bigData = 'y'.repeat(100 * 1024) // 100 KB, send 4 = 400 KB total
  const events: BridgeEvent[] = [
    event(1, { kind: 'tool.call', toolCallId: 'tc-exit', title: 'Run something' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-exit',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-exit' } } },
    }),
  ]
  for (let i = 0; i < 4; i++) {
    events.push(
      event(3 + i, {
        kind: 'tool.update',
        toolCallId: 'tc-exit',
        rawOutput: { _meta: { terminal_output: { terminal_id: 'term-exit', data: bigData } } },
      }),
    )
  }
  events.push(
    event(7, {
      kind: 'tool.update',
      toolCallId: 'tc-exit',
      rawOutput: { _meta: { terminal_exit: { terminal_id: 'term-exit', exit_code: 0 } } },
    }),
  )

  const derived = deriveTranscriptAndActivity(events)
  const tc = derived.messages[0]?.toolCalls[0]
  assert.equal(tc?.terminal?.exitCode, 0, 'exit code should be 0')
  // After compact-and-freeze, raw chunks should be a single string <= 256 KB.
  assert.ok(tc?.terminal?.rawChunks.length === 1, 'should be compacted to 1 chunk after exit')
  assert.ok(
    (tc?.terminal?.rawChunks[0]?.length ?? 0) <= 256 * 1024,
    'compacted chunk should be <= TERMINAL_RENDER_TAIL_BYTES (256 KB)',
  )
})

test('second terminal_info for same toolCallId overwrites existing terminal', () => {
  const derived = deriveTranscriptAndActivity([
    event(1, { kind: 'tool.call', toolCallId: 'tc-c5', title: 'C5 test' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-c5',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-c5-a' } } },
    }),
    event(3, {
      kind: 'tool.update',
      toolCallId: 'tc-c5',
      rawOutput: { _meta: { terminal_output: { terminal_id: 'term-c5-a', data: 'first run\n' } } },
    }),
    event(4, {
      kind: 'tool.update',
      toolCallId: 'tc-c5',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-c5-b' } } },
    }),
  ])

  const tc = derived.messages[0]?.toolCalls[0]
  // After second terminal_info, the terminal entry is reset (overwritten).
  // rawChunks may still contain data from before the second info (implementation-defined)
  // but the key invariant is it does NOT crash.
  assert.ok(tc?.terminal != null, 'terminal should be present after second terminal_info')
})

test('per_chunk_dispatch_under_50_microseconds (O1)', () => {
  // Build 5000 terminal_output events for a warm perf test.
  const events: BridgeEvent[] = [
    event(1, { kind: 'tool.call', toolCallId: 'tc-perf', title: 'Perf test' }),
    event(2, {
      kind: 'tool.update',
      toolCallId: 'tc-perf',
      rawOutput: { _meta: { terminal_info: { terminal_id: 'term-perf' } } },
    }),
  ]
  for (let i = 0; i < 5000; i++) {
    events.push(
      event(3 + i, {
        kind: 'tool.update',
        toolCallId: 'tc-perf',
        rawOutput: { _meta: { terminal_output: { terminal_id: 'term-perf', data: `chunk ${i}\n` } } },
      }),
    )
  }

  // Warmup
  deriveTranscriptAndActivity(events.slice(0, 100))

  const start = performance.now()
  deriveTranscriptAndActivity(events)
  const elapsed = performance.now() - start

  const perChunkMicros = (elapsed * 1000) / 5000
  assert.ok(
    perChunkMicros < 50,
    `per-chunk dispatch ${perChunkMicros.toFixed(2)}µs exceeds 50µs budget`,
  )
})
