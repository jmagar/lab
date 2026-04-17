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

  const init = logsRequestInit('logs.search', { query: {} })

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
