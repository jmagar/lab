import test from 'node:test'
import assert from 'node:assert/strict'

import {
  getPluginWorkspace,
  savePluginWorkspaceFile,
  deployPluginWorkspace,
  previewPluginWorkspaceDeploy,
  MarketplaceApiError,
} from './marketplace-client'

test('loads workspace files for a plugin', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = async () => new Response(JSON.stringify({ pluginId: 'demo@market', files: [] }), { status: 200 })

  try {
    const workspace = await getPluginWorkspace('demo@market')
    assert.equal(workspace.pluginId, 'demo@market')
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('saves an edited file through the workspace API', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = async () => new Response(JSON.stringify({ savedAt: '2026-04-22T00:00:00Z' }), { status: 200 })

  try {
    const result = await savePluginWorkspaceFile({ pluginId: 'demo@market', path: 'plugin.json', content: '{}' })
    assert.equal(result.savedAt, '2026-04-22T00:00:00Z')
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('deploys a plugin workspace through the deploy API', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = async () => new Response(JSON.stringify({ ok: true, changed: ['plugin.json'], skipped: [], removed: [], failed: [] }), { status: 200 })

  try {
    const result = await deployPluginWorkspace('demo@market')
    assert.deepEqual(result.changed, ['plugin.json'])
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('loads a deploy preview for a plugin workspace', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = async () => new Response(JSON.stringify({ changed: ['plugin.json'], skipped: ['README.md'], removed: ['old.txt'] }), { status: 200 })

  try {
    const result = await previewPluginWorkspaceDeploy('demo@market')
    assert.deepEqual(result.changed, ['plugin.json'])
    assert.deepEqual(result.removed, ['old.txt'])
  } finally {
    globalThis.fetch = originalFetch
  }
})

test('surfaces structured save and deploy errors', async () => {
  const originalFetch = globalThis.fetch
  globalThis.fetch = async () => new Response(JSON.stringify({ kind: 'invalid_param', message: 'bad request' }), { status: 422 })

  try {
    await assert.rejects(
      () => savePluginWorkspaceFile({ pluginId: 'demo@market', path: 'plugin.json', content: '{}' }),
      (error) => error instanceof MarketplaceApiError && error.code === 'invalid_param',
    )
  } finally {
    globalThis.fetch = originalFetch
  }
})
