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
  desc: string
  autoUpdate: boolean
  totalPlugins: number
  lastUpdated: string
}

export interface Plugin {
  id: string
  name: string
  marketplaceId: string
  version: string
  description: string
  mkt: string
  ver: string
  desc: string
  tags: string[]
  installed: boolean
  hasUpdate?: boolean
  installedAt?: string
  updatedAt?: string
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
