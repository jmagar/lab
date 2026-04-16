import test from 'node:test'
import assert from 'node:assert/strict'

import {
  __setBrowserSessionStateForTests,
  getBrowserSessionState,
  loadBrowserSession,
  logoutBrowserSession,
} from '../auth/session-store.ts'

test('loadBrowserSession stores authenticated payloads', async () => {
  globalThis.fetch = async () =>
    new Response(
      JSON.stringify({
        authenticated: true,
        user: {
          sub: 'browser-user',
          email: 'browser@example.com',
        },
        expires_at: 123,
        csrf_token: 'csrf-123',
      }),
      { status: 200 },
    ) as typeof fetch

  const state = await loadBrowserSession()
  assert.equal(state.status, 'authenticated')
  assert.equal(getBrowserSessionState().status, 'authenticated')
})

test('logoutBrowserSession resets local state after POST', async () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 123,
    csrfToken: 'csrf-123',
  })

  let requestInit: RequestInit | undefined
  globalThis.fetch = async (_input, init) => {
    requestInit = init
    return new Response(null, { status: 204 })
  }

  await logoutBrowserSession()
  assert.equal(requestInit?.method, 'POST')
  assert.equal(
    (requestInit?.headers as Record<string, string>)['x-csrf-token'],
    'csrf-123',
  )
  assert.deepEqual(getBrowserSessionState(), { status: 'unauthenticated' })
})
