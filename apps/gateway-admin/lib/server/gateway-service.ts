import 'server-only'

import type {
  CreateGatewayInput,
  ExposurePolicy,
  ExposurePolicyPreview,
  Gateway,
  ReloadGatewayResult,
  TestGatewayResult,
  UpdateGatewayInput,
} from '../types/gateway'
import {
  type BackendGatewayRuntimeView,
  type BackendGatewayView,
  type GatewayDiscoverySnapshot,
  buildGatewayPatch,
  exposurePolicyFromConfig,
  gatewayInputToSpec,
  humanizeProbeError,
  normalizeGateway,
  previewExposurePolicy,
  probeStatusFromRuntime,
} from './gateway-adapter'
import { BackendGatewayError, gatewayAction } from './gateway-backend'
import { testResultFromProbe } from './gateway-test-result'

async function probeGateway(name: string) {
  try {
    const runtime = await gatewayAction<BackendGatewayRuntimeView>('gateway.test', { name })
    return probeStatusFromRuntime(runtime)
  } catch (error) {
    if (error instanceof BackendGatewayError) {
      return {
        connected: false,
        healthy: false,
        last_error: error.message,
      }
    }
    throw error
  }
}

async function fetchDiscovery(name: string): Promise<GatewayDiscoverySnapshot> {
  const [tools, resources, prompts] = await Promise.all([
    gatewayAction<string[]>('gateway.discovered_tools', { name }),
    gatewayAction<string[]>('gateway.discovered_resources', { name }),
    gatewayAction<string[]>('gateway.discovered_prompts', { name }),
  ])

  return {
    tools,
    resources: resources.map((resource) =>
      resource.includes('://') ? resource : `lab://upstream/${name}/${resource}`,
    ),
    prompts,
  }
}

async function normalizeGatewayView(
  view: BackendGatewayView,
  includeDiscovery: boolean,
): Promise<Gateway> {
  const [probe, discovery] = await Promise.all([
    probeGateway(view.config.name),
    includeDiscovery
      ? fetchDiscovery(view.config.name)
      : Promise.resolve({
          tools: [],
          resources: [],
          prompts: [],
        }),
  ])

  return normalizeGateway(view, probe, discovery)
}

export async function listGateways(): Promise<Gateway[]> {
  const views = await gatewayAction<BackendGatewayView[]>('gateway.list', {})
  return Promise.all(views.map((view) => normalizeGatewayView(view, false)))
}

export async function getGateway(name: string): Promise<Gateway> {
  const view = await gatewayAction<BackendGatewayView>('gateway.get', { name })
  return normalizeGatewayView(view, true)
}

export async function createGateway(input: CreateGatewayInput): Promise<Gateway> {
  const view = await gatewayAction<BackendGatewayView>('gateway.add', {
    spec: gatewayInputToSpec(input),
  })
  return normalizeGatewayView(view, true)
}

export async function updateGateway(name: string, input: UpdateGatewayInput): Promise<Gateway> {
  const view = await gatewayAction<BackendGatewayView>('gateway.update', {
    name,
    patch: buildGatewayPatch(input),
  })
  return normalizeGatewayView(view, true)
}

export async function removeGateway(name: string): Promise<Gateway> {
  const view = await gatewayAction<BackendGatewayView>('gateway.remove', { name })
  return normalizeGatewayView(view, false)
}

export async function testGateway(name: string): Promise<TestGatewayResult> {
  try {
    const [runtime, view] = await Promise.all([
      gatewayAction<BackendGatewayRuntimeView>('gateway.test', { name }),
      gatewayAction<BackendGatewayView>('gateway.get', { name }),
    ])
    const probe = probeStatusFromRuntime(runtime)
    const detail = humanizeProbeError(probe.last_error, view.config)
    return testResultFromProbe(runtime, probe, detail)
  } catch (error) {
    if (error instanceof BackendGatewayError) {
      return {
        success: false,
        severity: 'failure',
        message: 'Connection test failed',
        error: error.message,
      }
    }
    throw error
  }
}

export async function reloadGateway(name: string): Promise<ReloadGatewayResult> {
  const before = await gatewayAction<BackendGatewayView>('gateway.get', { name })
  await gatewayAction('gateway.reload', {})
  const after = await gatewayAction<BackendGatewayView>('gateway.get', { name })

  return {
    success: true,
    message: 'Gateway reloaded successfully',
    previous_tool_count: before.runtime.tool_count,
    new_tool_count: after.runtime.tool_count,
  }
}

export async function getExposurePolicy(name: string): Promise<ExposurePolicy> {
  const view = await gatewayAction<BackendGatewayView>('gateway.get', { name })
  return exposurePolicyFromConfig(view.config)
}

export async function setExposurePolicy(name: string, policy: ExposurePolicy): Promise<ExposurePolicy> {
  const exposeTools = policy.mode === 'allowlist' ? policy.patterns : null
  await gatewayAction<BackendGatewayView>('gateway.update', {
    name,
    patch: {
      expose_tools: exposeTools,
    },
  })
  return policy.mode === 'allowlist'
    ? { mode: 'allowlist', patterns: policy.patterns }
    : { mode: 'expose_all', patterns: [] }
}

export async function previewGatewayExposure(
  name: string,
  patterns: string[],
): Promise<ExposurePolicyPreview> {
  const tools = await gatewayAction<string[]>('gateway.discovered_tools', { name })
  return previewExposurePolicy(tools, patterns)
}
