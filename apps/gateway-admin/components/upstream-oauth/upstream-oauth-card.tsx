'use client'

import { useEffect, useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { upstreamOauthApi } from '@/lib/api/upstream-oauth-client'
import { useUpstreamOauthStatus } from '@/lib/hooks/use-upstream-oauth'

interface UpstreamOauthCardProps {
  name: string
}

export function UpstreamOauthCard({ name }: UpstreamOauthCardProps) {
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const { data: status, mutate } = useUpstreamOauthStatus(name, {
    pollWhilePending: connecting,
  })

  useEffect(() => {
    if (connecting && status?.authenticated) {
      setConnecting(false)
    }
  }, [connecting, status?.authenticated])

  async function handleConnect() {
    setError(null)
    setConnecting(true)
    try {
      const { authorization_url } = await upstreamOauthApi.start(name)
      window.open(authorization_url, '_blank', 'noopener,noreferrer')
    } catch (err: unknown) {
      setConnecting(false)
      setError(err instanceof Error ? err.message : 'Failed to start authorization')
    }
  }

  async function handleDisconnect() {
    setError(null)
    try {
      await upstreamOauthApi.clear(name)
      await mutate()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to disconnect')
    }
  }

  const badge = (() => {
    if (!status) return <Badge variant="outline">Loading…</Badge>
    if (status.authenticated && status.expires_within_5m)
      return <Badge variant="outline" className="border-yellow-500 text-yellow-600">Expiring</Badge>
    if (status.authenticated)
      return <Badge variant="outline" className="border-green-500 text-green-600">Connected</Badge>
    return <Badge variant="outline" className="text-muted-foreground">Disconnected</Badge>
  })()

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between gap-2">
          <CardTitle className="text-sm font-medium">{name}</CardTitle>
          {badge}
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 pt-0">
        {error && <p className="text-xs text-destructive">{error}</p>}
        <div className="flex items-center gap-2">
          {status?.authenticated ? (
            <Button variant="outline" size="sm" onClick={handleDisconnect}>
              Disconnect
            </Button>
          ) : (
            <Button size="sm" onClick={handleConnect} disabled={connecting}>
              {connecting ? 'Waiting…' : 'Connect'}
            </Button>
          )}
          {connecting && (
            <p className="text-xs text-muted-foreground">
              Complete authorization in the new tab
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
