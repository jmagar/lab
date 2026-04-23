import type { Marketplace, Plugin, Artifact, ArtifactLang, MarketplaceSource } from '../types/marketplace.js'
import { marketplaceActionUrl } from './gateway-config.ts'
import { performServiceAction, type ServiceActionError } from './service-action-client.ts'
import type {
  DeployPluginWorkspacePreviewResult,
  DeployPluginWorkspaceResult,
  PluginWorkspace,
  SavePluginWorkspaceFileInput,
  SavePluginWorkspaceFileResult,
} from '../editor/types.js'

export class MarketplaceApiError extends Error implements ServiceActionError {
  status: number
  code?: string

  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'MarketplaceApiError'
    this.status = status
    this.code = code
  }
}

async function marketplaceAction<T>(
  action: string,
  params: object,
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, MarketplaceApiError>({
    action,
    params,
    signal,
    serviceLabel: 'Marketplace',
    url: marketplaceActionUrl(),
    createError: (message, status, code) => new MarketplaceApiError(message, status, code),
  })
}

export function detectArtifactLang(path: string): ArtifactLang {
  const fileName = path.split('/').pop() ?? path
  if (path.endsWith('.json')) return 'json'
  if (path.endsWith('.yaml') || path.endsWith('.yml')) return 'yaml'
  if (path.endsWith('.md')) return 'markdown'
  if (path.endsWith('.sh') || path.endsWith('.bash') || fileName === '.bashrc' || fileName === '.zshrc') return 'bash'
  if (path.endsWith('.toml')) return 'toml'
  return 'text'
}

function normalizeMarketplace(raw: Marketplace): Marketplace {
  const source = raw.source as MarketplaceSource
  const githubOwner = source === 'github'
    ? (raw.githubOwner ?? raw.ghUser ?? raw.owner) || undefined
    : undefined
  const repository = raw.repository ?? raw.repo
  const remoteUrl = raw.remoteUrl ?? raw.url
  const localPath = raw.localPath ?? raw.path
  const description = raw.description ?? raw.desc ?? ''
  const autoUpdateEnabled = raw.autoUpdateEnabled ?? raw.autoUpdate ?? false
  const pluginCount = raw.pluginCount ?? raw.totalPlugins ?? 0
  const lastUpdatedAt = raw.lastUpdatedAt ?? raw.lastUpdated ?? ''

  return {
    ...raw,
    source,
    githubOwner,
    repository,
    remoteUrl,
    localPath,
    description,
    autoUpdateEnabled,
    pluginCount,
    lastUpdatedAt,
    ghUser: githubOwner,
    repo: repository,
    url: remoteUrl,
    path: localPath,
    desc: description,
    autoUpdate: autoUpdateEnabled,
    totalPlugins: pluginCount,
    lastUpdated: lastUpdatedAt,
  }
}

function normalizePlugin(raw: Plugin): Plugin {
  const marketplaceId = raw.marketplaceId ?? raw.mkt
  const version = raw.version ?? raw.ver
  const description = raw.description ?? raw.desc ?? ''

  return {
    ...raw,
    marketplaceId,
    version,
    description,
    mkt: marketplaceId,
    ver: version,
    desc: description,
  }
}

export async function fetchMarketplaces(signal?: AbortSignal): Promise<Marketplace[]> {
  const marketplaces = await marketplaceAction<Marketplace[]>('sources.list', {}, signal)
  return marketplaces.map(normalizeMarketplace)
}

export async function fetchPlugins(signal?: AbortSignal): Promise<Plugin[]> {
  const plugins = await marketplaceAction<Plugin[]>('plugins.list', {}, signal)
  return plugins.map(normalizePlugin)
}

export async function getInstalledPluginIds(signal?: AbortSignal): Promise<Set<string>> {
  const plugins = await fetchPlugins(signal)
  return new Set(plugins.filter(p => p.installed).map(p => p.id))
}

export async function getArtifacts(pluginId: string, signal?: AbortSignal): Promise<Artifact[]> {
  const artifacts = await marketplaceAction<Artifact[]>('plugin.artifacts', { id: pluginId }, signal)
  return artifacts.map((artifact) => ({ ...artifact }))
}

export async function installPlugin(pluginId: string, signal?: AbortSignal): Promise<void> {
  await marketplaceAction<unknown>('plugin.install', { id: pluginId }, signal)
}

export async function uninstallPlugin(pluginId: string, signal?: AbortSignal): Promise<void> {
  await marketplaceAction<unknown>('plugin.uninstall', { id: pluginId, confirm: true }, signal)
}

export async function getPluginWorkspace(pluginId: string, signal?: AbortSignal): Promise<PluginWorkspace> {
  return marketplaceAction<PluginWorkspace>('plugin.workspace', { id: pluginId }, signal)
}

export async function savePluginWorkspaceFile(
  input: SavePluginWorkspaceFileInput,
  signal?: AbortSignal,
): Promise<SavePluginWorkspaceFileResult> {
  return marketplaceAction<SavePluginWorkspaceFileResult>(
    'plugin.save',
    { id: input.pluginId, path: input.path, content: input.content },
    signal,
  )
}

export async function deployPluginWorkspace(
  pluginId: string,
  signal?: AbortSignal,
): Promise<DeployPluginWorkspaceResult> {
  return marketplaceAction<DeployPluginWorkspaceResult>(
    'plugin.deploy',
    { id: pluginId, confirm: true },
    signal,
  )
}

export async function previewPluginWorkspaceDeploy(
  pluginId: string,
  signal?: AbortSignal,
): Promise<DeployPluginWorkspacePreviewResult> {
  return marketplaceAction<DeployPluginWorkspacePreviewResult>(
    'plugin.deploy.preview',
    { id: pluginId },
    signal,
  )
}

export async function addMarketplace(
  input: { repo?: string; url?: string; name?: string; autoUpdate: boolean },
  signal?: AbortSignal,
): Promise<Marketplace> {
  const params: Record<string, unknown> = {}
  if (input.repo) params.repo = input.repo
  else if (input.url) params.url = input.url
  else {
    throw new MarketplaceApiError(
      'addMarketplace requires either `repo` or `url`',
      400,
      'missing_param',
    )
  }
  await marketplaceAction<unknown>('sources.add', params, signal)
  const sources = await fetchMarketplaces(signal)
  const target = input.repo ?? input.url
  const found = sources.find(m => m.repo === target || m.url === target || m.id === target)
  if (found) return found
  return normalizeMarketplace({
    id: input.repo ?? input.url ?? `custom-${Date.now()}`,
    name: input.name ?? input.repo ?? input.url ?? 'Custom',
    owner: input.repo?.split('/')[0] ?? '',
    description: '',
    autoUpdateEnabled: input.autoUpdate,
    pluginCount: 0,
    lastUpdatedAt: new Date().toISOString(),
    githubOwner: input.repo?.split('/')[0] ?? undefined,
    repo: input.repo,
    url: input.url,
    source: input.repo ? 'github' : 'git',
    desc: '',
    autoUpdate: input.autoUpdate,
    totalPlugins: 0,
    lastUpdated: new Date().toISOString(),
  })
}
