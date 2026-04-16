'use client'

import Link from 'next/link'
import { AlertTriangle, CheckCircle2, Clock3, XCircle } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { buildGatewayActivityFeed } from '@/lib/dashboard/admin-insights'
import { useGateways } from '@/lib/hooks/use-gateways'

const activityTimestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

const toneStyles = {
  success: 'border-emerald-500/20 bg-emerald-500/5 text-emerald-700 dark:text-emerald-300',
  warning: 'border-amber-500/20 bg-amber-500/5 text-amber-700 dark:text-amber-300',
  danger: 'border-rose-500/20 bg-rose-500/5 text-rose-700 dark:text-rose-300',
} as const

const toneIcons = {
  success: CheckCircle2,
  warning: AlertTriangle,
  danger: XCircle,
} as const

export default function ActivityPage() {
  const { data: gateways, isLoading, error } = useGateways()
  const items = buildGatewayActivityFeed(gateways ?? []).slice(0, 12)

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Activity' }
        ]}
      />
      <div className="flex-1 p-6">
        <div className="space-y-6">
          <div className="grid gap-3 sm:grid-cols-3">
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Events</p>
              <p className="mt-2 text-3xl font-semibold">{items.length}</p>
              <p className="mt-1 text-sm text-muted-foreground">Latest gateway health and warning signals.</p>
            </div>
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Warnings</p>
              <p className="mt-2 text-3xl font-semibold text-warning">
                {gateways?.reduce((count, gateway) => count + gateway.warnings.length, 0) ?? 0}
              </p>
              <p className="mt-1 text-sm text-muted-foreground">Operator-facing issues that still need cleanup.</p>
            </div>
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Disconnected</p>
              <p className="mt-2 text-3xl font-semibold text-destructive">
                {gateways?.filter((gateway) => !gateway.status.connected).length ?? 0}
              </p>
              <p className="mt-1 text-sm text-muted-foreground">Gateways that are not currently reachable.</p>
            </div>
          </div>

          <div className="rounded-lg border bg-card">
            <div className="border-b px-6 py-4">
              <h1 className="text-lg font-semibold">Gateway activity</h1>
              <p className="mt-1 text-sm text-muted-foreground">
                Health probes and warning events derived from the current control-plane state.
              </p>
            </div>

            {isLoading ? (
              <div className="space-y-3 p-6">
                {Array.from({ length: 4 }, (_, index) => (
                  <div key={index} className="h-24 animate-pulse rounded-lg border bg-muted/30" />
                ))}
              </div>
            ) : error ? (
              <div className="p-6 text-sm text-destructive">
                Failed to load activity because the gateway list could not be loaded.
              </div>
            ) : items.length === 0 ? (
              <div className="p-6 text-sm text-muted-foreground">
                No activity available yet. Add a gateway or run a probe to populate this feed.
              </div>
            ) : (
              <div className="divide-y">
                {items.map((item) => {
                  const Icon = toneIcons[item.tone]

                  return (
                    <div key={item.id} className="flex flex-col gap-4 px-6 py-5 lg:flex-row lg:items-start lg:justify-between">
                      <div className="flex gap-3">
                        <div className={`mt-0.5 rounded-full border p-2 ${toneStyles[item.tone]}`}>
                          <Icon className="size-4" />
                        </div>
                        <div className="space-y-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <p className="font-medium">{item.title}</p>
                            <Badge variant="outline">{item.kind}</Badge>
                          </div>
                          <p className="text-sm text-muted-foreground">{item.detail}</p>
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Clock3 className="size-3.5" />
                            <span>{activityTimestampFormatter.format(new Date(item.timestamp))}</span>
                          </div>
                        </div>
                      </div>
                      <Button variant="outline" size="sm" asChild>
                        <Link href={gatewayDetailHref(item.gatewayId)}>Open gateway</Link>
                      </Button>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
