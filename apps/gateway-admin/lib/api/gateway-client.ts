import type {
  Gateway,
  CreateGatewayInput,
  UpdateGatewayInput,
  TestGatewayResult,
  ReloadGatewayResult,
  ExposurePolicy,
  ExposurePolicyPreview,
  ServiceConfig,
  SupportedService,
} from '@/lib/types/gateway'
import {
  type BackendGatewayRuntimeView,
  type BackendServerView,
  type BackendGatewayView,
  type GatewayDiscoverySnapshot,
  buildGatewayPatch,
  exposurePolicyFromConfig,
  gatewayInputToSpec,
  humanizeProbeError,
  normalizeGateway,
  normalizeServerView,
  previewExposurePolicy,
  probeStatusFromRuntime,
} from '@/lib/server/gateway-adapter'
import { testResultFromProbe } from '@/lib/server/gateway-test-result'
import { gatewayActionUrl } from './gateway-config'
import { gatewayRequestInit } from './gateway-request'

class GatewayApiError extends Error {
  status: number
  code?: string

  constructor(
    message: string,
    status: number,
    code?: string
  ) {
    super(message)
    this.name = 'GatewayApiError'
    this.status = status
    this.code = code
  }
}

async function parseActionResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'An error occurred' }))
    throw new GatewayApiError(
      error.message || 'An error occurred',
      response.status,
      error.kind || error.code
    )
  }
  return response.json()
}

async function gatewayAction<T>(action: string, params: object, signal?: AbortSignal): Promise<T> {
  let response: Response
  try {
    response = await fetch(gatewayActionUrl(), gatewayRequestInit(action, params, undefined, signal))
  } catch (error) {
    const message = error instanceof Error ? error.message : 'unknown network error'
    throw new GatewayApiError(
      `Gateway backend action \`${action}\` failed before a response was received: ${message}`,
      502,
      'backend_unreachable'
    )
  }

  return parseActionResponse<T>(response)
}

async function fetchDiscovery(name: string, signal?: AbortSignal): Promise<GatewayDiscoverySnapshot> {
  const [tools, resources, prompts] = await Promise.all([
    gatewayAction<string[]>('gateway.discovered_tools', { name }, signal),
    gatewayAction<string[]>('gateway.discovered_resources', { name }, signal),
    gatewayAction<string[]>('gateway.discovered_prompts', { name }, signal),
  ])

  return {
    tools,
    resources: resources.map((resource) =>
      resource.includes('://') ? resource : `lab://upstream/${name}/${resource}`,
    ),
    prompts,
  }
}

async function probeGateway(name: string, signal?: AbortSignal) {
  try {
    const runtime = await gatewayAction<BackendGatewayRuntimeView>('gateway.test', { name }, signal)
    return probeStatusFromRuntime(runtime)
  } catch (error) {
    if (error instanceof GatewayApiError) {
      return {
        connected: false,
        healthy: false,
        last_error: error.message,
      }
    }
    throw error
  }
}

async function normalizeGatewayView(
  view: BackendGatewayView,
  includeDiscovery: boolean,
  signal?: AbortSignal,
): Promise<Gateway> {
  const [probe, discovery] = await Promise.all([
    probeGateway(view.config.name, signal),
    includeDiscovery
      ? fetchDiscovery(view.config.name, signal)
      : Promise.resolve({
          tools: [],
          resources: [],
          prompts: [],
        }),
  ])

  return normalizeGateway(view, probe, discovery)
}

async function findServerView(id: string, signal?: AbortSignal): Promise<BackendServerView> {
  const views = await gatewayAction<BackendServerView[]>('gateway.list', {}, signal)
  const view = views.find((candidate) => candidate.id === id)
  if (!view) {
    throw new GatewayApiError(`gateway \`${id}\` not found`, 404, 'not_found')
  }
  return view
}

async function mutateVirtualServer(
  action: 'gateway.virtual_server.enable' | 'gateway.virtual_server.disable',
  id: string,
  signal?: AbortSignal,
): Promise<Gateway> {
  const view = await gatewayAction<BackendServerView>(action, { id }, signal)
  return normalizeServerView(view)
}

function fieldPreview(config: ServiceConfig, suffix: string): string | undefined {
  return config.fields.find((field) => field.name.endsWith(suffix))?.value_preview ?? undefined
}

