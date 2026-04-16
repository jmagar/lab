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
  variant?: 'default' | 'success' | 'warning' | 'info'
  loading?: boolean
}) {
  const colorMap = {
    default: 'bg-primary/15 text-primary shadow-md shadow-primary/15',
    success: 'bg-[#00e676]/20 text-[#00e676] shadow-md shadow-[#00e676]/20',
    warning: 'bg-[#ff9100]/20 text-[#ff9100] shadow-md shadow-[#ff9100]/20',
    info: 'bg-[#00b0ff]/20 text-[#00b0ff] shadow-md shadow-[#00b0ff]/20',
  }

  return (
    <div className="flex items-center gap-4 p-4">
      <div className={`flex size-10 items-center justify-center rounded-lg ${colorMap[variant]}`}>
        <Icon className="size-5" />
      </div>
      <div>
        {loading ? (
          <>
            <Skeleton className="h-7 w-12 mb-1" />
            <Skeleton className="h-4 w-32" />
          </>
        ) : (
          <>
            <p className="text-2xl font-semibold tabular-nums">{value}</p>
            <p className="text-sm font-medium text-foreground">{label}</p>
            <p className="text-xs text-muted-foreground">{detail}</p>
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

      <div className="flex-1 p-6 space-y-8">
        <div className="rounded-2xl border bg-card p-6 shadow-sm shadow-black/5">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div className="max-w-2xl space-y-2">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Gateway Fleet</p>
              <h1 className="text-3xl font-semibold tracking-tight">Operational overview</h1>
              <p className="text-sm text-muted-foreground sm:text-base">
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
        </div>

        {/* Stats Grid */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="rounded-lg border bg-card">
            <StatCard
              label="Total Gateways"
              value={stats.totalGateways}
              detail="Managed upstream MCP connections."
              icon={Cable}
              loading={isLoading}
            />
          </div>
          <div className="rounded-lg border bg-card">
            <StatCard
              label="Healthy Connections"
              value={stats.healthyGateways}
              detail="Connected and probing successfully."
              icon={Activity}
              variant="success"
              loading={isLoading}
            />
          </div>
          <div className="rounded-lg border bg-card">
            <StatCard
              label="Discovered Tools"
              value={stats.totalTools}
              detail="Capabilities currently visible upstream."
              icon={Wrench}
              variant="info"
              loading={isLoading}
            />
          </div>
          <div className="rounded-lg border bg-card">
            <StatCard
              label="Exposed Downstream"
              value={stats.exposedTools}
              detail="Tools currently re-published to clients."
              icon={Eye}
              variant="success"
              loading={isLoading}
            />
          </div>
        </div>

        {/* Warnings Banner */}
        {!isLoading && stats.totalWarnings > 0 && (
          <div className="flex items-center gap-3 rounded-lg border-2 border-[#ff9100]/50 bg-gradient-to-r from-[#ff9100]/20 to-[#ffea00]/10 p-4 shadow-lg shadow-[#ff9100]/10">
            <div className="p-2 rounded-full bg-[#ff9100]/20">
              <AlertTriangle className="size-5 text-[#ff9100]" />
            </div>
            <div className="flex-1">
              <p className="font-semibold text-[#ff9100]">
                {stats.totalWarnings} warning{stats.totalWarnings !== 1 ? 's' : ''} across gateways
              </p>
              <p className="text-sm text-muted-foreground">
                Review unhealthy or overexposed gateways before publishing more downstream tools.
              </p>
            </div>
            <Button variant="outline" size="sm" asChild className="border-[#ff9100] text-[#ff9100] hover:bg-[#ff9100]/20 hover:text-[#ff9100]">
              <Link href="/gateways">View gateways</Link>
            </Button>
          </div>
        )}

        {/* Recent Gateways */}
        <div>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold">Recent Gateways</h2>
            <Button variant="ghost" size="sm" asChild>
              <Link href="/gateways">
                View all
                <ArrowRight className="size-4 ml-1" />
              </Link>
            </Button>
          </div>

          {isLoading ? (
            <div className="space-y-2">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-4 rounded-lg border bg-card p-4">
                  <Skeleton className="size-10 rounded-lg" />
                  <div className="flex-1">
                    <Skeleton className="h-5 w-32 mb-1" />
                    <Skeleton className="h-4 w-24" />
                  </div>
                  <Skeleton className="h-5 w-16" />
                </div>
              ))}
            </div>
          ) : recentGateways.length === 0 ? (
            <div className="rounded-lg border-2 border-dashed border-primary/20 bg-gradient-to-br from-primary/5 to-info/5 p-8 text-center">
              <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-full bg-gradient-to-br from-primary to-info shadow-lg shadow-primary/20">
                <Cable className="size-7 text-white" />
              </div>
              <p className="font-semibold text-lg">No gateways configured</p>
              <p className="text-sm text-muted-foreground mt-1">
                Add your first MCP gateway to get started
              </p>
              <Button className="mt-4 bg-gradient-to-r from-primary to-info text-white shadow-lg shadow-primary/20 hover:from-primary/90 hover:to-info/90" asChild>
                <Link href="/gateways">Add Gateway</Link>
              </Button>
            </div>
          ) : (
            <div className="space-y-2">
              {recentGateways.map((gateway) => (
                <Link
                  key={gateway.id}
                  href={gatewayDetailHref(gateway.id)}
                  className="group flex flex-col gap-4 rounded-lg border bg-card p-4 transition-all duration-200 hover:border-primary/30 hover:shadow-lg hover:shadow-primary/10 sm:flex-row sm:items-start"
                >
                  <div className={`flex size-10 items-center justify-center rounded-lg transition-colors ${
                    gateway.status.healthy && gateway.status.connected 
                      ? 'bg-[#00e676]/20 text-[#00e676] shadow-md shadow-[#00e676]/20'
                      : 'bg-[#ff1744]/20 text-[#ff1744] shadow-md shadow-[#ff1744]/20'
                  }`}>
                    <Cable className="size-5" />
                  </div>
                  <div className="flex-1 min-w-0 space-y-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="truncate font-semibold transition-colors group-hover:text-primary">{gateway.name}</p>
                      <StatusBadge healthy={gateway.status.healthy} connected={gateway.status.connected} />
                      <TransportBadge transport={gateway.transport} />
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {gateway.status.discovered_tool_count} discovered tools, {gateway.status.exposed_tool_count} exposed downstream
                    </p>
                    <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                      <span className="inline-flex items-center gap-1">
                        <Clock3 className="size-3.5" />
                        Updated {new Date(gateway.updated_at).toLocaleDateString()}
                      </span>
                      {gateway.warnings.length > 0 && (
                        <span>{gateway.warnings.length} warning{gateway.warnings.length === 1 ? '' : 's'}</span>
                      )}
                    </div>
                  </div>
                  <div className="text-sm font-medium text-muted-foreground sm:text-right">
                    <span className="block text-lg font-semibold tabular-nums text-foreground">
                      {gateway.status.exposed_tool_count}
                    </span>
                    exposed
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  )
}
