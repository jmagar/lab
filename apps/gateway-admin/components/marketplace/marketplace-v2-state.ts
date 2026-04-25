import type { Marketplace, MarketplaceRuntime, Plugin } from '@/lib/types/marketplace'
import type { AcpAgent, McpServer } from '@/lib/marketplace/types'

export type MarketplaceCatalogKind = 'plugin' | 'mcp_server' | 'acp_agent' | 'source'
export type MarketplaceCatalogLens = 'all' | 'installed' | 'plugins' | 'mcp_servers' | 'acp_agents' | 'sources'
export type MarketplaceInstallFacet = 'installed' | 'not_installed' | 'update_available' | 'builtin'
export type MarketplaceSort = 'name' | 'source' | 'installed' | 'updated'

export interface MarketplaceCatalogItem {
  id: string
  kind: MarketplaceCatalogKind
  name: string
  subtitle: string
  description: string
  version?: string
  sourceId?: string
  sourceName?: string
  distribution?: string
  ecosystem: string
  installed: boolean
  hasUpdate: boolean
  builtin: boolean
  updatedAt?: string
  tags: string[]
  raw: unknown
}

export interface MarketplaceCatalogFilterState {
  lens: MarketplaceCatalogLens
  search: string
  types: MarketplaceCatalogKind[]
  installStates: MarketplaceInstallFacet[]
  ecosystems: string[]
  sourceIds: string[]
  distributions: string[]
  sort: MarketplaceSort
}

export interface MarketplaceCatalogSummary {
  all: number
  installed: number
  plugins: number
  mcpServers: number
  acpAgents: number
  sources: number
  updates: number
}

interface BuildMarketplaceCatalogItemsInput {
  plugins: Plugin[]
  sources: Marketplace[]
  mcpServers: Array<McpServer | McpRegistryEnvelope>
  acpAgents: AcpAgent[]
}

interface McpPackage {
  registryType?: string
  identifier?: string
  transport?: { type?: string }
}

interface McpRemote {
  type?: string
}

interface McpRegistryEnvelope {
  server?: McpServer & {
    title?: string
    packages?: McpPackage[]
    remotes?: McpRemote[]
    repository?: { source?: string; url?: string }
    websiteUrl?: string
  }
  _meta?: {
    'io.modelcontextprotocol.registry/official'?: {
      updatedAt?: string
      publishedAt?: string
      isLatest?: boolean
      status?: string
    }
  }
}

const RUNTIME_LABELS: Record<MarketplaceRuntime, string> = {
  claude: 'Claude',
  codex: 'Codex',
  gemini: 'Gemini',
}

function sourceDisplayName(source: Marketplace): string {
  return source.name || source.id
}

function sourceSubtitle(source: Marketplace): string {
  if (source.source === 'github') return source.repository ?? source.repo ?? source.githubOwner ?? 'GitHub source'
  if (source.source === 'git') return source.remoteUrl ?? source.url ?? 'Git source'
  return source.localPath ?? source.path ?? 'Local source'
}

function normalizeMcpServer(entry: McpServer | McpRegistryEnvelope): {
  server: McpRegistryEnvelope['server'] & McpServer
  updatedAt?: string
  raw: McpServer | McpRegistryEnvelope
} {
  const envelope = entry as McpRegistryEnvelope
  const server = (envelope.server ?? entry) as McpRegistryEnvelope['server'] & McpServer
  const officialMeta = envelope._meta?.['io.modelcontextprotocol.registry/official']
  return {
    server,
    updatedAt: officialMeta?.updatedAt ?? officialMeta?.publishedAt,
    raw: entry,
  }
}

function mcpDisplayName(server: McpRegistryEnvelope['server'] & McpServer): string {
  return server.title ?? server.name ?? server.packages?.[0]?.identifier ?? 'Unknown MCP server'
}

function mcpIdentifier(server: McpRegistryEnvelope['server'] & McpServer): string {
  return server.name ?? server.packages?.[0]?.identifier ?? mcpDisplayName(server)
}

function mcpSubtitle(server: McpRegistryEnvelope['server'] & McpServer): string {
  return server.package ?? server.packages?.[0]?.identifier ?? server.repository?.url ?? 'MCP Registry'
}