export const gatewayApi = {
  async list(signal?: AbortSignal): Promise<Gateway[]> {
    const views = await gatewayAction<BackendServerView[]>('gateway.list', {}, signal)
    return views.map((view) => normalizeServerView(view))
  },

  async get(id: string, signal?: AbortSignal): Promise<Gateway> {
    const serverView = await findServerView(id, signal)
    if (serverView.source === 'lab_service') {
      const [serviceConfig, serviceView] = await Promise.all([
        gatewayAction<ServiceConfig>('gateway.service_config.get', { service: serverView.name }, signal),
        Promise.resolve(normalizeServerView(serverView)),
      ])

      return {
        ...serviceView,
        config: {
          ...serviceView.config,
          url: fieldPreview(serviceConfig, '_URL'),
        },
      }
    }

    const view = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)
    return normalizeGatewayView(view, true, signal)
  },

  async create(input: CreateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const view = await gatewayAction<BackendGatewayView>(
      'gateway.add',
      { spec: gatewayInputToSpec(input) },
      signal,
    )
    return normalizeGatewayView(view, true, signal)
  },

  async update(id: string, input: UpdateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const view = await gatewayAction<BackendGatewayView>(
      'gateway.update',
      {
        name: id,
        patch: buildGatewayPatch(input),
      },
      signal,
    )
    return normalizeGatewayView(view, true, signal)
  },

  async remove(id: string, signal?: AbortSignal): Promise<void> {
    await gatewayAction<BackendGatewayView>('gateway.remove', { name: id }, signal)
  },

  async test(id: string, signal?: AbortSignal): Promise<TestGatewayResult> {
    const [runtime, view] = await Promise.all([
      gatewayAction<BackendGatewayRuntimeView>('gateway.test', { name: id }, signal),
      gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal),
    ])
    const probe = probeStatusFromRuntime(runtime)
    const detail = humanizeProbeError(probe.last_error, view.config)
    return testResultFromProbe(runtime, probe, detail)
  },

  async reload(id: string, signal?: AbortSignal): Promise<ReloadGatewayResult> {
    const before = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)
    await gatewayAction('gateway.reload', {}, signal)
    const after = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)

    return {
      success: true,
      message: 'Gateway reloaded successfully',
      previous_tool_count: before.runtime.tool_count,
      new_tool_count: after.runtime.tool_count,
    }
  },

  async getExposurePolicy(id: string, signal?: AbortSignal): Promise<ExposurePolicy> {
    const view = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)
    return exposurePolicyFromConfig(view.config)
  },

  async setExposurePolicy(id: string, policy: ExposurePolicy, signal?: AbortSignal): Promise<ExposurePolicy> {
    const exposeTools = policy.mode === 'allowlist' ? policy.patterns : null
    await gatewayAction<BackendGatewayView>(
      'gateway.update',
      {
        name: id,
        patch: {
          expose_tools: exposeTools,
        },
      },
      signal,
    )
    return policy.mode === 'allowlist'
      ? { mode: 'allowlist', patterns: policy.patterns }
      : { mode: 'expose_all', patterns: [] }
  },

  async previewExposurePolicy(
    id: string,
    patterns: string[],
    signal?: AbortSignal,
  ): Promise<ExposurePolicyPreview> {
    const tools = await gatewayAction<string[]>('gateway.discovered_tools', { name: id }, signal)
    return previewExposurePolicy(tools, patterns)
  },

  async supportedServices(signal?: AbortSignal): Promise<SupportedService[]> {
    return gatewayAction<SupportedService[]>('gateway.supported_services', {}, signal)
  },

  async getServiceConfig(service: string, signal?: AbortSignal): Promise<ServiceConfig> {
    return gatewayAction<ServiceConfig>('gateway.service_config.get', { service }, signal)
  },

  async setServiceConfig(
    service: string,
    values: Record<string, string>,
    signal?: AbortSignal,
  ): Promise<ServiceConfig> {
    return gatewayAction<ServiceConfig>('gateway.service_config.set', { service, values }, signal)
  },

  async enableVirtualServer(id: string, signal?: AbortSignal): Promise<Gateway> {
    return mutateVirtualServer('gateway.virtual_server.enable', id, signal)
  },

  async disableVirtualServer(id: string, signal?: AbortSignal): Promise<Gateway> {
    return mutateVirtualServer('gateway.virtual_server.disable', id, signal)
  },

  async setVirtualServerSurface(
    id: string,
    surface: 'cli' | 'api' | 'mcp' | 'webui',
    enabled: boolean,
    signal?: AbortSignal,
  ): Promise<Gateway> {
    const view = await gatewayAction<BackendServerView>(
      'gateway.virtual_server.set_surface',
      { id, surface, enabled },
      signal,
    )
    return normalizeServerView(view)
  },
}

export { GatewayApiError }
