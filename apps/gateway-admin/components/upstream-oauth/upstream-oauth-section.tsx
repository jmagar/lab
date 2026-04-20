'use client'

import { useState } from 'react'
import { Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useUpstreamOauthUpstreams } from '@/lib/hooks/use-upstream-oauth'
import { ConnectUpstreamDialog } from './connect-upstream-dialog'
import { UpstreamOauthCard } from './upstream-oauth-card'

export function UpstreamOauthSection() {
  const { data: upstreams, error, isLoading, mutate } = useUpstreamOauthUpstreams()
  const [dialogOpen, setDialogOpen] = useState(false)

  const header = (
    <div className="mb-3 flex items-center justify-between">
      <h2 className="text-lg font-semibold">Upstream Connections</h2>
      <Button size="sm" variant="outline" onClick={() => setDialogOpen(true)}>
        <Plus className="mr-1.5 size-3.5" />
        Connect
      </Button>
    </div>
  )

  return (
    <section>
      {header}

      {isLoading && <p className="text-sm text-muted-foreground">Loading…</p>}

      {error && (
        <p className="text-sm text-destructive">
          Failed to load upstream list: {error.message}
        </p>
      )}

      {!isLoading && !error && upstreams && upstreams.length > 0 && (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {upstreams.map((u) => (
            <UpstreamOauthCard key={u.name} name={u.name} />
          ))}
        </div>
      )}

      {!isLoading && !error && (!upstreams || upstreams.length === 0) && (
        <p className="text-sm text-muted-foreground">No upstream connections configured.</p>
      )}

      <ConnectUpstreamDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onConnected={() => void mutate()}
      />
    </section>
  )
}
