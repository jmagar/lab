'use client'

import { useUpstreamOauthUpstreams } from '@/lib/hooks/use-upstream-oauth'
import { UpstreamOauthCard } from './upstream-oauth-card'

export function UpstreamOauthSection() {
  const { data: upstreams, error, isLoading } = useUpstreamOauthUpstreams()

  if (isLoading) {
    return (
      <section>
        <h2 className="mb-3 text-lg font-semibold">Upstream Connections</h2>
        <p className="text-sm text-muted-foreground">Loading…</p>
      </section>
    )
  }

  if (error) {
    return (
      <section>
        <h2 className="mb-3 text-lg font-semibold">Upstream Connections</h2>
        <p className="text-sm text-destructive">
          Failed to load upstream list: {error.message}
        </p>
      </section>
    )
  }

  if (!upstreams || upstreams.length === 0) {
    return null
  }

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold">Upstream Connections</h2>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {upstreams.map((u) => (
          <UpstreamOauthCard key={u.name} name={u.name} />
        ))}
      </div>
    </section>
  )
}
