import test from 'node:test'
import assert from 'node:assert/strict'

import {
  __setBrowserSessionStateForTests,
  getBrowserSessionState,
  loadBrowserSession,
  logoutBrowserSession,
} from '../auth/session-store.ts'
import {
  hasApiTokenAuth,
  isStandaloneBearerAuthMode,
  shouldBypassBrowserSessionAuth,
} from '../auth/auth-mode.ts'

type FetchMock = typeof globalThis.fetch

test('loadBrowserSession stores authenticated payloads', async () => {
  globalThis.fetch = (async () =>
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
    )) as FetchMock

  const state = await loadBrowserSession()
  assert.equal(state.status, 'authenticated')
  assert.equal(getBrowserSessionState().status, 'authenticated')
})

test('loadBrowserSession falls back to unauthenticated when /auth/session fails', async () => {
  __setBrowserSessionStateForTests({ status: 'loading' })
  globalThis.fetch = (async () =>
    new Response('not found', {
      status: 401,
      headers: { 'content-type': 'text/plain' },
    })) as FetchMock

  const state = await loadBrowserSession()
  assert.deepEqual(state, { status: 'unauthenticated' })
  assert.deepEqual(getBrowserSessionState(), { status: 'unauthenticated' })
})

test('loadBrowserSession keeps backend failures distinct from auth failures', async () => {
  __setBrowserSessionStateForTests({ status: 'loading' })
  globalThis.fetch = (async () =>
    new Response('boom', {
      status: 500,
      headers: { 'content-type': 'text/plain' },
    })) as FetchMock

  const state = await loadBrowserSession()
  assert.deepEqual(state, {
    status: 'error',
    message: 'Unable to reach the authentication service. Try again.',
  })
  assert.deepEqual(getBrowserSessionState(), {
    status: 'error',
    message: 'Unable to reach the authentication service. Try again.',
  })
})

test('loadBrowserSession keeps network failures distinct from auth failures', async () => {
  __setBrowserSessionStateForTests({ status: 'loading' })
  globalThis.fetch = (async () => {
    throw new Error('socket hang up')
  }) as FetchMock

  const state = await loadBrowserSession()
  assert.deepEqual(state, {
    status: 'error',
    message: 'Unable to reach the authentication service. Try again.',
  })
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

test('logoutBrowserSession preserves local state when /auth/logout fails', async () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 123,
    csrfToken: 'csrf-123',
  })

  globalThis.fetch = (async () => new Response('boom', { status: 500 })) as FetchMock

  await assert.rejects(
    logoutBrowserSession(),
    /Failed to logout browser session/,
  )
  assert.equal(getBrowserSessionState().status, 'authenticated')
})

test('hasApiTokenAuth only enables bearer mode for non-empty tokens', () => {
  assert.equal(hasApiTokenAuth(undefined), false)
  assert.equal(hasApiTokenAuth(''), false)
  assert.equal(hasApiTokenAuth('   '), false)
  assert.equal(hasApiTokenAuth('dev-token'), true)
})

test('isStandaloneBearerAuthMode requires both a token and the explicit standalone override', () => {
  assert.equal(isStandaloneBearerAuthMode(undefined, undefined), false)
  assert.equal(isStandaloneBearerAuthMode('dev-token', undefined), false)
  assert.equal(isStandaloneBearerAuthMode(undefined, 'true'), false)
  assert.equal(isStandaloneBearerAuthMode('dev-token', 'false'), false)
  assert.equal(isStandaloneBearerAuthMode('dev-token', 'true'), true)
})

test('shouldBypassBrowserSessionAuth only bypasses hosted auth for mock mode or explicit standalone bearer mode', () => {
  assert.equal(shouldBypassBrowserSessionAuth(undefined, 'false', undefined), false)
  assert.equal(shouldBypassBrowserSessionAuth('dev-token', 'false', undefined), false)
  assert.equal(shouldBypassBrowserSessionAuth('dev-token', 'false', 'false'), false)
  assert.equal(shouldBypassBrowserSessionAuth('dev-token', 'false', 'true'), true)
  assert.equal(shouldBypassBrowserSessionAuth(undefined, 'true', undefined), true)
})
