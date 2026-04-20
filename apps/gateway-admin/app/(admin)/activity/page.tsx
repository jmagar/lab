'use client'

import Link from 'next/link'
import { AlertTriangle, CheckCircle2, Clock3, XCircle } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { buildGatewayActivityFeed } from '@/lib/dashboard/admin-insights'
import { useGateways } from '@/lib/hooks/use-gateways'
import { cn } from '@/lib/utils'
import {
  AURORA_DISPLAY_1,
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'

const activityTimestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

/** Aurora-token tone classes — replaces emerald/amber/rose palette. */
const toneStyles = {
  success: 'border-aurora-accent-strong/30 bg-aurora-accent-strong/10 text-aurora-accent-strong',
  warning: 'border-aurora-warn/30 bg-aurora-warn/10 text-aurora-warn',
  danger: 'border-aurora-error/30 bg-aurora-error/10 text-aurora-error',
} as const

const toneIcons = {
  success: CheckCircle2,
  warning: AlertTriangle,
  danger: XCircle,
} as const

export default function ActivityPage() {
  const { data: gateways, isLoading, error } = useGateways()
  const items = buildGatewayActivityFeed(gateways ?? []).slice(0, 12)

  const warningCount = gateways?.reduce((count, gateway) => count + gateway.warnings.length, 0) ?? 0
  const disconnectedCount = gateways?.filter((gateway) => !gateway.status.connected).length ?? 0

  return (
    <>
      <AppHeader
        breadcrumbs={[{ label: 'Activity' }]}
        actions={(
          <Button variant="outline" size="sm" asChild>
            <Link href="/logs">Open logs</Link>
          </Button>
        )}
      />
      <div className={cn(AURORA_PAGE_FRAME, AURORA_PAGE_SHELL)}>
        {/* Page header */}
        <div className={cn(AURORA_STRONG_PANEL, 'px-6 py-5')}>
          <p className={AURORA_MUTED_LABEL}>Control Plane</p>
          <h1 className={cn(AURORA_DISPLAY_1, 'mt-2 text-aurora-text-primary')}>Gateway activity</h1>
          <p className="mt-2 text-sm text-aurora-text-muted">
            Health probes and warning events derived from the current control-plane state.
            Need the raw event stream?{' '}
            <Link href="/logs" className="font-medium text-aurora-accent-primary underline-offset-4 hover:underline">
              Open the log console
            </Link>
            .
          </p>
        </div>

        {/* Stat cards */}
        <div className="grid gap-3 sm:grid-cols-3">
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>Events</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-aurora-text-primary')}>{items.length}</p>
            <p className="mt-1 text-sm text-aurora-text-muted">Latest gateway health and warning signals.</p>
          </div>
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>Warnings</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2', warningCount > 0 ? 'text-aurora-warn' : 'text-aurora-text-primary')}>
              {warningCount}
            </p>
            <p className="mt-1 text-sm text-aurora-text-muted">Operator-facing issues that still need cleanup.</p>
          </div>
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>Disconnected</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2', disconnectedCount > 0 ? 'text-aurora-error' : 'text-aurora-text-primary')}>
              {disconnectedCount}
            </p>
            <p className="mt-1 text-sm text-aurora-text-muted">Gateways that are not currently reachable.</p>
          </div>
        </div>

        {/* Activity feed */}
        <div className={cn(AURORA_STRONG_PANEL, 'overflow-hidden')}>
          {isLoading ? (
            <div className="space-y-3 p-6">
              {Array.from({ length: 4 }, (_, index) => (
                <div key={index} className="h-24 animate-pulse rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface" />
              ))}
            </div>
          ) : error ? (
            <div className="p-6 text-sm text-aurora-error">
              Failed to load activity because the gateway list could not be loaded.
            </div>
          ) : items.length === 0 ? (
            <div className="p-6 text-sm text-aurora-text-muted">
              No activity available yet. Add a gateway or run a probe to populate this feed.
            </div>
          ) : (
            <div className="divide-y divide-aurora-border-strong/60">
              {items.map((item) => {
                const Icon = toneIcons[item.tone]

                return (
                  <div
                    key={item.id}
                    className="flex flex-col gap-4 px-6 py-5 lg:flex-row lg:items-start lg:justify-between"
                  >
                    <div className="flex gap-3">
                      <div className={cn('mt-0.5 rounded-full border p-2', toneStyles[item.tone])}>
                        <Icon className="size-4" />
                      </div>
                      <div className="space-y-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="font-medium text-aurora-text-primary">{item.title}</p>
                          <Badge variant="outline">{item.kind}</Badge>
                        </div>
                        <p className="text-sm text-aurora-text-muted">{item.detail}</p>
                        <div className="flex items-center gap-2 text-xs text-aurora-text-muted">
                          <Clock3 className="size-3.5" />
                          <span>{activityTimestampFormatter.format(new Date(item.timestamp))}</span>
                        </div>
                      </div>
                    </div>
                    <Button variant="outline" size="sm" asChild>
                      <Link
                        href={gatewayDetailHref(item.gatewayId)}
                        className="focus-visible:ring-aurora-accent-primary/34 focus-visible:ring-offset-aurora-page-bg"
                      >
                        Open gateway
                      </Link>
                    </Button>
                  </div>
                )
              })}
            </div>
          )}
        </div>
      </div>
    </>
  )
}
