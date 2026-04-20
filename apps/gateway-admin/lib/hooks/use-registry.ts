'use client'

import useSWR from 'swr'
import { listServers, getServer } from '@/lib/api/mcpregistry-client'
import type { ServerListResponse, ServerResponse } from '@/lib/types/registry'

export const REGISTRY_SERVERS_KEY = '/registry/servers'
export const REGISTRY_SERVER_KEY = '/registry/server'

export const registryServersKey = (
  query: string,
  cursor: string | null,
  version?: string,
  updatedSince?: string,
): [string, string, string | null, string | undefined, string | undefined] => [
  REGISTRY_SERVERS_KEY,
  query,
  cursor,
  version || undefined,
  updatedSince || undefined,
]

export const registryServerKey = (name: string): [string, string] => [
  REGISTRY_SERVER_KEY,
  name,
]

type RegistryServersKey = [string, string, string | null, string | undefined, string | undefined]

// Fetcher exported so bead 3 can wrap it with an AbortController ref.
export function fetchRegistryServers(
  [, query, cursor, version, updatedSince]: RegistryServersKey,
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  return listServers(
    {
      search: query || undefined,
      limit: 20,
      cursor: cursor ?? undefined,
      version: version || undefined,
      updated_since: updatedSince || undefined,
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
    (key: readonly [...RegistryServersKey]) =>
      fetchRegistryServers([key[0], key[1], key[2], key[3], key[4]], undefined),
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
