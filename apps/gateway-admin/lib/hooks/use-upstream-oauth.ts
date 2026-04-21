import { useRef } from 'react'
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
  options: { pollWhilePending?: boolean; maxPollDuration?: number } = {},
) {
  const { pollWhilePending = false, maxPollDuration = 300_000 } = options
  const pollStartRef = useRef<number | null>(null)

  return useSWR<UpstreamOauthStatus, Error>(
    name ? `/v1/gateway/oauth/status/${name}` : null,
    () => upstreamOauthApi.status(name!),
    {
      refreshInterval: (data) => {
        if (!pollWhilePending || data?.authenticated) return 0
        if (pollStartRef.current === null) pollStartRef.current = Date.now()
        return Date.now() - pollStartRef.current < maxPollDuration ? 3_000 : 0
      },
      revalidateOnFocus: true,
    },
  )
}
