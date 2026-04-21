import { useState, useEffect } from 'react'
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
  const [pollingExpired, setPollingExpired] = useState(false)

  useEffect(() => {
    if (!pollWhilePending) return
    setPollingExpired(false)
    const timer = setTimeout(() => setPollingExpired(true), maxPollDuration)
    return () => clearTimeout(timer)
  }, [pollWhilePending, maxPollDuration])

  return useSWR<UpstreamOauthStatus, Error>(
    name ? `/v1/gateway/oauth/status/${name}` : null,
    () => upstreamOauthApi.status(name!),
    {
      refreshInterval: (data) =>
        pollWhilePending && !pollingExpired && !data?.authenticated ? 3_000 : 0,
      revalidateOnFocus: true,
    },
  )
}
