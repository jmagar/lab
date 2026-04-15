import type {
  CreateGatewayInput,
  ExposurePolicy,
  ExposurePolicyPreview,
  Gateway,
  GatewayWarning,
  TransportType,
  UpdateGatewayInput,
} from '../types/gateway'

export interface BackendGatewayConfigView {
  name: string
  url?: string | null
  command?: string | null
  args?: string[]
  bearer_token_env?: string | null
  proxy_resources?: boolean
  expose_tools?: string[] | null
}

export interface BackendGatewayRuntimeView {
  name: string
  tool_count: number
  resource_count: number
  prompt_count: number
  last_error?: string | null
}

export interface BackendGatewayView {
  config: BackendGatewayConfigView
  runtime: BackendGatewayRuntimeView
}

export interface GatewayProbeStatus {
  connected: boolean
  healthy: boolean
  last_error?: string
}

export interface GatewayDiscoverySnapshot {
  tools: string[]
  resources: string[]
  prompts: string[]
}

const NOW = () => new Date().toISOString()

function inferTransport(config: BackendGatewayConfigView): TransportType {
  return config.command ? 'stdio' : 'http'
}

function normalizeArgs(args?: string[]): string[] {
  return Array.isArray(args) ? args : []
}

function matchPattern(toolName: string, pattern: string): boolean {
  if (pattern === '*') {
    return true
  }

  const parts = pattern.split('*')
  if (parts.length === 1) {
    return pattern === toolName
  }

  const anchoredStart = !pattern.startsWith('*')
  const anchoredEnd = !pattern.endsWith('*')
  const nonEmptyParts = parts.filter((part) => part.length > 0)

  if (nonEmptyParts.length === 0) {
    return true
  }

  let cursor = 0
  for (const [index, part] of nonEmptyParts.entries()) {
    if (index === 0 && anchoredStart) {
      if (!toolName.slice(cursor).startsWith(part)) {
        return false
      }
      cursor += part.length
      continue
    }

    const found = toolName.slice(cursor).indexOf(part)
    if (found === -1) {
      return false
    }
    cursor += found + part.length
  }

  if (anchoredEnd) {
    return toolName.endsWith(nonEmptyParts[nonEmptyParts.length - 1]!)
  }

  return true
}

function matchTool(toolName: string, patterns?: string[] | null): string | null {
  if (!patterns || patterns.length === 0) {
    return '*'
  }

  for (const pattern of patterns) {
    if (matchPattern(toolName, pattern)) {
      return pattern
    }
  }

  return null
}

function describeTarget(config: BackendGatewayConfigView): string {
  if (config.url) {
    return config.url
  }

  if (!config.command) {
    return config.name
  }

  if (config.command === 'env') {
    const actualCommand = (config.args ?? []).find((arg) => !arg.includes('='))
    return actualCommand ?? config.command
  }

  return config.command
}

function classifyWarning(lastError?: string) {
  if (!lastError) {
    return { code: 'connection_error', message: undefined }
  }

  if (lastError.includes('Authentication is required')) {
    return { code: 'auth_required', message: lastError }
  }

  if (lastError.includes('does not implement MCP prompts discovery')) {
    return { code: 'partial_capability', message: lastError }
  }

  if (lastError.toLowerCase().includes('timed out')) {
    return { code: 'timeout', message: lastError }
  }

  return { code: 'connection_error', message: lastError }
}

export function humanizeProbeError(lastError: string | undefined, config: BackendGatewayConfigView): string | undefined {
  if (!lastError) {
    return undefined
  }

  const target = describeTarget(config)

  if (lastError.includes('Auth required')) {
    return `Authentication is required by ${target}. Configure \`bearer_token_env\` with a valid upstream token, then reload this gateway.`
  }

  const urlMatch = lastError.match(/url \(([^)]+)\)/)
  if (urlMatch?.[1]) {
    return `Could not connect to ${urlMatch[1]}. The upstream did not complete the MCP initialize request. Verify the server is running, reachable, and speaking MCP.`
  }

  if (lastError.includes('failed to list prompts from upstream:') && lastError.includes('Method not found')) {
    return `Connected to ${target}, but it does not implement MCP prompts discovery. Tools and resources may still work.`
  }

  if (lastError.includes('No such file or directory')) {
    return `The stdio command for ${config.name} could not start because a referenced file or path does not exist.`
  }

  if (lastError.includes('timed out')) {
    return `The gateway timed out while waiting for ${target} to respond during MCP initialization.`
  }

  return lastError
}

function buildWarnings(probe: GatewayProbeStatus): GatewayWarning[] {
  if (!probe.last_error) {
    return []
  }

  const warning = classifyWarning(probe.last_error)

  return [
    {
      code: warning.code,
      message: warning.message ?? probe.last_error,
      timestamp: NOW(),
    },
  ]
}

