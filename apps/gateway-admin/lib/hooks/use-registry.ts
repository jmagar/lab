'use client'

import useSWR from 'swr'
import { listServers, getServer } from '@/lib/api/mcpregistry-client'
import type { ServerListResponse, ServerResponse } from '@/lib/types/registry'

export const REGISTRY_SERVERS_KEY = '/registry/servers'
export const REGISTRY_SERVER_KEY = '/registry/server'

export const registryServersKey = (
  query: string,
  cursor: string | null,
): [string, string, string | null] => [REGISTRY_SERVERS_KEY, query, cursor]

export const registryServerKey = (name: string): [string, string] => [
  REGISTRY_SERVER_KEY,
  name,
]

// Fetcher exported so bead 3 can wrap it with an AbortController ref.
export function fetchRegistryServers(
  [, query, cursor]: [string, string, string | null],
  signal?: AbortSignal,
): Promise<ServerListResponse> {
  return listServers({ search: query || undefined, limit: 20, cursor: cursor ?? undefined }, signal)
}

export function useRegistryServers(query: string, cursor: string | null = null) {
  return useSWR<ServerListResponse>(
    registryServersKey(query, cursor),
    (key: readonly [string, string, string | null]) =>
      fetchRegistryServers([key[0], key[1], key[2]], undefined),
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
