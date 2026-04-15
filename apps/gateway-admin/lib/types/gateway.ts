// Gateway data model types

export type TransportType = 'http' | 'stdio'

export interface GatewayConfig {
  url?: string
  command?: string
  args?: string[]
  bearer_token_env?: string
  proxy_resources?: boolean
  expose_tools?: string[]
}

export interface GatewayStatus {
  healthy: boolean
  connected: boolean
  last_error?: string
  discovered_tool_count: number
  exposed_tool_count: number
}

export interface DiscoveredTool {
  name: string
  description?: string
  exposed: boolean
  matched_by: string | null
}

export interface DiscoveredResource {
  name: string
  uri: string
  description?: string
}

export interface DiscoveredPrompt {
  name: string
  description?: string
  arguments?: Array<{
    name: string
    description?: string
    required?: boolean
  }>
}

export interface GatewayDiscovery {
  tools: DiscoveredTool[]
  resources: DiscoveredResource[]
  prompts: DiscoveredPrompt[]
}

export interface GatewayWarning {
  code: string
  message: string
  timestamp: string
}

export interface Gateway {
  id: string
  name: string
  transport: TransportType
  config: GatewayConfig
  status: GatewayStatus
  discovery: GatewayDiscovery
  warnings: GatewayWarning[]
  created_at: string
  updated_at: string
}

export interface CreateGatewayInput {
  name: string
  transport: TransportType
  config: GatewayConfig
}

export interface UpdateGatewayInput {
  name?: string
  transport?: TransportType
  config?: Partial<GatewayConfig>
}

export interface TestGatewayResult {
  success: boolean
  severity?: 'success' | 'warning' | 'failure'
  message: string
  latency_ms?: number
  discovered_tools?: number
  discovered_resources?: number
  discovered_prompts?: number
  error?: string
  detail?: string
}

export interface ReloadGatewayResult {
  success: boolean
  message: string
  previous_tool_count: number
  new_tool_count: number
}

// Exposure policy types
export interface ExposurePolicy {
  mode: 'expose_all' | 'allowlist'
  patterns: string[]
}

export interface ExposurePolicyPreview {
  matched_tools: Array<{
    name: string
    matched_by: string
  }>
  unmatched_patterns: string[]
  filtered_tools: string[]
  exposed_count: number
  filtered_count: number
}
