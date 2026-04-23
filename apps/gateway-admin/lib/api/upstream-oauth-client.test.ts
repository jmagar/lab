import test from 'node:test'
import assert from 'node:assert/strict'

import { __setBrowserSessionStateForTests } from '../auth/session-store.ts'
import { upstreamOauthApi } from './upstream-oauth-client.ts'

test('upstreamOauthApi keeps hosted session auth and omits bearer headers', async () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 42,
    csrfToken: 'csrf-123',
  })

  const originalFetch = globalThis.fetch

  try {
    globalThis.fetch = (async (input, init) => {
      assert.equal(String(input), '/v1/gateway/oauth/upstreams')
      assert.equal(init?.credentials, 'include')
      const headers = new Headers(init?.headers)
      assert.equal(headers.get('Authorization'), null)
      assert.equal(headers.get('x-csrf-token'), 'csrf-123')

      return new Response(JSON.stringify([]), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    }) as typeof fetch

    assert.deepEqual(await upstreamOauthApi.listUpstreams(), [])
  } finally {
    globalThis.fetch = originalFetch
  }
})
