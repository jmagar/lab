'use client'

import { PauseCircle, PlayCircle, RadioTower, RefreshCw } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import type { LogStreamStatus } from '@/lib/types/logs'

interface LogStreamStatusProps {
  status: LogStreamStatus
  onTogglePause: () => void
  onJumpToNewest: () => void
}

export function LogStreamStatusCard({
  status,
  onTogglePause,
  onJumpToNewest,
}: LogStreamStatusProps) {
  const timeFormatter = new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'medium',
  })

  return (
    <Card className="border-slate-200/80 bg-linear-to-br from-slate-950 via-slate-900 to-sky-950 text-slate-50 shadow-lg shadow-sky-950/10">
      <CardHeader className="gap-3">
        <div className="flex items-center justify-between gap-3">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2 text-base text-slate-50">
              <RadioTower className="size-4 text-cyan-300" />
              Live stream
            </CardTitle>
            <CardDescription className="text-slate-300">
              HTTP SSE is the only true live transport in v1. MCP stays bounded by design.
            </CardDescription>
          </div>
          <Badge
            variant="outline"
            className={
              status.connected
                ? 'border-emerald-400/40 bg-emerald-400/10 text-emerald-200'
                : 'border-rose-400/40 bg-rose-400/10 text-rose-200'
            }
          >
            {status.connected ? 'Connected' : 'Disconnected'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-3">
          <div className="rounded-xl border border-white/10 bg-white/5 p-3">
            <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Mode</p>
            <p className="mt-2 text-lg font-semibold">{status.paused ? 'Buffered' : 'Live edge'}</p>
          </div>
          <div className="rounded-xl border border-white/10 bg-white/5 p-3">
            <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Buffered</p>
            <p className="mt-2 text-lg font-semibold tabular-nums">{status.buffered}</p>
          </div>
          <div className="rounded-xl border border-white/10 bg-white/5 p-3">
            <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Last event</p>
            <p className="mt-2 text-sm font-medium">
              {status.lastEventTs ? timeFormatter.format(new Date(status.lastEventTs)) : 'Waiting for traffic'}
            </p>
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button
            variant={status.paused ? 'default' : 'outline'}
            className={status.paused ? 'bg-cyan-500 text-slate-950 hover:bg-cyan-400' : 'border-white/15 bg-white/5 text-slate-50 hover:bg-white/10'}
            onClick={onTogglePause}
          >
            {status.paused ? <PlayCircle className="size-4" /> : <PauseCircle className="size-4" />}
            {status.paused ? 'Resume live edge' : 'Pause stream'}
          </Button>
          <Button
            variant="outline"
            className="border-white/15 bg-white/5 text-slate-50 hover:bg-white/10"
            onClick={onJumpToNewest}
            disabled={status.buffered === 0 && !status.paused}
          >
            <RefreshCw className="size-4" />
            Jump to newest
          </Button>
        </div>

        {status.error ? (
          <p className="rounded-lg border border-rose-400/30 bg-rose-400/10 px-3 py-2 text-sm text-rose-100">
            {status.error}
          </p>
        ) : null}
      </CardContent>
    </Card>
  )
}
