// Gateway data model types

export type TransportType = 'http' | 'stdio' | 'in_process'

export interface GatewayConfig {
  url?: string
  command?: string
  args?: string[]
  bearer_token_env?: string
  oauth_enabled?: boolean
  proxy_resources?: boolean
  proxy_prompts?: boolean
  expose_tools?: string[]
}

/** Extended config for create/update payloads only. `bearer_token_value` is write-only and never returned by the API. */
export interface GatewayWriteConfig extends GatewayConfig {
  bearer_token_value?: string
  /** OAuth spec — write-only, never returned by the API. Set when auth mode is 'oauth'. */
  oauth?: { registration_strategy: string; scopes?: string[] }
}

export interface GatewayStatus {
  healthy: boolean
  connected: boolean
  last_error?: string
  discovered_tool_count: number
  exposed_tool_count: number
  discovered_resource_count: number
  exposed_resource_count: number
  discovered_prompt_count: number
  exposed_prompt_count: number
}

export interface SurfaceState {
  enabled: boolean
  connected: boolean
}

export interface SurfaceStates {
  cli: SurfaceState
  api: SurfaceState
  mcp: SurfaceState
  webui: SurfaceState
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
  source?: string
  configured?: boolean
  enabled?: boolean
  surfaces?: SurfaceStates
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
  config: GatewayWriteConfig
}

export interface UpdateGatewayInput {
  name?: string
  transport?: TransportType
  config?: Partial<GatewayWriteConfig>
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

export interface SupportedServiceField {
  name: string
  description: string
  example: string
  secret: boolean
}

export interface SupportedService {
  key: string
  display_name: string
  category: string
  description: string
  required_env: SupportedServiceField[]
  optional_env: SupportedServiceField[]
  default_port?: number | null
}

export interface ServiceConfigField {
  name: string
  present: boolean
  secret: boolean
  value_preview?: string | null
}

export interface ServiceConfig {
  service: string
  configured: boolean
  fields: ServiceConfigField[]
}

export interface ServiceAction {
  name: string
  description: string
  destructive: boolean
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
