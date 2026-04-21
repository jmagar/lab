'use client'

import Link from 'next/link'
import { Cable, Wrench, Eye, AlertTriangle, ArrowRight, Activity, Clock3 } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Button } from '@/components/ui/button'
import { gatewayDetailHref } from '@/lib/api/gateway-config'
import { useGateways } from '@/lib/hooks/use-gateways'
import { Skeleton } from '@/components/ui/skeleton'
import { StatusBadge } from '@/components/gateway/status-badge'
import { TransportBadge } from '@/components/gateway/transport-badge'
import { cn } from '@/lib/utils'
import {
  AURORA_DISPLAY_1,
  AURORA_DISPLAY_2,
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '@/components/logs/log-theme'

const AURORA_FOCUS_RING =
  'focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/34 focus-visible:ring-offset-2 focus-visible:ring-offset-aurora-page-bg'

type StatVariant = 'default' | 'success' | 'warning' | 'info'

const STAT_ICON_TONE: Record<StatVariant, string> = {
  default:
    'bg-aurora-panel-medium text-aurora-accent-primary shadow-aurora-medium',
  success:
    'bg-aurora-panel-medium text-aurora-accent-strong shadow-aurora-medium',
  warning:
    'bg-aurora-panel-medium text-aurora-warn shadow-aurora-medium',
  info:
    'bg-aurora-panel-medium text-aurora-accent-primary shadow-aurora-medium',
}

function StatCard({
  label,
  value,
  detail,
  icon: Icon,
  variant = 'default',
  loading = false,
}: {
  label: string
  value: number | string
  detail: string
  icon: React.ElementType
  variant?: StatVariant
  loading?: boolean
}) {
  return (
    <div className={cn(AURORA_STRONG_PANEL, 'flex items-center gap-4 p-4')}>
      <div
        className={cn(
          'flex size-10 items-center justify-center rounded-lg border border-aurora-border-strong',
          STAT_ICON_TONE[variant],
        )}
      >
        <Icon className="size-5" />
      </div>
      <div className="min-w-0">
        {loading ? (
          <>
            <Skeleton className="mb-1 h-7 w-12" />
            <Skeleton className="h-4 w-32" />
          </>
        ) : (
          <>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'text-2xl text-aurora-text-primary')}>{value}</p>
            <p className="text-sm font-medium text-aurora-text-primary">{label}</p>
            <p className="text-xs text-aurora-text-muted">{detail}</p>
          </>
        )}
      </div>
    </div>
  )
}

