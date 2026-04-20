'use client'

import { useState } from 'react'
import { Package, ExternalLink, Globe } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
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
import type { ServerJSON } from '@/lib/types/registry'

interface ServerDetailPanelProps {
  server: ServerJSON | null
  updatedAt?: string | null
  status?: 'active' | 'deprecated' | 'deleted' | null
  statusMessage?: string | null
  onClose: () => void
}

const dateFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
})

function formatRelativeTime(isoString: string): string {
  const now = Date.now()
  let past: number
  try {
    past = new Date(isoString).getTime()
  } catch {
    return isoString
  }
  if (Number.isNaN(past)) return isoString

  const diffMs = now - past
  const diffSecs = Math.floor(diffMs / 1000)
  if (diffSecs < 60) return 'just now'

  const diffMins = Math.floor(diffSecs / 60)
  if (diffMins < 60) return diffMins === 1 ? '1 minute ago' : `${diffMins} minutes ago`

  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return diffHours === 1 ? '1 hour ago' : `${diffHours} hours ago`

  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 30) return diffDays === 1 ? '1 day ago' : `${diffDays} days ago`

  const diffMonths = Math.floor(diffDays / 30)
  if (diffMonths < 12) return diffMonths === 1 ? '1 month ago' : `${diffMonths} months ago`

  const diffYears = Math.floor(diffMonths / 12)
  return diffYears === 1 ? '1 year ago' : `${diffYears} years ago`
}

export function ServerDetailPanel({ server, updatedAt, status, statusMessage, onClose }: ServerDetailPanelProps) {
  const open = server !== null

  return (
    <Sheet open={open} onOpenChange={(v) => { if (!v) onClose() }}>
      <SheetContent className="flex w-full flex-col gap-0 overflow-y-auto sm:max-w-lg" side="right">
        {server && <PanelBody server={server} updatedAt={updatedAt} status={status} statusMessage={statusMessage} />}
      </SheetContent>
    </Sheet>
  )
}

function PanelBody({ server, updatedAt, status, statusMessage }: { server: ServerJSON; updatedAt?: string | null; status?: 'active' | 'deprecated' | 'deleted' | null; statusMessage?: string | null }) {
  const displayName = server.title ?? server.name
  const isHTTP = server.remotes.length > 0
  const [installOpen, setInstallOpen] = useState(false)
  const icon = server.icons.find((ic) => ic.type === 'icon')
  const repoHref = safeHref(server.repository?.url)
  const websiteHref = safeHref(server.websiteUrl)

  return (
    <>
      <SheetHeader className="border-b px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-12 shrink-0 items-center justify-center rounded-xl border border-aurora-border-strong/60 bg-[rgba(14,31,44,0.8)]">
            {icon ? (
              <img
                src={icon.url}
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
            <SheetTitle className="text-base font-semibold leading-tight">
              {displayName}
            </SheetTitle>
            <p className="mt-0.5 font-mono text-xs text-aurora-text-muted">{server.name}</p>
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
              {updatedAt && (
                <span
                  title={updatedAt}
                  className="rounded-full border border-aurora-border-strong/40 px-2 py-0.5 text-xs text-aurora-text-muted cursor-default"
                >
                  {formatRelativeTime(updatedAt)}
                </span>
              )}
              {status === 'deprecated' && (
                <span className="rounded-full border border-aurora-warn/20 bg-aurora-warn/15 px-2 py-0.5 text-xs font-medium text-aurora-warn">
                  Deprecated
                </span>
              )}
              {status === 'deleted' && (
                <span className="rounded-full border border-aurora-error/20 bg-aurora-error/15 px-2 py-0.5 text-xs font-medium text-aurora-error">
                  Deleted
                </span>
              )}
            </div>
            {statusMessage && (status === 'deprecated' || status === 'deleted') && (
              <p className="mt-2 text-xs leading-relaxed text-aurora-text-secondary">
                {statusMessage}
              </p>
            )}
          </div>
        </div>
      </SheetHeader>

      <div className="flex-1 space-y-6 px-6 py-5">
        {/* Description */}
        <div className="space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Description</p>
          {/* untrusted registry data — do not use dangerouslySetInnerHTML */}
          <p className="text-sm leading-relaxed text-aurora-text-secondary">{server.description}</p>
        </div>

        {/* Links */}
        {(repoHref || websiteHref) && (
          <div className="space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Links</p>
            <div className="flex flex-wrap gap-2">
              {repoHref && (
                <a
                  href={repoHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                >
                  <ExternalLink className="size-3.5" />
                  Repository
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
          </div>
        )}

        {/* Remotes */}
        {server.remotes.length > 0 && (
          <div className="space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Remote transports</p>
            <div className="space-y-2">
              {server.remotes.map((remote, i) => (
                <div key={i} className={cn(AURORA_MEDIUM_PANEL, 'p-3 text-sm')}>
                  <span className="font-medium text-aurora-text-primary">{remote.type}</span>
                  {remote.url && (
                    <p
                      className="mt-1 break-all font-mono text-xs text-aurora-text-muted"
                      style={{ wordBreak: 'break-all' }}
                    >
                      {remote.url}
                    </p>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* No remotes banner */}
        {server.remotes.length === 0 && (
          <div className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-text-muted')}>
            This server uses stdio transport only and cannot be added to a gateway.
          </div>
        )}

        {/* Packages */}
        {server.packages.length > 0 && (
          <div className="space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Packages ({server.packages.length})</p>
            <div className="space-y-2">
              {server.packages.map((pkg, i) => (
                <div key={i} className={cn(AURORA_MEDIUM_PANEL, 'p-3 text-sm')}>
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-aurora-text-primary">{pkg.name}</span>
                    {pkg.version && (
                      <span className="text-xs text-aurora-text-muted">v{pkg.version}</span>
                    )}
                  </div>
                  {pkg.command && (
                    <p className="mt-1 font-mono text-xs text-aurora-text-muted">{pkg.command}</p>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Install button */}
      <div className="border-t px-6 py-4">
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
