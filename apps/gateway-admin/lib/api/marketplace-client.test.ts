import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import {
  fetchMarketplaces,
  fetchPlugins,
  getInstalledPluginIds,
  detectArtifactLang,
  getArtifacts,
} from './marketplace-client.js'

describe('fetchMarketplaces', () => {
  it('returns non-empty array', async () => {
    const result = await fetchMarketplaces()
    assert.ok(Array.isArray(result))
    assert.ok(result.length > 0)
  })

  it('every entry has required fields', async () => {
    const result = await fetchMarketplaces()
    for (const m of result) {
      assert.ok(typeof m.id === 'string')
      assert.ok(typeof m.name === 'string')
      assert.ok(['github', 'git', 'local'].includes(m.source))
    }
  })
})

describe('fetchPlugins', () => {
  it('returns non-empty array', async () => {
    const result = await fetchPlugins()
    assert.ok(result.length > 0)
  })

  it('every plugin has a valid mkt reference', async () => {
    const [plugins, marketplaces] = await Promise.all([fetchPlugins(), fetchMarketplaces()])
    const mktIds = new Set(marketplaces.map(m => m.id))
    for (const p of plugins) {
      assert.ok(mktIds.has(p.mkt), `plugin ${p.id} references unknown mkt ${p.mkt}`)
    }
  })

  it('plugin id is namespaced as name@mkt', async () => {
    const plugins = await fetchPlugins()
    for (const p of plugins) {
      assert.ok(p.id.includes('@'), `expected id to contain @, got: ${p.id}`)
    }
  })
})

describe('getInstalledPluginIds', () => {
  it('returns a Set', async () => {
    const result = await getInstalledPluginIds()
    assert.ok(result instanceof Set)
  })
})

describe('detectArtifactLang', () => {
  it('detects json', () => { assert.equal(detectArtifactLang('plugin.json'), 'json') })
  it('detects yaml', () => { assert.equal(detectArtifactLang('agent.yaml'), 'yaml') })
  it('detects yaml for yml', () => { assert.equal(detectArtifactLang('agent.yml'), 'yaml') })
  it('detects markdown', () => { assert.equal(detectArtifactLang('README.md'), 'markdown') })
  it('detects bash for .sh', () => { assert.equal(detectArtifactLang('setup.sh'), 'bash') })
  it('detects bash for extensionless', () => { assert.equal(detectArtifactLang('Makefile'), 'bash') })
  it('detects toml', () => { assert.equal(detectArtifactLang('config.toml'), 'toml') })
  it('falls back to text', () => { assert.equal(detectArtifactLang('notes.txt'), 'text') })
})

describe('getArtifacts', () => {
  it('returns array for known plugin', async () => {
    const plugins = await fetchPlugins()
    const first = plugins[0]
    const artifacts = getArtifacts(first.id)
    assert.ok(Array.isArray(artifacts))
  })

  it('returns empty array for unknown id', () => {
    const artifacts = getArtifacts('unknown@nowhere')
    assert.deepEqual(artifacts, [])
  })

  it('every artifact has a plugin.json', async () => {
    const plugins = await fetchPlugins()
    const withArtifacts = plugins.filter(p => getArtifacts(p.id).length > 0)
    for (const p of withArtifacts) {
      const arts = getArtifacts(p.id)
      assert.ok(arts.some(a => a.path === 'plugin.json'), `${p.id} missing plugin.json`)
    }
  })
})
