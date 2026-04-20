import type {
  Gateway,
  CreateGatewayInput,
  UpdateGatewayInput,
  TestGatewayResult,
  ReloadGatewayResult,
  ExposurePolicy,
  ExposurePolicyPreview,
  ServiceConfig,
  ServiceAction,
  SupportedService,
} from '../types/gateway.ts'
import {
  type BackendGatewayRuntimeView,
  type BackendGatewayToolRow,
  type BackendServerView,
  type BackendGatewayView,
  type GatewayDiscoverySnapshot,
  buildGatewayCreatePayload,
  buildGatewayUpdatePayload,
  exposurePolicyFromConfig,
  humanizeProbeError,
  normalizeGateway,
  normalizeServerView,
  previewExposurePolicy,
  probeStatusFromRuntime,
} from '../server/gateway-adapter.ts'
import { testResultFromProbe } from '../server/gateway-test-result.ts'
import { gatewayActionUrl } from './gateway-config'
import { confirmGatewayParams } from './gateway-request'
import { EXPOSE_NONE_PATTERN, stripExposeNonePattern } from './tool-exposure-draft'
import { synthesizeLabGateway } from './gateway-list-model'
import { performServiceAction, type ServiceActionError } from './service-action-client'

export class GatewayApiError extends Error implements ServiceActionError {
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

async function gatewayAction<T>(action: string, params: object, signal?: AbortSignal): Promise<T> {
  return performServiceAction<T, GatewayApiError>({
    action,
    params,
    signal,
    serviceLabel: 'Gateway',
    url: gatewayActionUrl(),
    createError: (message, status, code) => new GatewayApiError(message, status, code),
  })
}

async function fetchDiscovery(name: string, signal?: AbortSignal): Promise<GatewayDiscoverySnapshot> {
  const [tools, resources, prompts] = await Promise.all([
    gatewayAction<BackendGatewayToolRow[]>('gateway.discovered_tools', { name }, signal),
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
  return gatewayAction<BackendServerView>('gateway.server.get', { id }, signal)
}

function compiledTools(actions: ServiceAction[]): ServiceAction[] {
  return [...actions].sort((left, right) => left.name.localeCompare(right.name))
}

async function fetchSortedServiceActions(
  service: string,
  signal?: AbortSignal,
): Promise<ServiceAction[]> {
  return compiledTools(
    await gatewayAction<ServiceAction[]>(
      'gateway.service_actions',
      { service },
      signal,
    ),
  )
}

async function normalizeListedServerView(
  view: BackendServerView,
  signal?: AbortSignal,
): Promise<Gateway> {
  if (view.source === 'lab_service') {
    return normalizeServerView(view, {
      tools: await fetchSortedServiceActions(view.name, signal),
    })
  }

  return normalizeServerView(view)
}

async function normalizeLabServiceServer(
  serverView: BackendServerView,
  signal?: AbortSignal,
): Promise<Gateway> {
  const [serviceConfig, actions] = await Promise.all([
    gatewayAction<ServiceConfig>(
      'gateway.service_config.get',
      { service: serverView.name },
      signal,
    ),
    fetchSortedServiceActions(serverView.name, signal),
  ])
  const serviceView = normalizeServerView(serverView, {
    tools: actions,
  })

  return {
    ...serviceView,
    config: {
      ...serviceView.config,
      url: fieldPreview(serviceConfig, '_URL'),
    },
  }
}

async function fallbackSupportedServiceGateway(
  id: string,
  signal?: AbortSignal,
): Promise<Gateway | null> {
  const supportedServices = await gatewayAction<SupportedService[]>('gateway.supported_services', {}, signal)
  const supported = supportedServices.find((service) => service.key === id)

  if (!supported) {
    return null
  }

  const [serviceConfig, actions] = await Promise.all([
    gatewayAction<ServiceConfig>('gateway.service_config.get', { service: supported.key }, signal),
    fetchSortedServiceActions(supported.key, signal),
  ])

  return synthesizeLabGateway(supported, serviceConfig, actions)
}

async function mutateVirtualServer(
  action: 'gateway.virtual_server.enable' | 'gateway.virtual_server.disable',
  id: string,
  signal?: AbortSignal,
): Promise<Gateway> {
  const view = await gatewayAction<BackendServerView>(action, confirmGatewayParams({ id }), signal)
  return normalizeLabServiceServer(view, signal)
}

function fieldPreview(config: ServiceConfig, suffix: string): string | undefined {
  return config.fields.find((field) => field.name.endsWith(suffix))?.value_preview ?? undefined
}

export const gatewayApi = {
  async list(signal?: AbortSignal): Promise<Gateway[]> {
    const views = await gatewayAction<BackendServerView[]>('gateway.list', {}, signal)
    return Promise.all(views.map((view) => normalizeListedServerView(view, signal)))
  },

  async get(id: string, signal?: AbortSignal): Promise<Gateway> {
    let serverView: BackendServerView
    try {
      serverView = await findServerView(id, signal)
    } catch (error) {
      if (error instanceof GatewayApiError) {
        const fallback = await fallbackSupportedServiceGateway(id, signal)
        if (fallback) {
          return fallback
        }
      }
      throw error
    }
    if (serverView.source === 'lab_service') {
      return normalizeLabServiceServer(serverView, signal)
    }

    const view = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)
    return normalizeGatewayView(view, true, signal)
  },

  async create(input: CreateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const view = await gatewayAction<BackendGatewayView>(
      'gateway.add',
      confirmGatewayParams(buildGatewayCreatePayload(input)),
      signal,
    )
    return normalizeGatewayView(view, true, signal)
  },

  async update(id: string, input: UpdateGatewayInput, signal?: AbortSignal): Promise<Gateway> {
    const view = await gatewayAction<BackendGatewayView>(
      'gateway.update',
      confirmGatewayParams(buildGatewayUpdatePayload(id, input)),
      signal,
    )
    return normalizeGatewayView(view, true, signal)
  },

  async remove(id: string, signal?: AbortSignal): Promise<void> {
    await gatewayAction<BackendGatewayView>('gateway.remove', confirmGatewayParams({ name: id }), signal)
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
    await gatewayAction('gateway.reload', confirmGatewayParams({}), signal)
    const after = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)

    return {
      success: true,
      message: 'Gateway reloaded successfully',
      previous_tool_count: before.runtime.tool_count,
      new_tool_count: after.runtime.tool_count,
    }
  },

  async getExposurePolicy(id: string, signal?: AbortSignal): Promise<ExposurePolicy> {
    const serverView = await findServerView(id, signal)
    if (serverView.source === 'lab_service') {
      const policy = await gatewayAction<{ allowed_actions: string[] }>(
        'gateway.virtual_server.get_mcp_policy',
        { id },
        signal,
      )
      const patterns = stripExposeNonePattern(policy.allowed_actions)
      return {
        mode: policy.allowed_actions.length === 0 ? 'expose_all' : 'allowlist',
        patterns,
      }
    }

    const view = await gatewayAction<BackendGatewayView>('gateway.get', { name: id }, signal)
    return exposurePolicyFromConfig(view.config)
  },

  async setExposurePolicy(id: string, policy: ExposurePolicy, signal?: AbortSignal): Promise<ExposurePolicy> {
    const serverView = await findServerView(id, signal)
    if (serverView.source === 'lab_service') {
      const allowedActions = policy.mode === 'allowlist'
        ? policy.patterns.length === 0 ? [EXPOSE_NONE_PATTERN] : policy.patterns
        : []
      await gatewayAction<{ allowed_actions: string[] }>(
        'gateway.virtual_server.set_mcp_policy',
        confirmGatewayParams({
          id,
          allowed_actions: allowedActions,
        }),
        signal,
      )
      return {
        mode: policy.mode,
        patterns: stripExposeNonePattern(allowedActions),
      }
    }

    const exposeTools = policy.mode === 'allowlist'
      ? policy.patterns.length === 0 ? [EXPOSE_NONE_PATTERN] : policy.patterns
      : null
    await gatewayAction<BackendGatewayView>(
      'gateway.update',
      confirmGatewayParams({
        name: id,
        patch: {
          expose_tools: exposeTools,
        },
      }),
      signal,
    )
    return policy.mode === 'allowlist'
      ? { mode: 'allowlist', patterns: stripExposeNonePattern(exposeTools ?? []) }
      : { mode: 'expose_all', patterns: [] }
  },

  async previewExposurePolicy(
    id: string,
    patterns: string[],
    signal?: AbortSignal,
  ): Promise<ExposurePolicyPreview> {
    const serverView = await findServerView(id, signal)
    const tools =
      serverView.source === 'lab_service'
        ? (await gatewayAction<ServiceAction[]>(
            'gateway.service_actions',
            { service: serverView.name },
            signal,
          )).map(
            (action) => action.name,
          )
        : (await gatewayAction<BackendGatewayToolRow[]>('gateway.discovered_tools', { name: id }, signal)).map(
            (tool) => tool.name,
          )
    return previewExposurePolicy(tools, patterns)
  },

  async supportedServices(signal?: AbortSignal): Promise<SupportedService[]> {
    return gatewayAction<SupportedService[]>('gateway.supported_services', {}, signal)
  },

  async getServiceConfig(service: string, signal?: AbortSignal): Promise<ServiceConfig> {
    return gatewayAction<ServiceConfig>('gateway.service_config.get', { service }, signal)
  },

  async serviceActions(service: string, signal?: AbortSignal): Promise<ServiceAction[]> {
    return gatewayAction<ServiceAction[]>('gateway.service_actions', { service }, signal)
  },

  async setServiceConfig(
    service: string,
    values: Record<string, string>,
    signal?: AbortSignal,
  ): Promise<ServiceConfig> {
    return gatewayAction<ServiceConfig>(
      'gateway.service_config.set',
      confirmGatewayParams({ service, values }),
      signal,
    )
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
      confirmGatewayParams({ id, surface, enabled }),
      signal,
    )
    return normalizeLabServiceServer(view, signal)
  },
}
