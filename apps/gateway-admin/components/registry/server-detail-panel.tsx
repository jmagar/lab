'use client'

import { useState } from 'react'
import { formatDistanceToNow } from 'date-fns'
import { CheckCircle2, ExternalLink, Globe, Package } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { safeHref } from '@/lib/utils/safe-href'
import { cn } from '@/lib/utils'
import { AURORA_MEDIUM_PANEL, AURORA_MUTED_LABEL } from '@/components/gateway/gateway-theme'
import { InstallDialog } from './install-dialog'
import { RegistryStatusBadge } from './registry-status-badge'
import type {
  EnvironmentVariable,
  NormalizedServerJSON,
  Package as RegistryPackage,
  PackageArgument,
  RegistryExtensions,
  RuntimeArgument,
  Transport,
} from '@/lib/types/registry'

interface ServerDetailPanelProps {
  server: NormalizedServerJSON | null
  extensions?: RegistryExtensions | null
  onClose: () => void
}

export function ServerDetailPanel({ server, extensions, onClose }: ServerDetailPanelProps) {
  const open = server !== null

  return (
    <Dialog open={open} onOpenChange={(v) => { if (!v) onClose() }}>
      <DialogContent className="flex max-h-[90vh] w-full flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl">
        {server && <PanelBody server={server} extensions={extensions ?? null} />}
      </DialogContent>
    </Dialog>
  )
}

