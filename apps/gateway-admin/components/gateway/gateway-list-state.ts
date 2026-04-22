import type { Gateway, DiscoveredTool } from '@/lib/types/gateway'

export type GatewayPrimaryLens = 'configured' | 'healthy' | 'disconnected'
export type GatewayStatusFacet = 'configured' | 'healthy' | 'disconnected' | 'enabled' | 'disabled'
export type GatewaySourceFacet = 'lab' | 'custom'
export type GatewayTransportFacet = 'stdio' | 'http'
export type ToolsExposureFilter = 'all' | 'exposed' | 'hidden'

export interface GatewayFilterState {
  primaryLens: GatewayPrimaryLens
  search: string
  status: GatewayStatusFacet[]
  source: GatewaySourceFacet[]
  transport: GatewayTransportFacet[]
}

export interface ToolFilterState {
  search: string
  gatewayIds: string[]
  exposure: ToolsExposureFilter
  source: GatewaySourceFacet[]
  transport: GatewayTransportFacet[]
}

export interface ToolInventoryRow {
  gatewayId: string
  gatewayName: string
  source: string
  sourceFacet: GatewaySourceFacet
  transport: Gateway['transport']
  toolName: string
  description: string
  exposed: boolean
}

export function gatewaySourceFacet(gateway: Pick<Gateway, 'source'>): GatewaySourceFacet {
  return gateway.source === 'lab_service' ? 'lab' : 'custom'
}

export function matchesOrFacet<T extends string>(selected: T[], actual: T): boolean {
  return selected.length === 0 || selected.includes(actual)
}

export function matchesGatewayStatusFacet(
  gateway: Pick<Gateway, 'configured' | 'enabled' | 'status'>,
  selected: GatewayStatusFacet[],
): boolean {
  if (selected.length === 0) return true

  const actual = new Set<GatewayStatusFacet>()
  if (gateway.configured ?? true) actual.add('configured')
  if (gateway.status.healthy && gateway.status.connected) actual.add('healthy')
  if (!gateway.status.connected) actual.add('disconnected')
  if (gateway.enabled ?? true) actual.add('enabled')
  if (!(gateway.enabled ?? true)) actual.add('disabled')

  return selected.some((value) => actual.has(value))
}

function toolMatchesSearch(tool: DiscoveredTool, gateway: Gateway, search: string): boolean {
  if (!search) return true
  const haystack = [tool.name, tool.description ?? '', gateway.name].join(' ').toLowerCase()
  return haystack.includes(search.toLowerCase())
}

export function aggregateToolsFromGateways(gateways: Gateway[]): ToolInventoryRow[] {
  return gateways.flatMap((gateway) =>
    gateway.discovery.tools.map((tool) => ({
      gatewayId: gateway.id,
      gatewayName: gateway.name,
      source: gateway.source ?? 'custom',
      sourceFacet: gatewaySourceFacet(gateway),
      transport: gateway.transport,
      toolName: tool.name,
      description: tool.description ?? '',
      exposed: tool.exposed,
    })),
  )
}

export function filterGateways(gateways: Gateway[], state: GatewayFilterState): Gateway[] {
  return gateways.filter((gateway) => {
    if (state.primaryLens === 'healthy' && !(gateway.status.healthy && gateway.status.connected)) return false
    if (state.primaryLens === 'disconnected' && gateway.status.connected) return false
    if (state.primaryLens === 'configured' && !(gateway.configured ?? true)) return false

    if (state.search) {
      const haystack = [
        gateway.name,
        gateway.config.url ?? '',
        gateway.config.command ?? '',
        gateway.source ?? '',
        gateway.transport,
        ...gateway.discovery.tools.map((tool) => tool.name),
      ]
        .join(' ')
        .toLowerCase()

      if (!haystack.includes(state.search.toLowerCase())) return false
    }

    if (!matchesGatewayStatusFacet(gateway, state.status)) return false
    if (!matchesOrFacet(state.source, gatewaySourceFacet(gateway))) return false
    if (!matchesOrFacet(state.transport, gateway.transport)) return false

    return true
  })
}

export function filterTools(rows: ToolInventoryRow[], state: ToolFilterState): ToolInventoryRow[] {
  return rows.filter((row) => {
    const haystack = [row.toolName, row.description, row.gatewayName].join(' ').toLowerCase()

    if (state.search && !haystack.includes(state.search.toLowerCase())) return false
    if (state.gatewayIds.length > 0 && !state.gatewayIds.includes(row.gatewayId)) return false
    if (state.exposure === 'exposed' && !row.exposed) return false
    if (state.exposure === 'hidden' && row.exposed) return false
    if (!matchesOrFacet(state.source, row.sourceFacet)) return false
    if (!matchesOrFacet(state.transport, row.transport)) return false

    return true
  })
}

export function sortToolRows(rows: ToolInventoryRow[]): ToolInventoryRow[] {
  return [...rows].sort((a, b) => a.toolName.localeCompare(b.toolName, undefined, { sensitivity: 'base' }))
}

export function gatewayMatchesToolSearch(gateway: Gateway, search: string): boolean {
  return gateway.discovery.tools.some((tool) => toolMatchesSearch(tool, gateway, search))
}
