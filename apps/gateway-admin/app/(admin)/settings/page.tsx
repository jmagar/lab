'use client'

import { KeyRound, LifeBuoy, PlugZap, ShieldCheck } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import {
  AURORA_DISPLAY_TITLE,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
  AURORA_STRONG_PANEL,
} from '@/components/aurora/tokens'
import { hasMockDataAuthMode, isStandaloneBearerAuthMode } from '@/lib/auth/auth-mode'
import { buildGatewaySettingsSnapshot } from '@/lib/dashboard/admin-insights'
import { useGateways } from '@/lib/hooks/use-gateways'
import { cn } from '@/lib/utils'

// DEVIATION (eixf.6): components/ui/card.tsx does not expose a `variant` prop.
// Applying AURORA_STRONG_PANEL / AURORA_MEDIUM_PANEL utility strings via
// className instead of `<Card variant="strong">`. Visual result is identical
// (both are documented Aurora panel recipes) and no primitive is modified.

export default function SettingsPage() {
  const { data: gateways, isLoading, error } = useGateways()
  const snapshot = gateways ? buildGatewaySettingsSnapshot(gateways, {
    hasStandaloneBearerAuth: isStandaloneBearerAuthMode(),
    hasMockData: hasMockDataAuthMode(),
  }) : null

  return (
    <div className={AURORA_PAGE_SHELL}>
      <AppHeader
        breadcrumbs={[
          { label: 'Settings' }
        ]}
      />
      <div className={AURORA_PAGE_FRAME}>
        <header className="flex flex-col gap-2">
          <h1 className={cn(AURORA_DISPLAY_TITLE, 'text-2xl font-semibold text-aurora-text-primary')}>
            Settings
          </h1>
          <p className="text-sm text-aurora-text-muted">
            Control-plane posture and effective defaults for the admin surface.
          </p>
        </header>

        {isLoading ? (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {Array.from({ length: 4 }, (_, index) => (
              <div
                key={index}
                className={cn(AURORA_MEDIUM_PANEL, 'h-28 animate-pulse')}
              />
            ))}
          </div>
        ) : error || !snapshot ? (
          <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-error')}>
            Failed to load settings because the gateway list is unavailable.
          </Card>
        ) : (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <Card className={cn(AURORA_MEDIUM_PANEL, 'gap-2 p-4')}>
              <p className={AURORA_MUTED_LABEL}>Auth Mode</p>
              <p className={cn(AURORA_DISPLAY_TITLE, 'text-xl font-semibold text-aurora-text-primary')}>
                {snapshot.authModeLabel}
              </p>
              <p className="text-sm text-aurora-text-muted">
                How the web UI authenticates control-plane requests.
              </p>
            </Card>
            <Card className={cn(AURORA_MEDIUM_PANEL, 'gap-2 p-4')}>
              <p className={AURORA_MUTED_LABEL}>Runtime</p>
              <p className={cn(AURORA_DISPLAY_TITLE, 'text-xl font-semibold text-aurora-text-primary')}>
                {snapshot.runtimeLabel}
              </p>
              <p className="text-sm text-aurora-text-muted">
                Current environment mode exposed to the admin UI.
              </p>
            </Card>
            <Card className={cn(AURORA_MEDIUM_PANEL, 'gap-2 p-4')}>
              <p className={AURORA_MUTED_LABEL}>Warnings</p>
              <p className={cn(AURORA_DISPLAY_TITLE, 'text-xl font-semibold text-aurora-warn')}>
                {snapshot.warningCount}
              </p>
              <p className="text-sm text-aurora-text-muted">
                Warnings across all configured gateways.
              </p>
            </Card>
            <Card className={cn(AURORA_MEDIUM_PANEL, 'gap-2 p-4')}>
              <p className={AURORA_MUTED_LABEL}>Disconnected</p>
              <p className={cn(AURORA_DISPLAY_TITLE, 'text-xl font-semibold text-aurora-error')}>
                {snapshot.disconnectedGateways}
              </p>
              <p className="text-sm text-aurora-text-muted">
                Gateways that currently need operator attention.
              </p>
            </Card>
          </div>
        )}

        <div className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
          <Card className={cn(AURORA_STRONG_PANEL, 'gap-4 p-6')}>
            <div className="flex flex-col gap-1">
              <h2 className={cn(AURORA_DISPLAY_TITLE, 'text-lg font-semibold text-aurora-text-primary')}>
                Control-plane posture
              </h2>
              <p className="text-sm text-aurora-text-muted">
                A read-only summary of the admin surface and the current gateway fleet.
              </p>
            </div>

            {isLoading ? (
              <div className="space-y-3">
                {Array.from({ length: 4 }, (_, index) => (
                  <div
                    key={index}
                    className={cn(AURORA_MEDIUM_PANEL, 'h-20 animate-pulse')}
                  />
                ))}
              </div>
            ) : error || !snapshot ? (
              <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-error')}>
                Failed to load settings because the gateway list is unavailable.
              </Card>
            ) : (
              <div className="grid gap-4 md:grid-cols-2">
                <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4')}>
                  <div className="flex items-center gap-3">
                    <ShieldCheck className="size-5 text-aurora-accent-primary" />
                    <div className="space-y-1">
                      <p className="font-medium text-aurora-text-primary">Authentication</p>
                      <p className="text-sm text-aurora-text-muted">
                        UI requests are running in{' '}
                        <span className="font-medium text-aurora-text-primary">
                          {snapshot.authModeLabel}
                        </span>.
                      </p>
                    </div>
                  </div>
                </Card>
                <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4')}>
                  <div className="flex items-center gap-3">
                    <LifeBuoy className="size-5 text-aurora-accent-primary" />
                    <div className="space-y-1">
                      <p className="font-medium text-aurora-text-primary">Preview mode</p>
                      <p className="text-sm text-aurora-text-muted">
                        <span className="font-medium text-aurora-text-primary">
                          {snapshot.runtimeLabel}
                        </span>{' '}
                        is active for this build.
                      </p>
                    </div>
                  </div>
                </Card>
                <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4')}>
                  <div className="flex items-center gap-3">
                    <PlugZap className="size-5 text-aurora-accent-primary" />
                    <div className="space-y-1">
                      <p className="font-medium text-aurora-text-primary">Gateway reachability</p>
                      <p className="text-sm text-aurora-text-muted">
                        {snapshot.connectedGateways} of {snapshot.totalGateways} gateways are connected.
                      </p>
                    </div>
                  </div>
                </Card>
                <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4')}>
                  <div className="flex items-center gap-3">
                    <KeyRound className="size-5 text-aurora-accent-primary" />
                    <div className="space-y-1">
                      <p className="font-medium text-aurora-text-primary">Protected upstreams</p>
                      <p className="text-sm text-aurora-text-muted">
                        {snapshot.bearerTokenGateways} gateways require bearer-token env wiring.
                      </p>
                    </div>
                  </div>
                </Card>
              </div>
            )}
          </Card>

          <Card className={cn(AURORA_STRONG_PANEL, 'gap-4 p-6')}>
            <h2 className={cn(AURORA_DISPLAY_TITLE, 'text-lg font-semibold text-aurora-text-primary')}>
              Effective defaults
            </h2>
            {isLoading ? (
              <div className="space-y-3">
                {Array.from({ length: 4 }, (_, index) => (
                  <div
                    key={index}
                    className={cn(AURORA_MEDIUM_PANEL, 'h-14 animate-pulse')}
                  />
                ))}
              </div>
            ) : error || !snapshot ? (
              <Card className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-error')}>
                Effective defaults are unavailable until the gateway list loads successfully.
              </Card>
            ) : (
              <div className="space-y-3 text-sm text-aurora-text-muted">
                <div className={cn(AURORA_MEDIUM_PANEL, 'flex items-center justify-between p-3')}>
                  <span className="text-aurora-text-primary">Proxy resources enabled</span>
                  <Badge variant="secondary">{snapshot.proxyResourceGateways} gateways</Badge>
                </div>
                <div className={cn(AURORA_MEDIUM_PANEL, 'flex items-center justify-between p-3')}>
                  <span className="text-aurora-text-primary">Disconnected gateways</span>
                  <Badge variant={snapshot.disconnectedGateways === 0 ? 'secondary' : 'destructive'}>
                    {snapshot.disconnectedGateways}
                  </Badge>
                </div>
                <div className={cn(AURORA_MEDIUM_PANEL, 'flex items-center justify-between p-3')}>
                  <span className="text-aurora-text-primary">Warning backlog</span>
                  <Badge variant={snapshot.warningCount === 0 ? 'secondary' : 'outline'}>
                    {snapshot.warningCount}
                  </Badge>
                </div>
                <div className={cn(AURORA_MEDIUM_PANEL, 'border-dashed p-4 text-aurora-text-muted')}>
                  This page is intentionally read-only for now. Global configuration is still managed through environment and backend config, but the UI now exposes the active posture instead of a dead placeholder.
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  )
}
