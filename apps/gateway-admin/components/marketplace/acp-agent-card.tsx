'use client'

import { useState } from 'react'
import { Bot, CheckCircle2, ExternalLink } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { AcpAgent } from '@/lib/marketplace/types'
import { DIST_LABELS } from '@/lib/marketplace/types'
import { AcpAgentInstallModal } from './acp-agent-install-modal'

interface AcpAgentCardProps {
  agent: AcpAgent
}

const META_PILL =
  'inline-flex items-center gap-1 rounded-full border border-aurora-border-default bg-aurora-control-surface px-2 py-0.5 text-[10px] font-semibold text-aurora-text-muted'

function MetaLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      className={cn(META_PILL, 'hover:text-aurora-text-primary')}
    >
      {children}
      <ExternalLink className="size-2.5" />
    </a>
  )
}

function MetaPill({ children }: { children: React.ReactNode }) {
  return <span className={META_PILL}>{children}</span>
}

function getDistributionKey(dist: AcpAgent['distribution']): string | null {
  for (const key of ['npx', 'uvx', 'binary'] as const) {
    if (dist[key] !== undefined) return key
  }
  return null
}

export function AcpAgentCard({ agent }: AcpAgentCardProps) {
  const distKey = getDistributionKey(agent.distribution)
  const distLabel = distKey ? (DIST_LABELS[distKey] ?? distKey) : null
  const [installOpen, setInstallOpen] = useState(false)

  return (
    <>
      <div
        className={cn(
          'relative overflow-hidden rounded-aurora-3 border p-4 sm:p-[18px]',
          'flex flex-col gap-3',
          'bg-aurora-panel-medium border-aurora-border-strong',
          'shadow-aurora-medium',
          'before:absolute before:inset-0 before:rounded-aurora-3 before:pointer-events-none',
          'before:bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-text-primary)_1.5%,transparent)_0%,transparent_60%)]',
        )}
      >
        {/* Header */}
        <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-start gap-3">
          <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-[11px] border border-aurora-border-default bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-panel-medium)_88%,transparent),color-mix(in_srgb,var(--aurora-accent-strong)_10%,transparent))] shadow-[var(--aurora-shadow-small)]">
            <Bot className="w-[18px] h-[18px] text-aurora-accent-strong" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="font-display text-[14px] sm:text-[15px] font-extrabold tracking-[-0.02em] text-aurora-text-primary truncate">
              {agent.name}
            </div>
            <div className="text-[11px] text-aurora-text-muted mt-0.5 font-medium truncate">
              {agent.id}
            </div>
          </div>
          {distLabel && (
            <Badge variant="outline" className="shrink-0 text-[10px] uppercase tracking-[0.12em]">
              {distLabel}
            </Badge>
          )}
          {agent.installed && (
            <Badge variant="outline" className="shrink-0 gap-1 text-[10px] uppercase tracking-[0.12em] text-aurora-success">
              <CheckCircle2 className="size-3" />
              Installed
            </Badge>
          )}
        </div>

        {/* Description */}
        {agent.description && (
          <p className="text-[13px] text-aurora-text-muted leading-[1.55] line-clamp-3">
            {agent.description}
          </p>
        )}

        {(agent.repository || agent.website || agent.license || agent.authors?.length) && (
          <div className="flex flex-wrap gap-1.5">
            {agent.repository && <MetaLink href={agent.repository}>Repo</MetaLink>}
            {agent.website && <MetaLink href={agent.website}>Website</MetaLink>}
            {agent.license && <MetaPill>{agent.license}</MetaPill>}
            {agent.authors?.[0] && <MetaPill>{agent.authors[0]}</MetaPill>}
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between gap-2 border-t border-aurora-border-default pt-2">
          <span className="text-[11px] font-semibold bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default rounded-full px-[10px] py-[3px]">
            v{agent.version}
          </span>
          <button
            type="button"
            onClick={() => setInstallOpen(true)}
            disabled={agent.installed}
            className="inline-flex items-center gap-1.5 rounded-aurora-1 border border-aurora-accent-primary/40 bg-[color-mix(in_srgb,var(--aurora-accent-primary)_10%,transparent)] px-3 py-1.5 text-[12px] font-semibold text-aurora-accent-primary hover:bg-[color-mix(in_srgb,var(--aurora-accent-primary)_16%,transparent)] transition-colors duration-150"
            aria-label={`Install ${agent.name}`}
          >
            {agent.installed ? 'Installed' : 'Install'}
          </button>
        </div>
      </div>

      <AcpAgentInstallModal
        agent={agent}
        open={installOpen}
        onClose={() => setInstallOpen(false)}
      />
    </>
  )
}
