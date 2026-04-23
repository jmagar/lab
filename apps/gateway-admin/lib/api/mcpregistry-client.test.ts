import test from 'node:test'
import assert from 'node:assert/strict'

import { listServers } from './mcpregistry-client.ts'

test('listServers uses same-origin credentialed requests without browser bearer headers', async () => {
  const originalFetch = globalThis.fetch

  try {
    globalThis.fetch = (async (input, init) => {
      assert.equal(String(input), '/v0.1/servers')
      assert.equal(init?.credentials, 'include')
      assert.equal(new Headers(init?.headers).get('Authorization'), null)

      return new Response(
        JSON.stringify({
          servers: [],
          next_cursor: null,
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      )
    }) as typeof fetch

    const response = await listServers({})
    assert.equal(response.servers.length, 0)
    assert.equal(response.metadata.nextCursor, null)
  } finally {
    globalThis.fetch = originalFetch
  }
})