function mcpDistribution(server: McpRegistryEnvelope['server'] & McpServer): string {
  const packageName = server.package ?? server.packages?.[0]?.identifier ?? ''
  const registryType = server.packages?.[0]?.registryType?.toLowerCase()
  const transport = server.transport?.[0] ?? server.packages?.[0]?.transport?.type ?? server.remotes?.[0]?.type
  if (registryType === 'npm' || packageName.startsWith('@') || packageName.includes('npm')) return 'npm'
  if (registryType === 'pypi' || packageName.includes('uvx') || packageName.toLowerCase().includes('python')) return 'Python'
  if (registryType === 'oci') return 'Docker'
  return transport ?? 'MCP'
}

function agentDistribution(agent: AcpAgent): string {
  const distribution = agent.distribution ?? {}
  if (distribution.npx !== undefined) return 'npm'
  if (distribution.uvx !== undefined) return 'Python'
  if (distribution.binary !== undefined) return 'Binary'
  return 'ACP'
}

function pluginEcosystem(plugin: Plugin): string {
  return plugin.runtime ? RUNTIME_LABELS[plugin.runtime] : 'Generic'
}

export function buildMarketplaceCatalogItems({
  plugins,
  sources,
  mcpServers,
  acpAgents,
}: BuildMarketplaceCatalogItemsInput): MarketplaceCatalogItem[] {
  const sourceNames = new Map(sources.map((source) => [source.id, sourceDisplayName(source)]))

  return [
    ...plugins.map((plugin): MarketplaceCatalogItem => ({
      id: `plugin:${plugin.id}`,
      kind: 'plugin',
      name: plugin.name,
      subtitle: plugin.marketplaceId,
      description: plugin.description || plugin.desc || '',
      version: plugin.version || plugin.ver,
      sourceId: plugin.marketplaceId,
      sourceName: sourceNames.get(plugin.marketplaceId) ?? plugin.marketplaceId,
      distribution: 'Plugin',
      ecosystem: pluginEcosystem(plugin),
      installed: plugin.installed,
      hasUpdate: Boolean(plugin.hasUpdate),
      builtin: false,
      updatedAt: plugin.updatedAt,
      tags: plugin.tags ?? [],
      raw: plugin,
    })),
    ...mcpServers.map((entry): MarketplaceCatalogItem => {
      const { server, updatedAt, raw } = normalizeMcpServer(entry)
      return {
        id: `mcp:${mcpIdentifier(server)}`,
        kind: 'mcp_server',
        name: mcpDisplayName(server),
        subtitle: mcpSubtitle(server),
        description: server.description ?? '',
        version: server.version,
        distribution: mcpDistribution(server),
        ecosystem: 'MCP',
        installed: false,
        hasUpdate: false,
        builtin: false,
        updatedAt,
        tags: [server.transport?.[0], server.packages?.[0]?.registryType, server.remotes?.[0]?.type].filter(Boolean) as string[],
        raw,
      }
    }),
    ...acpAgents.map((agent): MarketplaceCatalogItem => ({
      id: `agent:${agent.id}`,
      kind: 'acp_agent',
      name: agent.name,
      subtitle: agent.id,
      description: agent.description ?? '',
      version: agent.version,
      distribution: agentDistribution(agent),
      ecosystem: 'ACP',
      installed: Boolean(agent.installed),
      hasUpdate: false,
      builtin: Boolean(agent.builtin),
      updatedAt: agent.installedAt,
      tags: [agent.license, ...(agent.authors ?? [])].filter(Boolean) as string[],
      raw: agent,
    })),
    ...sources.map((source): MarketplaceCatalogItem => ({
      id: `source:${source.id}`,
      kind: 'source',
      name: sourceDisplayName(source),
      subtitle: sourceSubtitle(source),
      description: source.description || source.desc || '',
      version: undefined,
      sourceId: source.id,
      sourceName: sourceDisplayName(source),
      distribution: source.source === 'github' ? 'GitHub' : source.source === 'git' ? 'git' : 'local',
      ecosystem: 'Source',
      installed: false,
      hasUpdate: false,
      builtin: false,
      updatedAt: source.lastUpdatedAt || source.lastUpdated,
      tags: [source.source, source.autoUpdateEnabled || source.autoUpdate ? 'auto-update' : 'manual'],
      raw: source,
    })),
  ]
}

