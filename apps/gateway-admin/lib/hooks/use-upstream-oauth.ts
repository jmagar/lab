import useSWR from 'swr'
import { upstreamOauthApi } from '@/lib/api/upstream-oauth-client'
import type { UpstreamEntry, UpstreamOauthStatus } from '@/lib/types/upstream-oauth'

export function useUpstreamOauthUpstreams() {
  return useSWR<UpstreamEntry[], Error>(
    '/v1/gateway/oauth/upstreams',
    () => upstreamOauthApi.listUpstreams(),
    { revalidateOnFocus: true },
  )
}

export function useUpstreamOauthStatus(
  name: string | null,
  options: { pollWhilePending?: boolean } = {},
) {
  return useSWR<UpstreamOauthStatus, Error>(
    name ? `/v1/gateway/oauth/status/${name}` : null,
    () => upstreamOauthApi.status(name!),
    {
      refreshInterval: options.pollWhilePending ? 3_000 : 0,
      revalidateOnFocus: true,
    },
  )
}
