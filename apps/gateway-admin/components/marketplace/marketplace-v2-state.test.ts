import test from 'node:test'
import assert from 'node:assert/strict'

import type { Marketplace, Plugin } from '@/lib/types/marketplace'
import type { AcpAgent, McpServer } from '@/lib/marketplace/types'
import {
  buildMarketplaceCatalogItems,
  filterMarketplaceCatalogItems,
  marketplaceCatalogSummary,
  sortMarketplaceCatalogItems,
  type MarketplaceCatalogFilterState,
} from './marketplace-v2-state'

const sources: Marketplace[] = [
  {
    id: 'official',
    name: 'Official',
    owner: 'Labby',
    description: 'Curated plugins',
    autoUpdateEnabled: true,
    pluginCount: 2,
    lastUpdatedAt: '2026-04-21T12:00:00Z',
    source: 'github',
    repository: 'labby/marketplace',
    githubOwner: 'labby',
  },
]

const plugins: Plugin[] = [
  {
    id: 'gateway-audit',
    name: 'Gateway Audit',
    marketplaceId: 'official',
    version: '1.0.0',
    description: 'Audit gateway exposure',
    tags: ['gateway'],
    installed: true,
    hasUpdate: true,
    runtime: 'claude',
  },
  {
    id: 'codex-helper',
    name: 'Codex Helper',
    marketplaceId: 'official',
    version: '0.2.0',
    description: 'Codex workflow helpers',
    tags: ['codex'],
    installed: false,
    runtime: 'codex',
  },
]

const mcpServers: McpServer[] = [
  {
    name: 'filesystem',
    description: 'Filesystem MCP server',
    version: '1.3.0',
    package: '@modelcontextprotocol/server-filesystem',
    transport: ['stdio'],
  },
]

const acpAgents: AcpAgent[] = [
  {
    id: 'codex-cli',
    name: 'Codex CLI',
    version: '0.5.0',
    description: 'Agent implementing ACP',
    distribution: { binary: {} },
    installed: true,
  },
]

const baseFilters: MarketplaceCatalogFilterState = {
  lens: 'all',
  search: '',
  types: [],
  installStates: [],
  ecosystems: [],
  sourceIds: [],
  distributions: [],
  sort: 'name',
}

test('buildMarketplaceCatalogItems creates unified plugin mcp agent and source rows', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })

  assert.deepEqual(items.map((item) => item.kind), ['plugin', 'plugin', 'mcp_server', 'acp_agent', 'source'])
  assert.equal(items.find((item) => item.id === 'plugin:gateway-audit')?.ecosystem, 'Claude')
  assert.equal(items.find((item) => item.id === 'mcp:filesystem')?.distribution, 'npm')
  assert.equal(items.find((item) => item.id === 'agent:codex-cli')?.ecosystem, 'ACP')
})

test('buildMarketplaceCatalogItems accepts MCP Registry server envelopes', () => {
  const items = buildMarketplaceCatalogItems({
    plugins: [],
    sources: [],
    mcpServers: [
      {
        server: {
          name: 'acme/widgets',
          title: 'Acme Widgets',
          description: 'Widget MCP server',
          version: '2.1.0',
          packages: [{ registryType: 'npm', identifier: '@acme/widgets-mcp', transport: { type: 'stdio' } }],
        },
        _meta: {
          'io.modelcontextprotocol.registry/official': {
            updatedAt: '2026-04-24T10:00:00Z',
            isLatest: true,
            status: 'active',
          },
        },
      },
    ],
    acpAgents: [],
  })

  assert.equal(items[0]?.id, 'mcp:acme/widgets')
  assert.equal(items[0]?.name, 'Acme Widgets')
  assert.equal(items[0]?.subtitle, '@acme/widgets-mcp')
  assert.equal(items[0]?.distribution, 'npm')
  assert.equal(items[0]?.updatedAt, '2026-04-24T10:00:00Z')
})

test('marketplaceCatalogSummary counts lenses from unified rows', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })
  const summary = marketplaceCatalogSummary(items)

  assert.deepEqual(summary, {
    all: 5,
    installed: 2,
    plugins: 2,
    mcpServers: 1,
    acpAgents: 1,
    sources: 1,
    updates: 1,
  })
})

test('filterMarketplaceCatalogItems combines lens search and facets', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })
  const filtered = filterMarketplaceCatalogItems(items, {
    ...baseFilters,
    lens: 'plugins',
    search: 'codex',
    installStates: ['not_installed'],
    ecosystems: ['Codex'],
  })

  assert.deepEqual(filtered.map((item) => item.id), ['plugin:codex-helper'])
})

test('filterMarketplaceCatalogItems treats sources as catalog items and facets', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })
  const sourceLens = filterMarketplaceCatalogItems(items, { ...baseFilters, lens: 'sources' })
  const sourceFacet = filterMarketplaceCatalogItems(items, { ...baseFilters, sourceIds: ['official'] })

  assert.deepEqual(sourceLens.map((item) => item.id), ['source:official'])
  assert.deepEqual(sourceFacet.map((item) => item.id), ['plugin:gateway-audit', 'plugin:codex-helper', 'source:official'])
})

test('sortMarketplaceCatalogItems supports install state and source sorting', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })

  assert.deepEqual(
    sortMarketplaceCatalogItems(items, 'installed').map((item) => item.id).slice(0, 2),
    ['plugin:gateway-audit', 'agent:codex-cli'],
  )
  assert.deepEqual(
    sortMarketplaceCatalogItems(items, 'source').map((item) => item.id).slice(0, 2),
    ['mcp:filesystem', 'agent:codex-cli'],
  )
})
