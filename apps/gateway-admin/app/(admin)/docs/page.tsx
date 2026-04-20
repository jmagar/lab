'use client'

import Link from 'next/link'
import {
  ArrowRight,
  BookOpenText,
  CircleHelp,
  ExternalLink,
  Layers3,
  PlugZap,
  Wrench,
} from 'lucide-react'

import { AppHeader } from '@/components/app-header'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { buildGatewayDocsSnapshot } from '@/lib/dashboard/admin-insights'
import { useGateways, useSupportedServices } from '@/lib/hooks/use-gateways'
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

const quickStartSteps = [
  {
    title: 'Add a gateway',
    detail: 'Start in Gateways to connect an HTTP MCP endpoint or promote a Lab service into its own gateway.',
    href: '/gateways',
  },
  {
    title: 'Check health and discovery',
    detail: 'Use Overview and Activity to confirm reachability, discovered tools, and current warnings.',
    href: '/',
  },
  {
    title: 'Constrain exposure',
    detail: 'Open a gateway detail page to narrow downstream tool exposure with explicit allowlist patterns.',
    href: '/gateways',
  },
] as const

const conceptCards = [
  {
    icon: PlugZap,
    title: 'Gateway',
    detail: 'A managed upstream MCP connection. Gateways can be HTTP endpoints, local stdio processes, or Lab-backed services surfaced through the control plane.',
  },
  {
    icon: Layers3,
    title: 'Exposure policy',
    detail: 'Controls which discovered tools are re-published downstream. Exact names and prefix wildcards are both supported.',
  },
  {
    icon: Wrench,
    title: 'Discovery',
    detail: 'Each probe refreshes the visible tool, resource, and prompt catalogs so operators can see what a gateway will expose before clients rely on it.',
  },
] as const

