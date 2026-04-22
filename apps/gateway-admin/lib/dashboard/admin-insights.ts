import type { Gateway } from '@/lib/types/gateway'
import type { LogEvent } from '@/lib/types/logs'

export interface ActivityItem {
  id: string
  kind: 'session' | 'tool' | 'resource' | 'prompt' | 'oauth' | 'other'
  tone: 'success' | 'warning' | 'danger' | 'info'
  title: string
  detail: string
  timestamp: string
  event: LogEvent
}

/** Log events surfaced on the activity page, by subsystem. */
export const ACTIVITY_SUBSYSTEMS = [
  'mcp_server',
  'oauth_relay',
  'auth_mcp',
  'auth_upstream',
] as const

interface ActionDescriptor {
  kind: ActivityItem['kind']
  label: string
}

const ACTION_LABELS: Record<string, ActionDescriptor> = {
  // MCP session lifecycle
  'session.initialized': { kind: 'session', label: 'MCP session connected' },
  // MCP tool / resource / prompt dispatch
  list_tools: { kind: 'tool', label: 'Listed tools' },
  call_tool: { kind: 'tool', label: 'Called tool' },
  list_resources: { kind: 'resource', label: 'Listed resources' },
  read_resource: { kind: 'resource', label: 'Read resource' },
  list_prompts: { kind: 'prompt', label: 'Listed prompts' },
  get_prompt: { kind: 'prompt', label: 'Fetched prompt' },
  // OAuth relay / upstream
  'oauth.relay.start': { kind: 'oauth', label: 'OAuth relay started' },
  probe: { kind: 'oauth', label: 'Probed upstream OAuth' },
  start: { kind: 'oauth', label: 'Started OAuth flow' },
  status: { kind: 'oauth', label: 'Checked OAuth status' },
  callback: { kind: 'oauth', label: 'OAuth callback' },
  clear: { kind: 'oauth', label: 'Cleared OAuth session' },
}

function describeAction(event: LogEvent): ActionDescriptor {
  if (event.action && ACTION_LABELS[event.action]) {
    return ACTION_LABELS[event.action]
  }
  if (event.subsystem === 'oauth_relay' || event.subsystem.startsWith('auth_')) {
    return { kind: 'oauth', label: event.action ?? 'OAuth event' }
  }
  if (event.subsystem === 'mcp_server') {
    return { kind: 'session', label: event.action ?? 'MCP event' }
  }
  return { kind: 'other', label: event.action ?? event.subsystem }
}

function toneFor(event: LogEvent): ActivityItem['tone'] {
  if (event.level === 'error') return 'danger'
  if (event.level === 'warn') return 'warning'
  if (event.outcome_kind && event.outcome_kind !== 'ok') return 'warning'
  return 'success'
}

function formatDetail(event: LogEvent, descriptor: ActionDescriptor): string {
  const parts: string[] = []
  const fields = event.fields_json as Record<string, unknown> | undefined

  if (descriptor.kind === 'tool' && event.action === 'call_tool') {
    const name = fields?.tool_name ?? fields?.name ?? fields?.service
    if (typeof name === 'string' && name.length > 0) parts.push(name)
  }
  if (descriptor.kind === 'resource' && event.action === 'read_resource') {
    const uri = fields?.uri ?? fields?.resource_uri
    if (typeof uri === 'string' && uri.length > 0) parts.push(uri)
  }
  if (descriptor.kind === 'prompt' && event.action === 'get_prompt') {
    const name = fields?.prompt_name ?? fields?.name
    if (typeof name === 'string' && name.length > 0) parts.push(name)
  }

  parts.push(event.message)
  if (event.instance) parts.push(`instance=${event.instance}`)
  if (event.source_node_id && event.source_kind !== 'local') {
    parts.push(`node=${event.source_node_id}`)
  }
  return parts.filter(Boolean).join(' · ')
}

export function buildActivityItemsFromLogs(events: LogEvent[]): ActivityItem[] {
  return events.map((event) => {
    const descriptor = describeAction(event)
    return {
      id: event.event_id,
      kind: descriptor.kind,
      tone: toneFor(event),
      title: descriptor.label,
      detail: formatDetail(event, descriptor),
      timestamp: new Date(event.ts).toISOString(),
      event,
    }
  })
}

