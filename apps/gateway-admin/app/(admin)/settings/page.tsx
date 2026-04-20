'use client'

import { KeyRound, LifeBuoy, PlugZap, ShieldCheck } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { hasMockDataAuthMode, isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import { buildGatewaySettingsSnapshot } from '@/lib/dashboard/admin-insights'
import { useGateways } from '@/lib/hooks/use-gateways'

export default function SettingsPage() {
  const { data: gateways, isLoading, error } = useGateways()
  const snapshot = gateways ? buildGatewaySettingsSnapshot(gateways, {
    hasStandaloneBearerAuth: isStandaloneBearerAuthMode(),
    hasMockData: hasMockDataAuthMode(),
  }) : null

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Settings' }
        ]}
      />
      <div className="flex-1 p-6">
        <div className="space-y-6">
          {isLoading ? (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              {Array.from({ length: 4 }, (_, index) => (
                <div key={index} className="h-28 animate-pulse rounded-xl border bg-muted/30" />
              ))}
            </div>
          ) : error || !snapshot ? (
            <div className="rounded-lg border border-destructive/20 bg-destructive/5 p-4 text-sm text-destructive">
              Failed to load settings because the gateway list is unavailable.
            </div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
                <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Auth Mode</p>
                <p className="mt-2 text-xl font-semibold">{snapshot.authModeLabel}</p>
                <p className="mt-1 text-sm text-muted-foreground">How the web UI authenticates control-plane requests.</p>
              </div>
              <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
                <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Runtime</p>
                <p className="mt-2 text-xl font-semibold">{snapshot.runtimeLabel}</p>
                <p className="mt-1 text-sm text-muted-foreground">Current environment mode exposed to the admin UI.</p>
              </div>
              <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
                <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Warnings</p>
                <p className="mt-2 text-xl font-semibold text-warning">{snapshot.warningCount}</p>
                <p className="mt-1 text-sm text-muted-foreground">Warnings across all configured gateways.</p>
              </div>
              <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
                <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Disconnected</p>
                <p className="mt-2 text-xl font-semibold text-destructive">{snapshot.disconnectedGateways}</p>
                <p className="mt-1 text-sm text-muted-foreground">Gateways that currently need operator attention.</p>
              </div>
            </div>
          )}

          <div className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
            <div className="rounded-lg border bg-card p-6">
              <h1 className="text-lg font-semibold">Control-plane posture</h1>
              <p className="mt-1 text-sm text-muted-foreground">
                A read-only summary of the admin surface and the current gateway fleet.
              </p>

              {isLoading ? (
                <div className="mt-6 space-y-3">
                  {Array.from({ length: 4 }, (_, index) => (
                    <div key={index} className="h-20 animate-pulse rounded-lg border bg-muted/30" />
                  ))}
                </div>
              ) : error || !snapshot ? (
                <div className="mt-6 rounded-lg border border-destructive/20 bg-destructive/5 p-4 text-sm text-destructive">
                  Failed to load settings because the gateway list is unavailable.
                </div>
              ) : (
                <div className="mt-6 grid gap-4 md:grid-cols-2">
                  <div className="rounded-lg border p-4">
                    <div className="flex items-center gap-3">
                      <ShieldCheck className="size-5 text-primary" />
                      <div>
                        <p className="font-medium">Authentication</p>
                        <p className="text-sm text-muted-foreground">
                          UI requests are running in <span className="font-medium text-foreground">{snapshot.authModeLabel}</span>.
                        </p>
                      </div>
                    </div>
                  </div>
                  <div className="rounded-lg border p-4">
                    <div className="flex items-center gap-3">
                      <LifeBuoy className="size-5 text-primary" />
                      <div>
                        <p className="font-medium">Preview mode</p>
                        <p className="text-sm text-muted-foreground">
                          <span className="font-medium text-foreground">{snapshot.runtimeLabel}</span> is active for this build.
                        </p>
                      </div>
                    </div>
                  </div>
                  <div className="rounded-lg border p-4">
                    <div className="flex items-center gap-3">
                      <PlugZap className="size-5 text-primary" />
                      <div>
                        <p className="font-medium">Gateway reachability</p>
                        <p className="text-sm text-muted-foreground">
                          {snapshot.connectedGateways} of {snapshot.totalGateways} gateways are connected.
                        </p>
                      </div>
                    </div>
                  </div>
                  <div className="rounded-lg border p-4">
                    <div className="flex items-center gap-3">
                      <KeyRound className="size-5 text-primary" />
                      <div>
                        <p className="font-medium">Protected upstreams</p>
                        <p className="text-sm text-muted-foreground">
                          {snapshot.bearerTokenGateways} gateways require bearer-token env wiring.
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold">Effective defaults</h2>
              {isLoading ? (
                <div className="mt-4 space-y-3">
                  {Array.from({ length: 4 }, (_, index) => (
                    <div key={index} className="h-14 animate-pulse rounded-lg border bg-muted/30" />
                  ))}
                </div>
              ) : error || !snapshot ? (
                <div className="mt-4 rounded-lg border border-destructive/20 bg-destructive/5 p-4 text-sm text-destructive">
                  Effective defaults are unavailable until the gateway list loads successfully.
                </div>
              ) : (
                <div className="mt-4 space-y-3 text-sm text-muted-foreground">
                  <div className="flex items-center justify-between rounded-lg border p-3">
                    <span>Proxy resources enabled</span>
                    <Badge variant="secondary">{snapshot.proxyResourceGateways} gateways</Badge>
                  </div>
                  <div className="flex items-center justify-between rounded-lg border p-3">
                    <span>Disconnected gateways</span>
                    <Badge variant="secondary" status={snapshot.disconnectedGateways === 0 ? 'default' : 'error'}>
                      {snapshot.disconnectedGateways}
                    </Badge>
                  </div>
                  <div className="flex items-center justify-between rounded-lg border p-3">
                    <span>Warning backlog</span>
                    <Badge variant={snapshot.warningCount === 0 ? 'secondary' : 'outline'}>
                      {snapshot.warningCount}
                    </Badge>
                  </div>
                  <div className="rounded-lg border border-dashed p-4">
                    This page is intentionally read-only for now. Global configuration is still managed through environment and backend config, but the UI now exposes the active posture instead of a dead placeholder.
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