export default function DocsPage() {
  const { data: gateways = [], isLoading: gatewaysLoading, error: gatewaysError } = useGateways()
  const { data: services = [], isLoading: servicesLoading, error: servicesError } = useSupportedServices()
  const snapshot = buildGatewayDocsSnapshot(gateways, services.length)

  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Documentation' },
        ]}
      />
      <div className={cn(AURORA_PAGE_FRAME, AURORA_PAGE_SHELL)}>
        {/* Hero panel */}
        <div className={cn(AURORA_STRONG_PANEL, 'px-6 py-8')}>
          <div className="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
            <div className="max-w-3xl space-y-4">
              <div className="flex size-12 items-center justify-center rounded-full border border-aurora-accent-primary/30 bg-aurora-accent-primary/10 text-aurora-accent-primary">
                <BookOpenText className="size-6" />
              </div>
              <div className="space-y-2">
                <h1 className={cn(AURORA_DISPLAY_1, 'text-aurora-text-primary')}>Operator guide</h1>
                <p className="max-w-2xl text-sm text-aurora-text-muted sm:text-base">
                  The admin console manages MCP gateways, shows what each upstream exposes, and lets
                  you tighten downstream tool publication before client traffic depends on it.
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                <Badge variant="secondary">{snapshot.totalGateways} gateways tracked</Badge>
                <Badge variant="secondary">{snapshot.connectedGateways} connected</Badge>
                <Badge variant="secondary">{snapshot.exposedTools} tools exposed</Badge>
                <Badge variant="secondary">{snapshot.supportedServices} Lab services available</Badge>
              </div>
            </div>
            <div className="flex flex-wrap gap-3">
              <Button asChild>
                <Link href="/gateways">
                  Open gateways
                  <ArrowRight className="ml-2 size-4" />
                </Link>
              </Button>
              <Button variant="outline" asChild>
                <a href="https://modelcontextprotocol.io" target="_blank" rel="noopener noreferrer">
                  MCP specification
                  <ExternalLink className="ml-2 size-4" />
                </a>
              </Button>
            </div>
          </div>
        </div>

        {/* Stat cards */}
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>Warnings</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-[22px]', snapshot.warningCount > 0 ? 'text-aurora-warn' : 'text-aurora-text-primary')}>
              {snapshot.warningCount}
            </p>
            <p className="mt-1 text-sm text-aurora-text-muted">Current warning backlog across all gateways.</p>
          </div>
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>HTTP Gateways</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-[22px] text-aurora-text-primary')}>{snapshot.httpGateways}</p>
            <p className="mt-1 text-sm text-aurora-text-muted">Network-connected MCP upstreams.</p>
          </div>
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>stdio Gateways</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-[22px] text-aurora-text-primary')}>{snapshot.stdioGateways}</p>
            <p className="mt-1 text-sm text-aurora-text-muted">Local process-based MCP upstreams.</p>
          </div>
          <div className={cn(AURORA_MEDIUM_PANEL, 'px-5 py-4')}>
            <p className={AURORA_MUTED_LABEL}>Lab Services</p>
            <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-[22px] text-aurora-text-primary')}>{snapshot.supportedServices}</p>
            <p className="mt-1 text-sm text-aurora-text-muted">Built-in services that can become gateways.</p>
          </div>
        </div>

        {/* Detail grid */}
        <div className="grid gap-5 xl:grid-cols-[1.1fr_0.9fr]">
          {/* Quick start */}
          <div className={cn(AURORA_STRONG_PANEL, 'px-6 py-5')}>
            <div className="flex items-center gap-3">
              <CircleHelp className="size-5 text-aurora-accent-primary" />
              <div>
                <h2 className="text-lg font-semibold text-aurora-text-primary">Quick start</h2>
                <p className="text-sm text-aurora-text-muted">
                  The shortest path to a working, constrained gateway setup.
                </p>
              </div>
            </div>
            <div className="mt-5 space-y-3">
              {quickStartSteps.map((step, index) => (
                <div key={step.title} className="flex gap-4 rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface p-4">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-full border border-aurora-accent-primary/30 bg-aurora-accent-primary/10 text-sm font-semibold text-aurora-accent-primary">
                    {index + 1}
                  </div>
                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex items-center justify-between gap-4">
                      <h3 className="font-medium text-aurora-text-primary">{step.title}</h3>
                      <Button variant="ghost" size="sm" asChild>
                        <Link
                          href={step.href}
                          className="focus-visible:ring-aurora-accent-primary/34 focus-visible:ring-offset-aurora-page-bg"
                        >
                          Open
                        </Link>
                      </Button>
                    </div>
                    <p className="text-sm text-aurora-text-muted">{step.detail}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Key concepts */}
          <div className={cn(AURORA_STRONG_PANEL, 'px-6 py-5')}>
            <h2 className="text-lg font-semibold text-aurora-text-primary">Key concepts</h2>
            <div className="mt-5 space-y-3">
              {conceptCards.map((card) => {
                const Icon = card.icon
                return (
                  <div key={card.title} className="rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface p-4">
                    <div className="flex items-center gap-3">
                      <div className="flex size-9 items-center justify-center rounded-lg border border-aurora-accent-primary/30 bg-aurora-accent-primary/10 text-aurora-accent-primary">
                        <Icon className="size-4.5" />
                      </div>
                      <h3 className="font-medium text-aurora-text-primary">{card.title}</h3>
                    </div>
                    <p className="mt-3 text-sm text-aurora-text-muted">{card.detail}</p>
                  </div>
                )
              })}
            </div>
          </div>
        </div>

        {/* Environment notes */}
        <div className={cn(AURORA_STRONG_PANEL, 'px-6 py-5')}>
          <h2 className="text-base font-semibold text-aurora-text-primary">Current environment notes</h2>
          {gatewaysLoading || servicesLoading ? (
            <p className="mt-3 text-sm text-aurora-text-muted">Loading gateway and service metadata…</p>
          ) : gatewaysError || servicesError ? (
            <p className="mt-3 text-sm text-aurora-error">
              Unable to build environment-aware guidance because the current gateway metadata could not be loaded.
            </p>
          ) : (
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              <div className="rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface p-4">
                <p className="font-medium text-aurora-text-primary">Gateway posture</p>
                <p className="mt-1 text-sm text-aurora-text-muted">
                  {snapshot.connectedGateways} of {snapshot.totalGateways} gateways are connected right now, with{' '}
                  {snapshot.warningCount} warning{snapshot.warningCount === 1 ? '' : 's'} still visible in the fleet.
                </p>
              </div>
              <div className="rounded-[1rem] border border-aurora-border-strong bg-aurora-control-surface p-4">
                <p className="font-medium text-aurora-text-primary">Lab-backed onboarding</p>
                <p className="mt-1 text-sm text-aurora-text-muted">
                  The add-gateway flow currently exposes {snapshot.supportedServices} supported Lab services for quick
                  setup without hand-writing upstream process details.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  )
}
