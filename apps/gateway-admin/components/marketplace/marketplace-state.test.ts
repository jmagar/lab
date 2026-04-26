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
} from './marketplace-state'

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
    updatedAt: '2026-04-22T10:00:00Z',
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
    updatedAt: '2026-04-21T10:00:00Z',
    runtime: 'codex',
    components: [
      {
        kind: 'agents',
        path: 'agents/reviewer.md',
        name: 'Reviewer Agent',
        metadata: { description: 'Reviews repository changes.' },
      },
      {
        kind: 'skills',
        path: 'skills/tdd/SKILL.md',
        name: 'TDD Skill',
      },
      {
        kind: 'commands',
        path: 'commands/ship.md',
        name: 'Ship Command',
      },
    ],
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

const registryMcpServers = [
  {
    server: {
      name: 'github',
      description: 'GitHub MCP server',
      version: '0.4.1',
      packages: [{ registryType: 'npm', identifier: '@modelcontextprotocol/server-github' }],
    },
    _meta: {
      'io.modelcontextprotocol.registry/official': {
        updatedAt: '2026-04-24T10:00:00Z',
      },
    },
  },
  {
    server: {
      name: 'github',
      description: 'Older GitHub MCP server',
      version: '0.3.0',
      packages: [{ registryType: 'npm', identifier: '@modelcontextprotocol/server-github' }],
    },
    _meta: {
      'io.modelcontextprotocol.registry/official': {
        updatedAt: '2026-04-19T10:00:00Z',
        isLatest: false,
      },
    },
  },
  {
    server: {
      name: 'brave-search',
      description: 'Search MCP server',
      version: '0.2.0',
      packages: [{ registryType: 'npm', identifier: '@modelcontextprotocol/server-brave-search' }],
    },
    _meta: {
      'io.modelcontextprotocol.registry/official': {
        updatedAt: '2026-04-20T10:00:00Z',
      },
    },
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
    installedAt: '2026-04-23T10:00:00Z',
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
  sort: 'updated',
}

test('buildMarketplaceCatalogItems creates unified plugin mcp agent and source rows', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })

  assert.deepEqual(items.map((item) => item.kind), ['plugin', 'plugin', 'agent', 'skill', 'command', 'mcp_server', 'acp_agent', 'source'])
  assert.equal(items.find((item) => item.id === 'plugin:gateway-audit')?.ecosystem, 'Claude')
  assert.equal(items.find((item) => item.id === 'component:codex-helper:agents:agents/reviewer.md')?.kind, 'agent')
  assert.equal(items.find((item) => item.id === 'component:codex-helper:skills:skills/tdd/SKILL.md')?.kind, 'skill')
  assert.equal(items.find((item) => item.id === 'component:codex-helper:commands:commands/ship.md')?.kind, 'command')
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

test('buildMarketplaceCatalogItems dedupes MCP Registry versions by server identifier', () => {
  const items = buildMarketplaceCatalogItems({
    plugins: [],
    sources: [],
    mcpServers: registryMcpServers,
    acpAgents: [],
  })

  assert.deepEqual(items.map((item) => item.id), ['mcp:github', 'mcp:brave-search'])
  assert.equal(items.find((item) => item.id === 'mcp:github')?.version, '0.4.1')
  assert.equal(items.find((item) => item.id === 'mcp:github')?.updatedAt, '2026-04-24T10:00:00Z')
})

test('marketplaceCatalogSummary counts lenses from unified rows', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })
  const summary = marketplaceCatalogSummary(items)

  assert.deepEqual(summary, {
    all: 8,
    installed: 2,
    plugins: 2,
    agents: 1,
    skills: 1,
    commands: 1,
    mcpServers: 1,
    acpAgents: 1,
    sources: 1,
    updates: 1,
  })
})

