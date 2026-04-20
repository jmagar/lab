import { performServiceAction } from './service-action-client'
import { RegistryApiError } from '@/lib/types/registry'
import type {
  ListServersParams,
  ServerListResponse,
  ServerResponse,
  ValidationResult,
  ServerJSON,
} from '@/lib/types/registry'

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

export async function listServers(
  params: ListServersParams,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  return registryAction<ServerListResponse>('server.list', params, signal)
}

export async function getServer(
  name: string,
  signal?: AbortSignal,
): Promise<ServerResponse> {
  return registryAction<ServerResponse>('server.get', { name }, signal)
}

export async function listVersions(
  name: string,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  return registryAction<ServerListResponse>('server.versions', { name }, signal)
}

export async function validateServer(
  serverJson: ServerJSON,
  signal?: AbortSignal,
): Promise<ValidationResult> {
  return registryAction<ValidationResult>('server.validate', { server_json: serverJson }, signal)
}
