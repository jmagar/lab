import type { Plugin } from '@/lib/types/marketplace'

export type { Plugin }

export type McpServer = {
  name: string
  description?: string
  version?: string
  package?: string
  transport?: string[]
}

export type AcpAgent = {
  id: string
  name: string
  version: string
  description?: string
  distribution: { npx?: object; uvx?: object; binary?: object }
  installed?: boolean
  builtin?: boolean
  installedAt?: string
  command?: string
  repository?: string
  website?: string
  license?: string
  icon?: string
  authors?: string[]
}

export type MarketplaceItem =
  | { kind: 'plugin'; data: Plugin }
  | { kind: 'mcp_server'; data: McpServer }
  | { kind: 'acp_agent'; data: AcpAgent }

export type ItemTypeFilter = 'all' | 'plugin' | 'mcp_server' | 'acp_agent'

/** Map distribution wire keys to user-facing labels */
export const DIST_LABELS: Record<string, string> = {
  npx: 'npm',
  uvx: 'Python',
  binary: 'Binary',
}