export default function OverviewPage() {
  const { data: gateways, isLoading } = useGateways()

  const stats = {
    totalGateways: gateways?.length ?? 0,
    healthyGateways: gateways?.filter(g => g.status.healthy && g.status.connected).length ?? 0,
    totalTools: gateways?.reduce((sum, g) => sum + g.status.discovered_tool_count, 0) ?? 0,
    exposedTools: gateways?.reduce((sum, g) => sum + g.status.exposed_tool_count, 0) ?? 0,
    totalWarnings: gateways?.reduce((sum, g) => sum + g.warnings.length, 0) ?? 0,
  }

  const recentGateways = gateways?.slice(0, 5) ?? []

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Overview' }
        ]}
      />

      <div
        className={cn(
          'relative min-h-[calc(100vh-3.5rem)] w-full overflow-hidden bg-aurora-page-bg text-aurora-text-primary',
          AURORA_PAGE_SHELL,
        )}
      >
        <div className={cn(AURORA_PAGE_FRAME, 'gap-8')}>
          {/* Hero / intro panel */}
          <section className={cn(AURORA_MEDIUM_PANEL, 'p-6')}>
            <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
              <div className="max-w-2xl space-y-2">
                <p className={AURORA_MUTED_LABEL}>Gateway Fleet</p>
                <h1 className={AURORA_DISPLAY_1}>Operational overview</h1>
                <p className="text-sm text-aurora-text-muted sm:text-base">
                  Keep an eye on reachability, exposure, and recent gateway changes before clients start depending on them.
                </p>
              </div>
              <div className="flex flex-wrap gap-3">
                <Button variant="outline" asChild>
                  <Link href="/activity">Review activity</Link>
                </Button>
                <Button asChild>
                  <Link href="/gateways">Manage gateways</Link>
                </Button>
              </div>
            </div>
          </section>

          {/* Stats Grid */}
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <StatCard
              label="Total Gateways"
              value={stats.totalGateways}
              detail="Managed upstream MCP connections."
              icon={Cable}
              loading={isLoading}
            />
            <StatCard
              label="Healthy Connections"
              value={stats.healthyGateways}
              detail="Connected and probing successfully."
              icon={Activity}
              variant="success"
              loading={isLoading}
            />
            <StatCard
              label="Discovered Tools"
              value={stats.totalTools}
              detail="Capabilities currently visible upstream."
              icon={Wrench}
              variant="info"
              loading={isLoading}
            />
            <StatCard
              label="Exposed Downstream"
              value={stats.exposedTools}
              detail="Tools currently re-published to clients."
              icon={Eye}
              variant="success"
              loading={isLoading}
            />
          </div>

          {/* Warnings Banner */}
          {!isLoading && stats.totalWarnings > 0 && (
            <div
              className={cn(
                AURORA_MEDIUM_PANEL,
                'flex items-center gap-3 border-aurora-warn/50 p-4',
              )}
            >
              <div className="rounded-full border border-aurora-border-strong bg-aurora-panel-strong p-2 shadow-aurora-medium">
                <AlertTriangle className="size-5 text-aurora-warn" />
              </div>
              <div className="flex-1">
                <p className="font-semibold text-aurora-warn">
                  {stats.totalWarnings} warning{stats.totalWarnings !== 1 ? 's' : ''} across gateways
                </p>
                <p className="text-sm text-aurora-text-muted">
                  Review unhealthy or overexposed gateways before publishing more downstream tools.
                </p>
              </div>
              <Button variant="outline" size="sm" asChild>
                <Link href="/gateways">View gateways</Link>
              </Button>
            </div>
          )}

          {/* Recent Gateways */}
          <section className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className={cn(AURORA_DISPLAY_2, 'text-lg font-semibold text-aurora-text-primary')}>
                Recent Gateways
              </h2>
              <Button variant="ghost" size="sm" asChild>
                <Link href="/gateways">
                  View all
                  <ArrowRight className="ml-1 size-4" />
                </Link>
              </Button>
            </div>

            {isLoading ? (
              <div className="space-y-2">
                {[1, 2, 3].map((i) => (
                  <div key={i} className={cn(AURORA_STRONG_PANEL, 'flex items-center gap-4 p-4')}>
                    <Skeleton className="size-10 rounded-lg" />
                    <div className="flex-1">
                      <Skeleton className="mb-1 h-5 w-32" />
                      <Skeleton className="h-4 w-24" />
                    </div>
                    <Skeleton className="h-5 w-16" />
                  </div>
                ))}
              </div>
            ) : recentGateways.length === 0 ? (
              <div className={cn(AURORA_STRONG_PANEL, 'p-8 text-center')}>
                <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-full border border-aurora-border-strong bg-aurora-panel-medium shadow-aurora-strong">
                  <Cable className="size-7 text-aurora-accent-primary" />
                </div>
                <p className={cn(AURORA_DISPLAY_2, 'text-lg font-semibold text-aurora-text-primary')}>
                  No gateways configured
                </p>
                <p className="mt-1 text-sm text-aurora-text-muted">
                  Add your first MCP gateway to get started
                </p>
                <Button className="mt-4" asChild>
                  <Link href="/gateways">Add Gateway</Link>
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                {recentGateways.map((gateway) => {
                  const healthy = gateway.status.healthy && gateway.status.connected
                  return (
                    <Link
                      key={gateway.id}
                      href={gatewayDetailHref(gateway.id)}
                      className={cn(
                        AURORA_STRONG_PANEL,
                        AURORA_FOCUS_RING,
                        'group flex flex-col gap-4 p-4 transition-all duration-200 hover:border-aurora-accent-primary/40 hover:shadow-aurora-strong sm:flex-row sm:items-start',
                      )}
                    >
                      <div
                        className={cn(
                          'flex size-10 items-center justify-center rounded-lg border border-aurora-border-strong bg-aurora-panel-medium shadow-aurora-medium transition-colors',
                          healthy ? 'text-aurora-accent-strong' : 'text-aurora-error',
                        )}
                      >
                        <Cable className="size-5" />
                      </div>
                      <div className="min-w-0 flex-1 space-y-2">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="truncate font-semibold text-aurora-text-primary transition-colors group-hover:text-aurora-accent-primary">
                            {gateway.name}
                          </p>
                          <StatusBadge healthy={gateway.status.healthy} connected={gateway.status.connected} />
                          <TransportBadge transport={gateway.transport} />
                        </div>
                        <p className="text-sm text-aurora-text-muted">
                          {gateway.status.discovered_tool_count} discovered tools, {gateway.status.exposed_tool_count} exposed downstream
                        </p>
                        <div className="flex flex-wrap items-center gap-3 text-xs text-aurora-text-muted">
                          <span className="inline-flex items-center gap-1">
                            <Clock3 className="size-3.5" />
                            Updated {new Date(gateway.updated_at).toLocaleDateString()}
                          </span>
                          {gateway.warnings.length > 0 && (
                            <span className="text-aurora-warn">
                              {gateway.warnings.length} warning{gateway.warnings.length === 1 ? '' : 's'}
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="text-sm font-medium text-aurora-text-muted sm:text-right">
                        <span className={cn(AURORA_DISPLAY_NUMBER, 'block text-lg text-aurora-text-primary')}>
                          {gateway.status.exposed_tool_count}
                        </span>
                        exposed
                      </div>
                    </Link>
                  )
                })}
              </div>
            )}
          </section>
        </div>
      </div>
    </>
  )
}
