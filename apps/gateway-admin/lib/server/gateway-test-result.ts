import type { TestGatewayResult } from '../types/gateway'
import type { BackendGatewayRuntimeView } from './gateway-adapter'

export function testResultFromProbe(
  runtime: BackendGatewayRuntimeView,
  probe: { connected: boolean; healthy: boolean; last_error?: string },
  detail?: string,
): TestGatewayResult {
  if (!probe.connected) {
    return {
      success: false,
      severity: 'failure',
      message: 'Connection test failed',
      discovered_tools: runtime.tool_count,
      discovered_resources: runtime.resource_count,
      discovered_prompts: runtime.prompt_count,
      error: detail ?? 'Gateway probe completed, but no usable MCP capabilities were discovered.',
    }
  }

  if (!probe.healthy) {
    return {
      success: true,
      severity: 'warning',
      message: 'Connection test passed with warnings',
      discovered_tools: runtime.tool_count,
      discovered_resources: runtime.resource_count,
      discovered_prompts: runtime.prompt_count,
      detail:
        detail ??
        'The gateway connected successfully, but one or more optional MCP capabilities could not be discovered.',
    }
  }

  return {
    success: true,
    severity: 'success',
    message: 'Connection test passed',
    discovered_tools: runtime.tool_count,
    discovered_resources: runtime.resource_count,
    discovered_prompts: runtime.prompt_count,
  }
}
