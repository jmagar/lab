'use client'

import Link from 'next/link'
import { AlertTriangle, CheckCircle2, Clock3, XCircle } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  AURORA_DISPLAY_1,
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { buildGatewayActivityFeed } from '@/lib/dashboard/admin-insights'
import { useGateways } from '@/lib/hooks/use-gateways'

const activityTimestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

type ActivityTone = 'success' | 'warning' | 'danger'

const toneToStatus: Record<ActivityTone, 'success' | 'warn' | 'error'> = {
  success: 'success',
  warning: 'warn',
  danger: 'error',
}

const toneIcons = {
  success: CheckCircle2,
  warning: AlertTriangle,
  danger: XCircle,
} as const

const toneIconClass: Record<ActivityTone, string> = {
  success: 'text-aurora-accent-primary',
  warning: 'text-aurora-warn',
  danger: 'text-aurora-error',
}

const ROW_FOCUS =
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/34 focus-visible:ring-offset-2 focus-visible:ring-offset-aurora-page-bg'

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
      <div className={`${AURORA_PAGE_SHELL} flex-1`}>
        <div className={AURORA_PAGE_FRAME}>
          <div className="flex flex-col gap-1.5">
            <h1 className={AURORA_DISPLAY_1}>Activity</h1>
            <p className="text-[13px] text-aurora-text-muted">
              Health probes and warning events derived from the current control-plane state.
            </p>
          </div>

          <div className="grid gap-3 sm:grid-cols-3">
            <div className={`${AURORA_MEDIUM_PANEL} p-4`}>
              <p className={AURORA_MUTED_LABEL}>Events</p>
              <p className={`${AURORA_DISPLAY_NUMBER} mt-2 text-3xl text-aurora-text-primary`}>{items.length}</p>
              <p className="mt-1 text-[12px] text-aurora-text-muted">Latest gateway health and warning signals.</p>
            </div>
            <div className={`${AURORA_MEDIUM_PANEL} p-4`}>
              <p className={AURORA_MUTED_LABEL}>Warnings</p>
              <p className={`${AURORA_DISPLAY_NUMBER} mt-2 text-3xl text-aurora-warn`}>{warningCount}</p>
              <p className="mt-1 text-[12px] text-aurora-text-muted">Operator-facing issues that still need cleanup.</p>
            </div>
            <div className={`${AURORA_MEDIUM_PANEL} p-4`}>
              <p className={AURORA_MUTED_LABEL}>Disconnected</p>
              <p className={`${AURORA_DISPLAY_NUMBER} mt-2 text-3xl text-aurora-error`}>{disconnectedCount}</p>
              <p className="mt-1 text-[12px] text-aurora-text-muted">Gateways that are not currently reachable.</p>
            </div>
          </div>

          <div className={AURORA_STRONG_PANEL}>
            <div className="border-b border-aurora-border-strong px-5 py-4">
              <h2 className="text-[15px] font-semibold text-aurora-text-primary">Gateway activity</h2>
              <p className="mt-1 text-[12px] text-aurora-text-muted">
                Need the raw event stream or persisted runtime history? Open the dedicated{' '}
                <Link
                  href="/logs"
                  className={`font-medium text-aurora-accent-primary underline-offset-4 hover:underline ${ROW_FOCUS} rounded-sm`}
                >
                  log console
                </Link>
                .
              </p>
            </div>

            {isLoading ? (
              <div className="space-y-2 p-5">
                {Array.from({ length: 4 }, (_, index) => (
                  <div key={index} className="h-14 animate-pulse rounded-lg border border-aurora-border-strong bg-aurora-panel-medium/40" />
                ))}
              </div>
            ) : error ? (
              <div className="p-5 text-[13px] text-aurora-error">
                Failed to load activity because the gateway list could not be loaded.
              </div>
            ) : items.length === 0 ? (
              <div className="p-5 text-[13px] text-aurora-text-muted">
                No activity yet. Add a gateway or run a probe to populate this feed.
              </div>
            ) : (
              <ul className="divide-y divide-aurora-border-strong">
                {items.map((item) => {
                  const Icon = toneIcons[item.tone as ActivityTone]
                  const status = toneToStatus[item.tone as ActivityTone]
                  return (
                    <li key={item.id}>
                      <div
                        className={`flex flex-col gap-3 px-5 py-3 lg:flex-row lg:items-center lg:justify-between ${ROW_FOCUS} rounded-none hover:bg-aurora-panel-medium/40`}
                      >
                        <div className="flex items-start gap-3">
                          <Icon className={`mt-0.5 size-4 shrink-0 ${toneIconClass[item.tone as ActivityTone]}`} aria-hidden />
                          <div className="min-w-0 space-y-1">
                            <div className="flex flex-wrap items-center gap-2">
                              <p className="text-[13px] font-medium text-aurora-text-primary">{item.title}</p>
                              <Badge status={status}>{item.kind}</Badge>
                            </div>
                            <p className="text-[12px] text-aurora-text-muted">{item.detail}</p>
                            <div className="flex items-center gap-1.5 text-[11px] text-aurora-text-muted">
                              <Clock3 className="size-3" aria-hidden />
                              <span>{activityTimestampFormatter.format(new Date(item.timestamp))}</span>
                            </div>
                          </div>
                        </div>
                        <Button variant="outline" size="sm" asChild className={ROW_FOCUS}>
                          <Link href={gatewayDetailHref(item.gatewayId)}>Open gateway</Link>
                        </Button>
                      </div>
                    </li>
                  )
                })}
              </ul>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
