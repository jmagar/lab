import { performServiceAction, isAbortError } from './service-action-client'
import { confirmGatewayParams } from './gateway-request'
import { isStandaloneBearerAuthMode } from '../auth/auth-mode.ts'
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
type RestServerListRaw = { servers: ServerJSON[]; next_cursor: string | null }

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
  const qs = new URLSearchParams()
  if (params.search) qs.set('search', params.search)
  if (params.owner) qs.set('owner', params.owner)
  if (params.limit != null) qs.set('limit', String(params.limit))
  if (params.cursor) qs.set('cursor', params.cursor)
  if (params.version) qs.set('version', params.version)
  if (params.updated_since) qs.set('updated_since', params.updated_since)

  const qstr = qs.toString()
  const url = qstr ? `/v0.1/servers?${qstr}` : '/v0.1/servers'
  const token = process.env.NEXT_PUBLIC_API_TOKEN
  const standaloneBearerAuth = isStandaloneBearerAuthMode(token)
  const headers: HeadersInit = {}

  if (standaloneBearerAuth && token) {
    headers.Authorization = `Bearer ${token}`
  }

  let response: Response
  try {
    response = await fetch(url, {
      headers,
      cache: 'no-store',
      credentials: standaloneBearerAuth ? 'omit' : 'include',
      signal,
    })
  } catch (error) {
    if (isAbortError(error)) throw error
    const msg = error instanceof Error ? error.message : 'unknown network error'
    throw createRegistryError(`Registry list failed: ${msg}`, 502, 'backend_unreachable')
  }

  if (!response.ok) {
    const body = await (response.json() as Promise<{ message?: string; kind?: string }>).catch((): { message?: string; kind?: string } => ({}))
    throw createRegistryError(body.message ?? 'Failed to list servers', response.status, body.kind)
  }

  const raw = await (response.json() as Promise<RestServerListRaw>)
  const servers: ServerResponse[] = raw.servers.map((s) => ({ server: normalizeServerJSON(s), _meta: null }))
  return { servers, metadata: { count: servers.length, nextCursor: raw.next_cursor } }
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