export function marketplaceCatalogSummary(items: MarketplaceCatalogItem[]): MarketplaceCatalogSummary {
  return {
    all: items.length,
    installed: items.filter((item) => item.installed).length,
    plugins: items.filter((item) => item.kind === 'plugin').length,
    mcpServers: items.filter((item) => item.kind === 'mcp_server').length,
    acpAgents: items.filter((item) => item.kind === 'acp_agent').length,
    sources: items.filter((item) => item.kind === 'source').length,
    updates: items.filter((item) => item.hasUpdate).length,
  }
}

function matchesLens(item: MarketplaceCatalogItem, lens: MarketplaceCatalogLens): boolean {
  if (lens === 'all') return true
  if (lens === 'installed') return item.installed
  if (lens === 'plugins') return item.kind === 'plugin'
  if (lens === 'mcp_servers') return item.kind === 'mcp_server'
  if (lens === 'acp_agents') return item.kind === 'acp_agent'
  return item.kind === 'source'
}

function matchesInstallFacets(item: MarketplaceCatalogItem, facets: MarketplaceInstallFacet[]): boolean {
  if (facets.length === 0) return true

  const actual = new Set<MarketplaceInstallFacet>()
  if (item.installed) actual.add('installed')
  if (!item.installed && item.kind !== 'source') actual.add('not_installed')
  if (item.hasUpdate) actual.add('update_available')
  if (item.builtin) actual.add('builtin')

  return facets.some((facet) => actual.has(facet))
}

function matchesSearch(item: MarketplaceCatalogItem, search: string): boolean {
  const normalized = search.trim().toLowerCase()
  if (!normalized) return true
  return [
    item.name,
    item.subtitle,
    item.description,
    item.sourceName ?? '',
    item.distribution ?? '',
    item.ecosystem,
    ...item.tags,
  ]
    .join(' ')
    .toLowerCase()
    .includes(normalized)
}

function matchesAny<T extends string>(selected: T[], actual?: T): boolean {
  return selected.length === 0 || (actual !== undefined && selected.includes(actual))
}

export function filterMarketplaceCatalogItems(
  items: MarketplaceCatalogItem[],
  state: MarketplaceCatalogFilterState,
): MarketplaceCatalogItem[] {
  return items.filter((item) => {
    if (!matchesLens(item, state.lens)) return false
    if (!matchesSearch(item, state.search)) return false
    if (!matchesAny(state.types, item.kind)) return false
    if (!matchesInstallFacets(item, state.installStates)) return false
    if (!matchesAny(state.ecosystems, item.ecosystem)) return false
    if (!matchesAny(state.sourceIds, item.sourceId)) return false
    if (!matchesAny(state.distributions, item.distribution)) return false
    return true
  })
}

export function sortMarketplaceCatalogItems(
  items: MarketplaceCatalogItem[],
  sort: MarketplaceSort,
): MarketplaceCatalogItem[] {
  return [...items].sort((left, right) => {
    if (sort === 'installed') {
      if (left.installed !== right.installed) return left.installed ? -1 : 1
      if (left.hasUpdate !== right.hasUpdate) return left.hasUpdate ? -1 : 1
    }

    if (sort === 'updated') {
      const leftTime = new Date(left.updatedAt ?? 0).getTime()
      const rightTime = new Date(right.updatedAt ?? 0).getTime()
      if (leftTime !== rightTime) return rightTime - leftTime
    }

    if (sort === 'source') {
      const bySource = (left.sourceName ?? left.sourceId ?? left.subtitle).localeCompare(
        right.sourceName ?? right.sourceId ?? right.subtitle,
        undefined,
        { sensitivity: 'base' },
      )
      if (bySource !== 0) return bySource
    }

    return left.name.localeCompare(right.name, undefined, { sensitivity: 'base' })
  })
}