test('marketplaceCatalogSummary does not count bundled components as plugins', () => {
  const installedPluginWithComponents: Plugin = {
    id: 'ops-pack',
    name: 'Ops Pack',
    marketplaceId: 'official',
    version: '1.0.0',
    description: 'Installed operator component bundle',
    tags: [],
    installed: true,
    components: [
      { kind: 'agents', path: 'agents/ops.md', name: 'Ops Agent' },
      { kind: 'skills', path: 'skills/ops/SKILL.md', name: 'Ops Skill' },
      { kind: 'commands', path: 'commands/ops.md', name: 'Ops Command' },
    ],
  }
  const items = buildMarketplaceCatalogItems({
    plugins: [installedPluginWithComponents],
    sources,
    mcpServers: [],
    acpAgents: [],
  })
  const summary = marketplaceCatalogSummary(items)

  assert.equal(summary.plugins, 1)
  assert.equal(summary.agents, 1)
  assert.equal(summary.skills, 1)
  assert.equal(summary.commands, 1)
  assert.deepEqual(
    items.filter((item) => item.installed).map((item) => item.kind),
    ['plugin', 'agent', 'skill', 'command'],
  )
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

test('filterMarketplaceCatalogItems treats plugin-distributed components as their own kinds', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })

  assert.deepEqual(
    filterMarketplaceCatalogItems(items, { ...baseFilters, lens: 'agents' }).map((item) => item.id),
    ['component:codex-helper:agents:agents/reviewer.md'],
  )
  assert.deepEqual(
    filterMarketplaceCatalogItems(items, { ...baseFilters, types: ['skill'] }).map((item) => item.id),
    ['component:codex-helper:skills:skills/tdd/SKILL.md'],
  )
  assert.deepEqual(
    filterMarketplaceCatalogItems(items, { ...baseFilters, search: 'ship' }).map((item) => item.id),
    ['component:codex-helper:commands:commands/ship.md'],
  )
})

test('filterMarketplaceCatalogItems treats sources as catalog items and facets', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })
  const sourceLens = filterMarketplaceCatalogItems(items, { ...baseFilters, lens: 'sources' })
  const sourceFacet = filterMarketplaceCatalogItems(items, { ...baseFilters, sourceIds: ['official'] })

  assert.deepEqual(sourceLens.map((item) => item.id), ['source:official'])
  assert.deepEqual(sourceFacet.map((item) => item.id), [
    'plugin:gateway-audit',
    'plugin:codex-helper',
    'component:codex-helper:agents:agents/reviewer.md',
    'component:codex-helper:skills:skills/tdd/SKILL.md',
    'component:codex-helper:commands:commands/ship.md',
    'source:official',
  ])
})

test('filterMarketplaceCatalogItems lets source filters target MCP Registry rows', () => {
  const items = buildMarketplaceCatalogItems({
    plugins,
    sources,
    mcpServers: registryMcpServers,
    acpAgents,
  })

  assert.deepEqual(
    filterMarketplaceCatalogItems(items, { ...baseFilters, sourceIds: ['mcp-registry'] }).map((item) => item.id),
    ['mcp:github', 'mcp:brave-search'],
  )
  assert.deepEqual(
    filterMarketplaceCatalogItems(items, { ...baseFilters, sourceIds: ['official'] }).map((item) => item.id),
    [
      'plugin:gateway-audit',
      'plugin:codex-helper',
      'component:codex-helper:agents:agents/reviewer.md',
      'component:codex-helper:skills:skills/tdd/SKILL.md',
      'component:codex-helper:commands:commands/ship.md',
      'source:official',
    ],
  )
})

test('sortMarketplaceCatalogItems supports install state and source sorting', () => {
  const items = buildMarketplaceCatalogItems({ plugins, sources, mcpServers, acpAgents })

  assert.deepEqual(
    sortMarketplaceCatalogItems(items, 'installed').map((item) => item.id).slice(0, 2),
    ['plugin:gateway-audit', 'agent:codex-cli'],
  )
  assert.deepEqual(
    sortMarketplaceCatalogItems(items, 'source').map((item) => item.id).slice(0, 2),
    ['agent:codex-cli', 'mcp:filesystem'],
  )
})

test('sortMarketplaceCatalogItems puts recently updated catalog entries first', () => {
  const items = buildMarketplaceCatalogItems({
    plugins,
    sources,
    mcpServers: registryMcpServers,
    acpAgents,
  })

  assert.deepEqual(
    sortMarketplaceCatalogItems(items, 'updated').map((item) => item.id).slice(0, 3),
    ['mcp:github', 'agent:codex-cli', 'plugin:gateway-audit'],
  )
})
