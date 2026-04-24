'use client'

import { Bot } from 'lucide-react'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { AcpAgent } from '@/lib/marketplace/types'
import { DIST_LABELS } from '@/lib/marketplace/types'

interface AcpAgentCardProps {
  agent: AcpAgent
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

  return (
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
      </div>

      {/* Description */}
      {agent.description && (
        <p className="text-[13px] text-aurora-text-muted leading-[1.55] line-clamp-3">
          {agent.description}
        </p>
      )}

      {/* Footer */}
      <div className="flex items-center justify-between gap-2 border-t border-aurora-border-default pt-2">
        <span className="text-[11px] font-semibold bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default rounded-full px-[10px] py-[3px]">
          v{agent.version}
        </span>
        <Tooltip>
          <TooltipTrigger asChild>
            <span className="inline-block">
              <button
                type="button"
                disabled
                className="inline-flex items-center gap-1.5 rounded-aurora-1 border border-aurora-border-default bg-aurora-control-surface px-3 py-1.5 text-[12px] font-semibold text-aurora-text-muted opacity-50 cursor-not-allowed"
                aria-label="Install ACP agent (coming soon)"
              >
                Install
              </button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Coming soon</TooltipContent>
        </Tooltip>
      </div>
    </div>
  )
}
