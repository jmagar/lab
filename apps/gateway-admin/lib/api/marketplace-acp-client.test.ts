import test from 'node:test'
import assert from 'node:assert/strict'

import { installAcpAgent } from './marketplace-client'

test('installAcpAgent sends backend agent install parameter names', async () => {
  const originalFetch = globalThis.fetch

  try {
    globalThis.fetch = (async (_input, init) => {
      const body = JSON.parse(String(init?.body))
      assert.equal(body.action, 'agent.install')
      assert.deepEqual(body.params, {
        id: 'codex-acp',
        node_ids: ['local'],
        confirm: true,
      })

      return new Response(
        JSON.stringify({
          agent_id: 'codex-acp',
          results: [{ node_id: 'local', ok: true, result: {} }],
        }),
        { status: 200 },
      )
    }) as typeof fetch

    const result = await installAcpAgent({
      agent_id: 'codex-acp',
      device_ids: ['local'],
      scope: 'global',
    })

    assert.deepEqual(result.results, [{ device_id: 'local', ok: true }])
  } finally {
    globalThis.fetch = originalFetch
  }
})
