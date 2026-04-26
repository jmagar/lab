export type MarketplaceSource = 'github' | 'git' | 'local'

export interface Marketplace {
  id: string
  name: string
  owner: string
  description: string
  autoUpdateEnabled: boolean
  pluginCount: number
  lastUpdatedAt: string
  githubOwner?: string
  repository?: string
  remoteUrl?: string
  localPath?: string
  ghUser?: string
  repo?: string
  source: MarketplaceSource
  url?: string
  path?: string
  desc?: string
  autoUpdate?: boolean
  totalPlugins?: number
  lastUpdated?: string
}

export interface Plugin {
  id: string
  name: string
  marketplaceId: string
  version: string
  description: string
  mkt?: string
  ver?: string
  desc?: string
  tags: string[]
  installed: boolean
  hasUpdate?: boolean
  installedAt?: string
  updatedAt?: string
  runtime?: MarketplaceRuntime
  components?: PluginComponent[]
  installState?: PluginInstallState
  manifest?: PluginManifestSummary
}

export type ArtifactLang = 'json' | 'yaml' | 'markdown' | 'bash' | 'toml' | 'text'

export interface Artifact {
  path: string
  lang: ArtifactLang
  content: string
}

export interface MarketplaceState {
  installed: Set<string>
}

export type PluginComponentKind =
  | 'agents'
  | 'apps'
  | 'assets'
  | 'bin'
  | 'commands'
  | 'files'
  | 'hooks'
  | 'lsp-config'
  | 'lsp_servers'
  | 'mcp-config'
  | 'mcp_servers'
  | 'monitors'
  | 'output-styles'
  | 'settings'
  | 'skills'

export interface PluginComponent {
  kind: PluginComponentKind
  path: string
  name: string
  metadata?: Record<string, unknown>
}

export type MarketplaceRuntime = 'claude' | 'codex' | 'gemini'

export interface PluginInstallState {
  installed: boolean
  enabled?: boolean
  installedAt?: string
  updatedAt?: string
}

export interface PluginManifestSummary {
  description?: string
  version?: string
  interface?: unknown
}
