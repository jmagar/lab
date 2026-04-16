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
      <div className="flex-1 p-6">
        <div className="space-y-6">
          <div className="rounded-2xl border bg-card p-8 shadow-sm shadow-black/5">
            <div className="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
              <div className="max-w-3xl space-y-4">
                <div className="flex size-12 items-center justify-center rounded-full bg-primary/10 text-primary">
                  <BookOpenText className="size-6" />
                </div>
                <div className="space-y-2">
                  <h1 className="text-3xl font-semibold tracking-tight">Operator guide</h1>
                  <p className="max-w-2xl text-sm text-muted-foreground sm:text-base">
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

          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Warnings</p>
              <p className="mt-2 text-3xl font-semibold text-warning">{snapshot.warningCount}</p>
              <p className="mt-1 text-sm text-muted-foreground">Current warning backlog across all gateways.</p>
            </div>
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">HTTP Gateways</p>
              <p className="mt-2 text-3xl font-semibold">{snapshot.httpGateways}</p>
              <p className="mt-1 text-sm text-muted-foreground">Network-connected MCP upstreams.</p>
            </div>
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">stdio Gateways</p>
              <p className="mt-2 text-3xl font-semibold">{snapshot.stdioGateways}</p>
              <p className="mt-1 text-sm text-muted-foreground">Local process-based MCP upstreams.</p>
            </div>
            <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
              <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">Lab Services</p>
              <p className="mt-2 text-3xl font-semibold">{snapshot.supportedServices}</p>
              <p className="mt-1 text-sm text-muted-foreground">Built-in services that can become gateways.</p>
            </div>
          </div>

          <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
            <div className="rounded-lg border bg-card p-6">
              <div className="flex items-center gap-3">
                <CircleHelp className="size-5 text-primary" />
                <div>
                  <h2 className="text-lg font-semibold">Quick start</h2>
                  <p className="text-sm text-muted-foreground">
                    The shortest path to a working, constrained gateway setup.
                  </p>
                </div>
              </div>
              <div className="mt-5 space-y-4">
                {quickStartSteps.map((step, index) => (
                  <div key={step.title} className="flex gap-4 rounded-xl border p-4">
                    <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold text-primary">
                      {index + 1}
                    </div>
                    <div className="min-w-0 flex-1 space-y-1">
                      <div className="flex items-center justify-between gap-4">
                        <h3 className="font-medium">{step.title}</h3>
                        <Button variant="ghost" size="sm" asChild>
                          <Link href={step.href}>Open</Link>
                        </Button>
                      </div>
                      <p className="text-sm text-muted-foreground">{step.detail}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="rounded-lg border bg-card p-6">
              <h2 className="text-lg font-semibold">Key concepts</h2>
              <div className="mt-5 space-y-4">
                {conceptCards.map((card) => {
                  const Icon = card.icon
                  return (
                    <div key={card.title} className="rounded-xl border p-4">
                      <div className="flex items-center gap-3">
                        <div className="flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
                          <Icon className="size-4.5" />
                        </div>
                        <h3 className="font-medium">{card.title}</h3>
                      </div>
                      <p className="mt-3 text-sm text-muted-foreground">{card.detail}</p>
                    </div>
                  )
                })}
              </div>
            </div>
          </div>

          <div className="rounded-lg border bg-card p-6">
            <h2 className="text-lg font-semibold">Current environment notes</h2>
            {gatewaysLoading || servicesLoading ? (
              <p className="mt-3 text-sm text-muted-foreground">Loading gateway and service metadata…</p>
            ) : gatewaysError || servicesError ? (
              <p className="mt-3 text-sm text-destructive">
                Unable to build environment-aware guidance because the current gateway metadata could not be loaded.
              </p>
            ) : (
              <div className="mt-4 grid gap-3 md:grid-cols-2">
                <div className="rounded-xl border p-4">
                  <p className="font-medium">Gateway posture</p>
                  <p className="mt-1 text-sm text-muted-foreground">
                    {snapshot.connectedGateways} of {snapshot.totalGateways} gateways are connected right now, with{' '}
                    {snapshot.warningCount} warning{snapshot.warningCount === 1 ? '' : 's'} still visible in the fleet.
                  </p>
                </div>
                <div className="rounded-xl border p-4">
                  <p className="font-medium">Lab-backed onboarding</p>
                  <p className="mt-1 text-sm text-muted-foreground">
                    The add-gateway flow currently exposes {snapshot.supportedServices} supported Lab services for quick
                    setup without hand-writing upstream process details.
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
