/**
 * Marketplace API client — thin wrappers around POST /v1/marketplace.
 *
 * All functions call the lab API directly at the same origin. The Rust binary
 * serves both the static frontend and the /v1/* API, so no proxy is needed.
 */

import { performServiceAction } from '@/lib/api/service-action-client'
import { marketplaceActionUrl } from '@/lib/api/gateway-config'
import type { McpServer, AcpAgent } from './types'

class MarketplaceError extends Error {
  status: number
  code?: string
  constructor(message: string, status: number, code?: string) {
    super(message)
    this.name = 'MarketplaceError'
    this.status = status
    this.code = code
  }
}

function marketplaceAction<T>(action: string, params: object, signal?: AbortSignal): Promise<T> {
  return performServiceAction<T, MarketplaceError>({
    action,
    params,
    signal,
    serviceLabel: 'Marketplace',
    url: marketplaceActionUrl(),
    createError: (msg, status, code) => new MarketplaceError(msg, status, code),
  })
}

// ── MCP Servers ──────────────────────────────────────────────────────────────

export interface McpListResult {
  servers: McpServer[]
}

export async function listMcpServers(signal?: AbortSignal): Promise<McpServer[]> {
  const res = await marketplaceAction<McpListResult>('mcp.list', {}, signal)
  return res.servers ?? []
}

export async function getMcpServer(name: string, signal?: AbortSignal): Promise<McpServer> {
  return marketplaceAction<McpServer>('mcp.get', { name }, signal)
}

export interface McpInstallParams {
  /** MCP server name (matches ServerJSON.name) */
  server_name: string
  /** Server version */
  version: string
  /** Gateway IDs/names to install into */
  gateway_ids: string[]
  /** Env var values to pass (keyed by env var name) */
  env_vars?: Record<string, string>
}

export interface McpInstallGatewayResult {
  gateway_id: string
  ok: boolean
  error?: string
}

export interface McpInstallResult {
  results: McpInstallGatewayResult[]
}

/**
 * Install an MCP server into one or more gateways.
 *
 * Dispatches to the backend marketplace action which handles per-gateway
 * installation and collects partial success results.
 */
export async function installMcpServer(
  params: McpInstallParams,
  signal?: AbortSignal,
): Promise<McpInstallResult> {
  return marketplaceAction<McpInstallResult>('mcp.install', { ...params, confirm: true }, signal)
}

// ── ACP Agents ───────────────────────────────────────────────────────────────

export interface AcpListResult {
  agents: AcpAgent[]
}

export async function listAcpAgents(signal?: AbortSignal): Promise<AcpAgent[]> {
  const res = await marketplaceAction<AcpListResult>('agent.list', {}, signal)
  return res.agents ?? []
}

export async function getAcpAgent(id: string, signal?: AbortSignal): Promise<AcpAgent> {
  return marketplaceAction<AcpAgent>('agent.get', { id }, signal)
}

export interface AcpAgentInstallParams {
  agent_id: string
  device_ids: string[]
  scope: 'global' | 'project'
  project_path?: string
  env_vars?: Record<string, string>
}

export interface AcpAgentInstallDeviceResult {
  device_id: string
  ok: boolean
  message?: string
}

export interface AcpAgentInstallResult {
  results: AcpAgentInstallDeviceResult[]
}

export async function installAcpAgent(
  params: AcpAgentInstallParams,
  signal?: AbortSignal,
): Promise<AcpAgentInstallResult> {
  return marketplaceAction<AcpAgentInstallResult>('agent.install', { ...params, confirm: true }, signal)
}

// ── Cherry-pick ───────────────────────────────────────────────────────────────

export interface CherryPickParams {
  plugin_id: string
  components: string[]
  device_ids: string[]
  scope: 'global' | 'project'
  project_path?: string
}

export interface CherryPickResult {
  rpc_id?: string
}

export async function cherryPickPlugin(
  params: CherryPickParams,
  signal?: AbortSignal,
): Promise<CherryPickResult> {
  return marketplaceAction<CherryPickResult>('plugin.cherry_pick', { ...params, confirm: true }, signal)
}
