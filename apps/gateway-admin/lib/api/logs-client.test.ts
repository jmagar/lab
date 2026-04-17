import test from 'node:test'
import assert from 'node:assert/strict'

import { __setBrowserSessionStateForTests } from '../auth/session-store.ts'
import { fetchLogStats, fetchLogs, logsActionUrl, logsRequestInit } from './logs-client.ts'
import { connectLogStream, logsStreamUrl } from './logs-stream.ts'

test('logsActionUrl uses the shared API base convention', () => {
  assert.equal(logsActionUrl('http://localhost:9911/v1/'), 'http://localhost:9911/v1/logs')
  assert.equal(logsStreamUrl('http://localhost:9911/v1/'), 'http://localhost:9911/v1/logs/stream')
})

test('logsRequestInit keeps credentialed session requests by default', () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 42,
    csrfToken: 'csrf-123',
  })

  const init = logsRequestInit('logs.search', { query: {} }, undefined, undefined, false)

  assert.equal(init.credentials, 'include')
  assert.equal((init.headers as Record<string, string>)['x-csrf-token'], 'csrf-123')
  assert.equal((init.headers as Record<string, string>).Authorization, undefined)
})

test('fetchLogs posts logs.search to the backend', async () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 42,
    csrfToken: 'csrf-123',
  })

  const originalFetch = globalThis.fetch
  try {
    globalThis.fetch = async (input, init) => {
      assert.equal(input, '/v1/logs')
      assert.equal(init?.method, 'POST')
      const body = JSON.parse(String(init?.body))
      assert.equal(body.action, 'logs.search')
      assert.deepEqual(body.params, { query: { subsystems: ['gateway'] } })
      return new Response(
        JSON.stringify({
          events: [{ event_id: 'evt-1', message: 'gateway event' }],
          next_cursor: null,
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      )
    }

    const result = await fetchLogs({ subsystems: ['gateway'] as const })
    assert.equal(result.events.length, 1)
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchLogStats posts logs.stats to the backend', async () => {
  const originalFetch = globalThis.fetch
  try {
    globalThis.fetch = async (_input, init) => {
      const body = JSON.parse(String(init?.body))
      assert.equal(body.action, 'logs.stats')
      return new Response(
        JSON.stringify({
          on_disk_bytes: 10,
          oldest_retained_ts: null,
          newest_retained_ts: null,
          total_event_count: 1,
          dropped_event_count: 0,
          retention: {
            max_age_days: 7,
            max_bytes: 1024,
          },
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      )
    }

    const result = await fetchLogStats()
    assert.equal(result.total_event_count, 1)
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('fetchLogs surfaces non-JSON error responses with status context', async () => {
  const originalFetch = globalThis.fetch
  try {
    globalThis.fetch = async () =>
      new Response('<html>bad gateway</html>', {
        status: 502,
        statusText: 'Bad Gateway',
        headers: { 'Content-Type': 'text/html' },
      })

    await assert.rejects(
      () => fetchLogs({ subsystems: ['gateway'] as const }),
      /502 Bad Gateway/,
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('connectLogStream refuses standalone bearer mode in v1', () => {
  assert.throws(
    () =>
      connectLogStream(
        { onEvent: () => {} },
        { token: 'dev-token', standaloneBearerAuth: true },
      ),
    /standalone bearer mode is not supported/,
  )
})

test('connectLogStream honors explicit standalone bearer opt-out and handles malformed frames', () => {
  const originalEventSource = globalThis.EventSource

  class MockEventSource {
    static readonly CONNECTING = 0
    static readonly OPEN = 1
    static readonly CLOSED = 2
    static lastCreated: MockEventSource | null = null

    readonly url: string
    readonly options: EventSourceInit
    readyState = MockEventSource.CONNECTING
    onopen: (() => void) | null = null
    onmessage: ((message: { data: string }) => void) | null = null
    onerror: (() => void) | null = null
    private readonly listeners = new Map<string, Array<(message: { data: string }) => void>>()

    constructor(url: string, options: EventSourceInit) {
      this.url = url
      this.options = options
      MockEventSource.lastCreated = this
    }

    addEventListener(type: string, listener: (message: { data: string }) => void) {
      const listeners = this.listeners.get(type) ?? []
      listeners.push(listener)
      this.listeners.set(type, listeners)
    }

    emit(type: string, data: string) {
      for (const listener of this.listeners.get(type) ?? []) {
        listener({ data })
      }
    }

    close() {
      this.readyState = MockEventSource.CLOSED
    }
  }

  globalThis.EventSource = MockEventSource as unknown as typeof EventSource

  try {
    const errorMessages: string[] = []
    let laggedCount: number | null = null

    const disconnect = connectLogStream(
      {
        onEvent: () => {
          throw new Error('unexpected event dispatch')
        },
        onLag: (skipped) => {
          laggedCount = skipped
        },
        onError: (message) => {
          errorMessages.push(message)
        },
      },
      { token: 'dev-token', standaloneBearerAuth: false, baseUrl: 'http://localhost:9911/v1/' },
    )

    const createdSource = MockEventSource.lastCreated
    assert.ok(createdSource)
    assert.equal(createdSource.url, 'http://localhost:9911/v1/logs/stream')
    assert.equal(createdSource.options.withCredentials, true)

    createdSource.onmessage?.({ data: '{not-json' })
    createdSource.emit('lag', '3')
    createdSource.onerror?.()
    createdSource.readyState = MockEventSource.CLOSED
    createdSource.onerror?.()

    assert.deepEqual(errorMessages, [
      'received malformed log event',
      'live stream disconnected',
    ])
    assert.equal(laggedCount, 3)

    disconnect()
    assert.equal(createdSource.readyState, MockEventSource.CLOSED)
  } finally {
    globalThis.EventSource = originalEventSource
  }
})