export interface GatewayActivityItem {
  id: string
  gatewayId: string
  gatewayName: string
  kind: 'status' | 'warning'
  tone: 'success' | 'warning' | 'danger'
  title: string
  detail: string
  timestamp: string
}

export interface GatewaySettingsSnapshot {
  authModeLabel: 'Browser session' | 'API token'
  runtimeLabel: 'Live control plane' | 'Mock preview'
  totalGateways: number
  connectedGateways: number
  disconnectedGateways: number
  warningCount: number
  proxyResourceGateways: number
  bearerTokenGateways: number
}

export interface GatewayDocsSnapshot {
  totalGateways: number
  connectedGateways: number
  warningCount: number
  httpGateways: number
  stdioGateways: number
  supportedServices: number
  exposedTools: number
}

interface SettingsOptions {
  hasStandaloneBearerAuth: boolean
  hasMockData: boolean
}

export function buildGatewayActivityFeed(gateways: Gateway[]): GatewayActivityItem[] {
  const parseTimestamp = (value: string) => {
    const parsed = Date.parse(value)
    return Number.isNaN(parsed) ? 0 : parsed
  }

  return gateways
    .flatMap((gateway) => {
      const statusItem: GatewayActivityItem = gateway.status.connected && gateway.status.healthy
        ? {
            id: `${gateway.id}-status`,
            gatewayId: gateway.id,
            gatewayName: gateway.name,
            kind: 'status',
            tone: 'success',
            title: `${gateway.name} is healthy`,
            detail: `Probe completed successfully with ${gateway.status.discovered_tool_count} discovered tools over ${gateway.transport.toUpperCase()}.`,
            timestamp: gateway.updated_at,
          }
        : {
            id: `${gateway.id}-status`,
            gatewayId: gateway.id,
            gatewayName: gateway.name,
            kind: 'status',
            tone: 'danger',
            title: `${gateway.name} needs attention`,
            detail: gateway.status.last_error || 'Gateway is disconnected or not yet configured.',
            timestamp: gateway.updated_at,
          }

      const warningItems = gateway.warnings.map<GatewayActivityItem>((warning, index) => ({
        id: `${gateway.id}-warning-${index}`,
        gatewayId: gateway.id,
        gatewayName: gateway.name,
        kind: 'warning',
        tone: 'warning',
        title: `${warning.code} on ${gateway.name}`,
        detail: warning.message,
        timestamp: warning.timestamp,
      }))

      return [statusItem, ...warningItems]
    })
    .sort((left, right) => {
      const timestampDelta = parseTimestamp(right.timestamp) - parseTimestamp(left.timestamp)
      if (timestampDelta !== 0) {
        return timestampDelta
      }

      if (left.kind === right.kind) {
        return left.title.localeCompare(right.title)
      }

      return left.kind === 'status' ? -1 : 1
    })
}

export function buildGatewaySettingsSnapshot(
  gateways: Gateway[],
  options: SettingsOptions,
): GatewaySettingsSnapshot {
  return {
    authModeLabel: options.hasStandaloneBearerAuth ? 'API token' : 'Browser session',
    runtimeLabel: options.hasMockData ? 'Mock preview' : 'Live control plane',
    totalGateways: gateways.length,
    connectedGateways: gateways.filter((gateway) => gateway.status.connected).length,
    disconnectedGateways: gateways.filter((gateway) => !gateway.status.connected).length,
    warningCount: gateways.reduce((count, gateway) => count + gateway.warnings.length, 0),
    proxyResourceGateways: gateways.filter((gateway) => gateway.config.proxy_resources !== false).length,
    bearerTokenGateways: gateways.filter((gateway) => Boolean(gateway.config.bearer_token_env)).length,
  }
}

export function buildGatewayDocsSnapshot(
  gateways: Gateway[],
  supportedServices: number,
): GatewayDocsSnapshot {
  return {
    totalGateways: gateways.length,
    connectedGateways: gateways.filter((gateway) => gateway.status.connected).length,
    warningCount: gateways.reduce((count, gateway) => count + gateway.warnings.length, 0),
    httpGateways: gateways.filter((gateway) => gateway.transport === 'http').length,
    stdioGateways: gateways.filter((gateway) => gateway.transport === 'stdio').length,
    supportedServices,
    exposedTools: gateways.reduce((count, gateway) => count + gateway.status.exposed_tool_count, 0),
  }
}
