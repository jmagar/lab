import { installServer } from '@/lib/api/mcpregistry-client'

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
 * Calls `installServer` from mcpregistry-client once per selected gateway.
 * Results are collected per-gateway so the modal can show partial success.
 */
export async function installMcpServer(
  params: McpInstallParams,
  signal?: AbortSignal,
): Promise<McpInstallResult> {
  const results = await Promise.allSettled(
    params.gateway_ids.map((gatewayId) =>
      installServer(
        {
          name: params.server_name,
          gateway_name: gatewayId,
          version: params.version,
        },
        signal,
      ),
    ),
  )

  return {
    results: results.map((result, i) => {
      const gateway_id = params.gateway_ids[i]!
      if (result.status === 'fulfilled') {
        return { gateway_id, ok: true }
      }
      const err = result.reason
      return {
        gateway_id,
        ok: false,
        error: err instanceof Error ? err.message : 'Installation failed',
      }
    }),
  }
}
