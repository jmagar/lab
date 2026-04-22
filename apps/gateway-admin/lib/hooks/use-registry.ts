'use client'

import useSWR from 'swr'
import { listServers, getServer } from '@/lib/api/mcpregistry-client'
import type { ServerListResponse, ServerResponse, RegistrySortBy, RegistrySortOrder } from '@/lib/types/registry'

export const REGISTRY_SERVERS_KEY = '/registry/servers'
export const REGISTRY_SERVER_KEY = '/registry/server'

export const registryServersKey = (
  query: string,
  cursor: string | null,
  version?: string,
  updatedSince?: string,
  sortBy?: RegistrySortBy,
  order?: RegistrySortOrder,
): RegistryServersKey => [
  REGISTRY_SERVERS_KEY,
  query,
  cursor,
  version || undefined,
  updatedSince || undefined,
  sortBy || undefined,
  order || undefined,
]

export const registryServerKey = (name: string): [string, string] => [
  REGISTRY_SERVER_KEY,
  name,
]

export type RegistryServersKey = [string, string, string | null, string?, string?, string?, string?]

// Fetcher exported so bead 3 can wrap it with an AbortController ref.
export function fetchRegistryServers(
  [, query, cursor, version, updatedSince, sortBy, order]: RegistryServersKey,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  return listServers(
    {
      search: query || undefined,
      limit: 20,
      cursor: cursor ?? undefined,
      version,
      updated_since: updatedSince && /^\d{4}-\d{2}-\d{2}$/.test(updatedSince)
        ? `${updatedSince}T00:00:00Z`
        : updatedSince,
      sort_by: sortBy as RegistrySortBy | undefined,
      order: order as RegistrySortOrder | undefined,
    },
    signal,
  )
}

export function useRegistryServers(
  query: string,
  cursor: string | null = null,
  version?: string,
  updatedSince?: string,
) {
  return useSWR<ServerListResponse>(
    registryServersKey(query, cursor, version, updatedSince),
    (key: RegistryServersKey) =>
      fetchRegistryServers(key, undefined),
    { revalidateOnFocus: false },
  )
}

export function useRegistryServer(name: string | null) {
  return useSWR<ServerResponse>(
    name ? registryServerKey(name) : null,
    name ? ([, n]: [string, string]) => getServer(n) : null,
    { revalidateOnFocus: false },
  )
}