export function normalizeGateway(
  view: BackendGatewayView,
  probe: GatewayProbeStatus,
  discovery: GatewayDiscoverySnapshot,
): Gateway {
  const config = view.config
  const humanizedError = humanizeProbeError(probe.last_error, config)
  const exposePatterns = config.expose_tools
  const tools = discovery.tools.map((name) => {
    const matchedBy = matchTool(name, exposePatterns)
    return {
      name,
      description: undefined,
      exposed: matchedBy !== null,
      matched_by: matchedBy,
    }
  })

  return {
    id: config.name,
    name: config.name,
    transport: inferTransport(config),
    config: {
      url: config.url ?? undefined,
      command: config.command ?? undefined,
      args: normalizeArgs(config.args),
      bearer_token_env: config.bearer_token_env ?? undefined,
      proxy_resources: config.proxy_resources ?? false,
      expose_tools: exposePatterns ?? undefined,
    },
    status: {
      healthy: probe.healthy,
      connected: probe.connected,
      last_error: humanizedError,
      discovered_tool_count: view.runtime.tool_count,
      exposed_tool_count: tools.filter((tool) => tool.exposed).length,
    },
    discovery: {
      tools,
      resources: discovery.resources.map((uri) => ({
        name: uri,
        uri,
      })),
      prompts: discovery.prompts.map((name) => ({
        name,
      })),
    },
    warnings: buildWarnings({
      ...probe,
      last_error: humanizedError,
    }),
    created_at: NOW(),
    updated_at: NOW(),
  }
}

export function probeStatusFromRuntime(runtime: BackendGatewayRuntimeView): GatewayProbeStatus {
  const capabilityCount = runtime.tool_count + runtime.resource_count + runtime.prompt_count
  const lastError = runtime.last_error?.trim() || undefined

  if (capabilityCount > 0) {
    return {
      connected: true,
      healthy: !lastError,
      ...(lastError ? { last_error: lastError } : {}),
    }
  }

  return {
    connected: false,
    healthy: false,
    last_error: lastError ?? 'No tools, resources, or prompts were discovered from this gateway.',
  }
}

export function gatewayInputToSpec(input: CreateGatewayInput) {
  return {
    name: input.name,
    url: input.transport === 'http' ? input.config.url ?? null : null,
    command: input.transport === 'stdio' ? input.config.command ?? null : null,
    args: input.transport === 'stdio' ? normalizeArgs(input.config.args) : [],
    bearer_token_env: input.config.bearer_token_env ?? null,
    proxy_resources: input.config.proxy_resources ?? false,
    expose_tools: input.config.expose_tools ?? null,
  }
}

export function buildGatewayPatch(input: UpdateGatewayInput & { name?: string; transport?: TransportType }) {
  const config = input.config ?? {}
  const patch: Record<string, unknown> = {}

  if (input.name !== undefined) {
    patch.name = input.name
  }

  if (input.transport === 'http') {
    patch.url = config.url ?? null
    patch.command = null
    patch.args = []
  } else if (input.transport === 'stdio') {
    patch.url = null
    patch.command = config.command ?? null
    patch.args = normalizeArgs(config.args)
  } else {
    if (config.url !== undefined) patch.url = config.url
    if (config.command !== undefined) patch.command = config.command
    if (config.args !== undefined) patch.args = normalizeArgs(config.args)
  }

  if (config.bearer_token_env !== undefined) {
    patch.bearer_token_env = config.bearer_token_env || null
  }

  if (config.proxy_resources !== undefined) {
    patch.proxy_resources = config.proxy_resources
  }

  if (config.expose_tools !== undefined) {
    patch.expose_tools = config.expose_tools
  }

  return patch
}

export function exposurePolicyFromConfig(config: BackendGatewayConfigView): ExposurePolicy {
  const patterns = config.expose_tools ?? []
  if (patterns.length === 0) {
    return { mode: 'expose_all', patterns: [] }
  }

  return { mode: 'allowlist', patterns }
}

export function previewExposurePolicy(
  toolNames: string[],
  patterns: string[],
): ExposurePolicyPreview {
  if (patterns.length === 0) {
    return {
      matched_tools: toolNames.map((name) => ({ name, matched_by: '*' })),
      unmatched_patterns: [],
      filtered_tools: [],
      exposed_count: toolNames.length,
      filtered_count: 0,
    }
  }

  const matched_tools: ExposurePolicyPreview['matched_tools'] = []
  const filtered_tools: string[] = []
  const usedPatterns = new Set<string>()

  for (const toolName of toolNames) {
    let matchedBy: string | null = null
    for (const pattern of patterns) {
      if (matchPattern(toolName, pattern)) {
        matchedBy = pattern
        usedPatterns.add(pattern)
        break
      }
    }

    if (matchedBy) {
      matched_tools.push({ name: toolName, matched_by: matchedBy })
    } else {
      filtered_tools.push(toolName)
    }
  }

  return {
    matched_tools,
    unmatched_patterns: patterns.filter((pattern) => !usedPatterns.has(pattern)),
    filtered_tools,
    exposed_count: matched_tools.length,
    filtered_count: filtered_tools.length,
  }
}
