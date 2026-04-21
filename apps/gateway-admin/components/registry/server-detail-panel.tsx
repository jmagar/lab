'use client'

import { useState } from 'react'
import { formatDistanceToNow } from 'date-fns'
import {
  Check,
  CheckCircle2,
  Copy,
  ExternalLink,
  FileCode,
  Globe,
  Package,
} from 'lucide-react'
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
import {
  formatLocalDateTimePrecise,
  formatUtcTooltip,
} from '@/lib/utils/format-time'
import { cn } from '@/lib/utils'
import { AURORA_MEDIUM_PANEL, AURORA_MUTED_LABEL } from '@/components/gateway/gateway-theme'
import { InstallDialog } from './install-dialog'
import { RegistryStatusBadge } from './registry-status-badge'
import type {
  EnvironmentVariable,
  Header,
  Icon as RegistryIcon,
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
  const primaryIcon = icons[0] ?? null
  const extraIcons = icons.slice(1)
  const repoHref = safeHref(server.repository?.url)
  const websiteHref = safeHref(server.websiteUrl)
  const schemaHref = safeHref(server.$schema)
  const status = extensions?.status ?? null
  const statusMessage = extensions?.statusMessage ?? null

  return (
    <>
      <DialogHeader className="shrink-0 space-y-0 border-b border-aurora-border-strong/60 px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-12 shrink-0 items-center justify-center rounded-xl border border-aurora-border-strong/60 bg-[rgba(14,31,44,0.8)]">
            {primaryIcon ? (
              <img
                src={safeHref(primaryIcon.src) ?? undefined}
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
            <div className="mt-0.5 flex items-center gap-1.5">
              <p className="break-all font-mono text-xs text-aurora-text-muted">{server.name}</p>
              <CopyButton value={server.name} label="Copy server name" />
            </div>
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
              {extensions?.updatedAt && <TimestampPill iso={extensions.updatedAt} />}
              <RegistryStatusBadge status={status} />
            </div>
            {statusMessage && (
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

        {(repoHref || websiteHref || schemaHref || server.repository?.id || server.repository?.subfolder) && (
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
              {schemaHref && (
                <a
                  href={schemaHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                  title={server.$schema ?? undefined}
                >
                  <FileCode className="size-3.5" />
                  Schema
                </a>
              )}
            </div>
            {(server.repository?.id || server.repository?.subfolder) && (
              <dl className={cn(AURORA_MEDIUM_PANEL, 'mt-2 grid grid-cols-1 gap-x-6 gap-y-2 p-3 text-xs sm:grid-cols-2')}>
                {server.repository?.id && <MetaRow label="Repository ID" value={server.repository.id} mono copy />}
                {server.repository?.subfolder && <MetaRow label="Subfolder" value={server.repository.subfolder} mono />}
              </dl>
            )}
          </Section>
        )}

        {extraIcons.length > 0 && (
          <Section label={`Icons (${icons.length})`}>
            <div className="flex flex-wrap gap-2">
              {icons.map((icon, i) => <IconChip key={i} icon={icon} />)}
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
              <TimeRow label="Published" iso={extensions.publishedAt} />
              <TimeRow label="Updated" iso={extensions.updatedAt} />
              <TimeRow label="Status changed" iso={extensions.statusChangedAt} />
              <MetaRow label="Status" value={extensions.status} />
              <MetaRow label="Is latest" value={extensions.isLatest ? 'yes' : 'no'} />
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

function MetaRow({
  label,
  value,
  mono,
  copy,
}: {
  label: string
  value?: string | null
  mono?: boolean
  copy?: boolean
}) {
  if (!value) return null
  return (
    <div className="flex flex-col gap-0.5">
      <dt className={AURORA_MUTED_LABEL}>{label}</dt>
      <dd className={cn('flex items-center gap-1.5 text-xs text-aurora-text-secondary', mono && 'font-mono')}>
        <span className="break-all">{value}</span>
        {copy && <CopyButton value={value} label={`Copy ${label.toLowerCase()}`} />}
      </dd>
    </div>
  )
}

function TimeRow({ label, iso }: { label: string; iso?: string | null }) {
  const local = formatLocalDateTimePrecise(iso)
  const utc = formatUtcTooltip(iso)
  if (!local) return null
  return (
    <div className="flex flex-col gap-0.5">
      <dt className={AURORA_MUTED_LABEL}>{label}</dt>
      <dd className="font-mono text-xs text-aurora-text-secondary" title={utc ?? undefined}>
        {local}
      </dd>
    </div>
  )
}

function TimestampPill({ iso }: { iso: string }) {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return null
  const tooltip = [formatLocalDateTimePrecise(iso), formatUtcTooltip(iso)]
    .filter(Boolean)
    .join(' · ')
  return (
    <span
      title={tooltip}
      className="cursor-default rounded-full border border-aurora-border-strong/40 px-2 py-0.5 text-xs text-aurora-text-muted"
    >
      {formatDistanceToNow(d, { addSuffix: true })}
    </span>
  )
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={async (e) => {
        e.stopPropagation()
        try {
          await navigator.clipboard.writeText(value)
          setCopied(true)
          setTimeout(() => setCopied(false), 1200)
        } catch {
          // clipboard access denied
        }
      }}
      className="inline-flex size-5 shrink-0 items-center justify-center rounded text-aurora-text-muted hover:bg-aurora-border-strong/40 hover:text-aurora-text-primary"
    >
      {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
    </button>
  )
}

function IconChip({ icon }: { icon: RegistryIcon }) {
  const href = safeHref(icon.src)
  const label = [icon.theme, icon.sizes?.join(', '), icon.mimeType]
    .filter(Boolean)
    .join(' · ')
  return (
    <div
      className={cn(AURORA_MEDIUM_PANEL, 'flex items-center gap-2 px-2 py-1.5 text-xs')}
      title={icon.src}
    >
      {href ? (
        <img
          src={href}
          alt=""
          className="size-6 rounded object-contain"
          referrerPolicy="no-referrer"
          loading="lazy"
          onError={(e) => { e.currentTarget.style.display = 'none' }}
        />
      ) : (
        <Package className="size-4 text-aurora-text-muted" />
      )}
      <span className="font-mono text-[11px] text-aurora-text-secondary">
        {label || 'icon'}
      </span>
    </div>
  )
}

function RemoteRow({ transport }: { transport: Transport }) {
  const headers: Header[] = transport.headers ?? []
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
        <div className="flex items-center gap-1.5">
          <p className="break-all font-mono text-xs text-aurora-text-muted">{transport.url}</p>
          <CopyButton value={transport.url} label="Copy URL" />
        </div>
      )}
      {headers.length > 0 && (
        <div className="space-y-1">
          <p className={AURORA_MUTED_LABEL}>Headers ({headers.length})</p>
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
          <CopyButton value={pkg.identifier} label="Copy identifier" />
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
          <div className="flex items-center gap-1.5">
            <p className="break-all font-mono text-[11px] text-aurora-text-muted">
              {pkg.registryBaseUrl}
            </p>
            <CopyButton value={pkg.registryBaseUrl} label="Copy registry URL" />
          </div>
        )}
        {pkg.fileSha256 && (
          <div className="flex items-center gap-1.5">
            <p className="break-all font-mono text-[11px] text-aurora-text-muted">
              sha256: {pkg.fileSha256}
            </p>
            <CopyButton value={pkg.fileSha256} label="Copy sha256" />
          </div>
        )}
      </div>

      <div className="space-y-1">
        <p className={AURORA_MUTED_LABEL}>Transport</p>
        <div className="flex items-center gap-1.5 font-mono text-xs text-aurora-text-secondary">
          <span className="text-aurora-text-primary">{pkg.transport.type}</span>
          {pkg.transport.url && (
            <>
              <span>·</span>
              <span className="break-all">{pkg.transport.url}</span>
              <CopyButton value={pkg.transport.url} label="Copy URL" />
            </>
          )}
        </div>
      </div>

      {runtimeArgs.length > 0 && <ArgsList label={`Runtime arguments (${runtimeArgs.length})`} items={runtimeArgs} renderItem={renderRuntimeArg} />}
      {packageArgs.length > 0 && <ArgsList label={`Package arguments (${packageArgs.length})`} items={packageArgs} renderItem={renderPackageArg} />}
      {envVars.length > 0 && <ArgsList label={`Environment variables (${envVars.length})`} items={envVars} renderItem={renderEnvVar} />}
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