function PanelBody({ server, extensions }: { server: NormalizedServerJSON; extensions: RegistryExtensions | null }) {
  const displayName = server.title ?? server.name
  const { remotes, icons, packages } = server
  const isHTTP = remotes.some((r) => r.type === 'streamable-http' || r.type === 'sse')
  const [installOpen, setInstallOpen] = useState(false)
  const icon = icons[0] ?? null
  const repoHref = safeHref(server.repository?.url)
  const websiteHref = safeHref(server.websiteUrl)
  const status = extensions?.status ?? null
  const statusMessage = extensions?.statusMessage ?? null

  return (
    <>
      <DialogHeader className="shrink-0 space-y-0 border-b border-aurora-border-strong/60 px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-12 shrink-0 items-center justify-center rounded-xl border border-aurora-border-strong/60 bg-[rgba(14,31,44,0.8)]">
            {icon ? (
              <img
                src={safeHref(icon.src) ?? undefined}
                alt=""
                className="size-8 rounded object-contain"
                referrerPolicy="no-referrer"
                loading="lazy"
                onError={(e) => { e.currentTarget.style.display = 'none' }}
              />
            ) : (
              <Package className="size-6 text-aurora-text-muted" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <DialogTitle className="text-base font-semibold leading-tight text-aurora-text-primary">
              {displayName}
            </DialogTitle>
            <p className="mt-0.5 break-all font-mono text-xs text-aurora-text-muted">{server.name}</p>
            <div className="mt-2 flex flex-wrap gap-2">
              <span className="rounded-full border border-aurora-border-strong/60 px-2 py-0.5 text-xs text-aurora-text-muted">
                v{server.version}
              </span>
              <span
                className={cn(
                  'rounded-full px-2 py-0.5 text-xs font-medium',
                  isHTTP
                    ? 'bg-aurora-accent-strong/15 text-aurora-accent-strong'
                    : 'bg-aurora-border-strong/40 text-aurora-text-muted',
                )}
              >
                {isHTTP ? 'HTTP' : 'stdio only'}
              </span>
              {extensions?.isLatest && (
                <span className="inline-flex items-center gap-1 rounded-full border border-aurora-accent-primary/30 bg-aurora-accent-primary/10 px-2 py-0.5 text-xs font-medium text-aurora-accent-primary">
                  <CheckCircle2 className="size-3" /> Latest
                </span>
              )}
              {extensions?.updatedAt && (
                <span
                  title={extensions.updatedAt}
                  className="cursor-default rounded-full border border-aurora-border-strong/40 px-2 py-0.5 text-xs text-aurora-text-muted"
                >
                  {formatDistanceToNow(new Date(extensions.updatedAt), { addSuffix: true })}
                </span>
              )}
              <RegistryStatusBadge status={status} />
            </div>
            {statusMessage && (status === 'deprecated' || status === 'deleted') && (
              <p className="mt-2 text-xs leading-relaxed text-aurora-text-secondary">
                {statusMessage}
              </p>
            )}
          </div>
        </div>
      </DialogHeader>

      <div className="flex-1 space-y-6 overflow-y-auto px-6 py-5">
        {/* Description — untrusted registry data, do not use dangerouslySetInnerHTML */}
        <Section label="Description">
          <p className="whitespace-pre-wrap text-sm leading-relaxed text-aurora-text-secondary">
            {server.description}
          </p>
        </Section>

        {(repoHref || websiteHref || server.$schema) && (
          <Section label="Links">
            <div className="flex flex-wrap gap-3">
              {repoHref && (
                <a
                  href={repoHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                >
                  <ExternalLink className="size-3.5" />
                  Repository
                  {server.repository?.source && (
                    <span className="text-xs text-aurora-text-muted">({server.repository.source})</span>
                  )}
                </a>
              )}
              {websiteHref && (
                <a
                  href={websiteHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                >
                  <Globe className="size-3.5" />
                  Website
                </a>
              )}
            </div>
          </Section>
        )}

        {remotes.length > 0 && (
          <Section label={`Remote transports (${remotes.length})`}>
            <div className="space-y-2">
              {remotes.map((r, i) => <RemoteRow key={i} transport={r} />)}
            </div>
          </Section>
        )}

        {remotes.length === 0 && (
          <div className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-text-muted')}>
            This server uses stdio transport only and cannot be added to a gateway.
          </div>
        )}

        {packages.length > 0 && (
          <Section label={`Packages (${packages.length})`}>
            <div className="space-y-3">
              {packages.map((p, i) => <PackageCard key={i} pkg={p} />)}
            </div>
          </Section>
        )}

        {extensions && (
          <Section label="Registry metadata">
            <dl className={cn(AURORA_MEDIUM_PANEL, 'grid grid-cols-1 gap-x-6 gap-y-2 p-4 text-xs sm:grid-cols-2')}>
              <MetaRow label="Published" value={extensions.publishedAt} />
              <MetaRow label="Updated" value={extensions.updatedAt ?? undefined} />
              <MetaRow label="Status changed" value={extensions.statusChangedAt} />
              <MetaRow label="Status" value={extensions.status} />
            </dl>
          </Section>
        )}
      </div>

      <div className="shrink-0 border-t border-aurora-border-strong/60 px-6 py-4">
        {isHTTP ? (
          <Button className="w-full" onClick={() => setInstallOpen(true)}>
            Install to Gateway
          </Button>
        ) : (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="w-full">
                  <Button disabled className="w-full" tabIndex={-1}>
                    Install to Gateway
                  </Button>
                </span>
              </TooltipTrigger>
              <TooltipContent>
                This server uses stdio transport only and cannot be installed to a gateway
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>

      <InstallDialog
        server={installOpen ? server : null}
        onClose={() => setInstallOpen(false)}
      />
    </>
  )
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      {children}
    </div>
  )
}

function MetaRow({ label, value }: { label: string; value?: string | null }) {
  if (!value) return null
  return (
    <div className="flex flex-col gap-0.5">
      <dt className={AURORA_MUTED_LABEL}>{label}</dt>
      <dd className="font-mono text-xs text-aurora-text-secondary">{value}</dd>
    </div>
  )
}

function RemoteRow({ transport }: { transport: Transport }) {
  const headers = transport.headers ?? []
  const variables = transport.variables ?? null
  const hasVariables = variables && Object.keys(variables).length > 0

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-2 p-3 text-sm')}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full bg-aurora-accent-strong/15 px-2 py-0.5 text-xs font-medium text-aurora-accent-strong">
          {transport.type}
        </span>
      </div>
      {transport.url && (
        <p className="break-all font-mono text-xs text-aurora-text-muted">{transport.url}</p>
      )}
      {headers.length > 0 && (
        <div className="space-y-1">
          <p className={AURORA_MUTED_LABEL}>Headers</p>
          <ul className="space-y-0.5">
            {headers.map((h, i) => (
              <li key={i} className="font-mono text-xs text-aurora-text-secondary">
                <span className="text-aurora-text-primary">{h.name}:</span> {h.value}
              </li>
            ))}
          </ul>
        </div>
      )}
      {hasVariables && (
        <div className="space-y-1">
          <p className={AURORA_MUTED_LABEL}>Variables</p>
          <pre className="overflow-x-auto rounded border border-aurora-border-strong/40 bg-[rgba(7,17,26,0.6)] p-2 font-mono text-xs text-aurora-text-secondary">
            {JSON.stringify(variables, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}

function PackageCard({ pkg }: { pkg: RegistryPackage }) {
  const runtimeArgs = pkg.runtimeArguments ?? []
  const packageArgs = pkg.packageArguments ?? []
  const envVars = pkg.environmentVariables ?? []

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-3 text-sm')}>
      <div className="space-y-1">
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="font-medium text-aurora-text-primary">{pkg.identifier}</span>
          {pkg.version && (
            <span className="text-xs text-aurora-text-muted">v{pkg.version}</span>
          )}
          <span className="rounded-full border border-aurora-border-strong/60 px-2 py-0.5 text-[11px] text-aurora-text-muted">
            {pkg.registryType}
          </span>
          {pkg.runtimeHint && (
            <span className="rounded-full bg-aurora-border-strong/30 px-2 py-0.5 text-[11px] text-aurora-text-muted">
              {pkg.runtimeHint}
            </span>
          )}
        </div>
        {pkg.registryBaseUrl && (
          <p className="break-all font-mono text-[11px] text-aurora-text-muted">
            {pkg.registryBaseUrl}
          </p>
        )}
        {pkg.fileSha256 && (
          <p className="break-all font-mono text-[11px] text-aurora-text-muted">
            sha256: {pkg.fileSha256}
          </p>
        )}
      </div>

      <div className="space-y-1">
        <p className={AURORA_MUTED_LABEL}>Transport</p>
        <div className="font-mono text-xs text-aurora-text-secondary">
          <span className="text-aurora-text-primary">{pkg.transport.type}</span>
          {pkg.transport.url && <> · {pkg.transport.url}</>}
        </div>
      </div>

      {runtimeArgs.length > 0 && <ArgsList label="Runtime arguments" items={runtimeArgs} renderItem={renderRuntimeArg} />}
      {packageArgs.length > 0 && <ArgsList label="Package arguments" items={packageArgs} renderItem={renderPackageArg} />}
      {envVars.length > 0 && <ArgsList label="Environment variables" items={envVars} renderItem={renderEnvVar} />}
    </div>
  )
}

function ArgsList<T>({
  label,
  items,
  renderItem,
}: {
  label: string
  items: T[]
  renderItem: (item: T, i: number) => React.ReactNode
}) {
  return (
    <div className="space-y-1">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <ul className="space-y-1">{items.map(renderItem)}</ul>
    </div>
  )
}

function renderRuntimeArg(arg: RuntimeArgument, i: number) {
  return (
    <li key={i} className="font-mono text-xs text-aurora-text-secondary">
      <span className="text-aurora-text-primary">{arg.name}</span>
      {arg.value && <> = {arg.value}</>}
    </li>
  )
}

function renderPackageArg(arg: PackageArgument, i: number) {
  return (
    <li key={i} className="text-xs">
      <div className="flex items-baseline gap-2">
        <span className="font-mono text-aurora-text-primary">{arg.value}</span>
        {arg.isRequired && (
          <span className="rounded-full bg-aurora-warn/15 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider text-aurora-warn">
            required
          </span>
        )}
      </div>
      {arg.description && (
        <p className="mt-0.5 text-aurora-text-muted">{arg.description}</p>
      )}
    </li>
  )
}

function renderEnvVar(env: EnvironmentVariable, i: number) {
  return (
    <li key={i} className="text-xs">
      <div className="flex items-baseline gap-2">
        <span className="font-mono text-aurora-text-primary">{env.name}</span>
        {env.required && (
          <span className="rounded-full bg-aurora-warn/15 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider text-aurora-warn">
            required
          </span>
        )}
      </div>
      {env.description && (
        <p className="mt-0.5 text-aurora-text-muted">{env.description}</p>
      )}
    </li>
  )
}
