import { performServiceAction } from './service-action-client'
import { confirmGatewayParams } from './gateway-request'
import { RegistryApiError, normalizeServerJSON } from '@/lib/types/registry'
import type {
  ListServersParams,
  ServerListResponse,
  ServerResponse,
  ValidationResult,
  ServerJSON,
} from '@/lib/types/registry'
import type { Gateway } from '@/lib/types/gateway'

type RawServerResponse = Omit<ServerResponse, 'server'> & { server: ServerJSON }
type RawServerListResponse = Omit<ServerListResponse, 'servers'> & { servers: RawServerResponse[] }

function normalizeResponse(raw: RawServerResponse): ServerResponse {
  return { ...raw, server: normalizeServerJSON(raw.server) }
}

function createRegistryError(message: string, status: number, code?: string): RegistryApiError {
  return new RegistryApiError(message, status, code)
}

async function registryAction<T>(
  action: string,
  params: object,
  signal?: AbortSignal,
): Promise<T> {
  return performServiceAction<T, RegistryApiError>({
    action,
    params,
    signal,
    serviceLabel: 'McpRegistry',
    url: '/v1/mcpregistry',
    createError: createRegistryError,
  })
}

export interface RegistryConfig {
  url: string
}

export async function getRegistryConfig(signal?: AbortSignal): Promise<RegistryConfig> {
  return registryAction<RegistryConfig>('config', {}, signal)
}

export async function listServers(
  params: ListServersParams,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  const raw = await registryAction<RawServerListResponse>('server.list', params, signal)
  return { ...raw, servers: raw.servers.map(normalizeResponse) }
}

export async function getServer(
  name: string,
  signal?: AbortSignal,
): Promise<ServerResponse> {
  const raw = await registryAction<RawServerResponse>('server.get', { name }, signal)
  return normalizeResponse(raw)
}

export async function listVersions(
  name: string,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  const raw = await registryAction<RawServerListResponse>('server.versions', { name }, signal)
  return { ...raw, servers: raw.servers.map(normalizeResponse) }
}

export async function validateServer(
  serverJson: ServerJSON,
  signal?: AbortSignal,
): Promise<ValidationResult> {
  return registryAction<ValidationResult>('server.validate', { server_json: serverJson }, signal)
}

export interface InstallServerParams {
  name: string
  gateway_name: string
  version: string
  bearer_token_env?: string
}

export async function installServer(
  params: InstallServerParams,
  signal?: AbortSignal,
): Promise<Gateway> {
  return registryAction<Gateway>(
    'server.install',
    confirmGatewayParams(params),
    signal,
  )
}
