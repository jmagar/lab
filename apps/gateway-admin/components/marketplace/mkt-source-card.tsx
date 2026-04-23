'use client'

import type { Marketplace } from '@/lib/types/marketplace'
import { cn } from '@/lib/utils'

interface MktSourceCardProps {
  marketplace: Marketplace
  installedCount: number
  onClick: () => void
}

function SourceAvatar({ ghUser, name }: { ghUser: string; name: string }) {
  const initials = name.split(/\s+/).map(w => w[0]).join('').toUpperCase().slice(0, 2)
  return (
    <div className="w-12 h-12 rounded-[14px] flex-shrink-0 overflow-hidden border border-white/[0.06] flex items-center justify-center font-display text-lg font-black text-aurora-text-muted bg-aurora-panel-medium">
      <img
        src={`https://github.com/${ghUser}.png?size=96`}
        alt={ghUser}
        className="w-full h-full object-cover"
        onError={e => {
          e.currentTarget.style.display = 'none'
          const el = e.currentTarget.parentElement
          if (el) el.textContent = initials
        }}
      />
    </div>
  )
}

function sourceLabel(m: Marketplace): string {
  if (m.source === 'github') return m.repo ?? m.repository ?? m.ghUser ?? m.githubOwner ?? 'github'
  if (m.source === 'git') return m.url?.replace('https://github.com/', '').replace('.git', '') ?? m.url ?? ''
  return m.path ?? 'local'
}

export function MktSourceCard({ marketplace: m, installedCount, onClick }: MktSourceCardProps) {
  return (
    <button
      type="button"
      onKeyDown={(event) => {
        if (event.key === ' ') {
          event.preventDefault()
        }
      }}
      onClick={onClick}
      className={cn(
        'rounded-aurora-3 border p-[22px] cursor-pointer',
        'flex flex-col gap-[14px]',
        'bg-aurora-panel-medium border-aurora-border-strong',
        'shadow-aurora-medium',
        'transition-[border-color,background,box-shadow] duration-150',
        'text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/60 focus-visible:ring-offset-2 focus-visible:ring-offset-aurora-page-bg',
        'hover:bg-aurora-panel-strong hover:border-aurora-accent-deep hover:shadow-aurora-strong',
      )}
    >
      <div className="flex items-center gap-[14px]">
        <SourceAvatar ghUser={m.ghUser ?? m.githubOwner ?? 'github'} name={m.name} />
        <div className="flex-1 min-w-0">
          <div className="font-display text-[16px] font-extrabold tracking-[-0.02em] text-aurora-text-primary">
            {m.name}
          </div>
          <div className="text-[12px] text-aurora-text-muted mt-[3px] font-medium">by {m.owner}</div>
        </div>
      </div>

      <p className="text-[13px] text-aurora-text-muted leading-[1.55]">{m.desc}</p>

      <div className="flex items-center gap-[10px] flex-wrap pt-[6px] border-t border-aurora-border-default">
        <span className="text-[12px] text-aurora-text-muted flex items-center gap-[5px]">
          <strong className="text-aurora-text-primary font-bold">{installedCount}</strong> installed
        </span>
        <span className="w-[3px] h-[3px] rounded-full bg-aurora-border-strong flex-shrink-0" />
        <span className="text-[12px] text-aurora-text-muted flex items-center gap-[5px]">
          <strong className="text-aurora-text-primary font-bold">{m.totalPlugins}</strong> available
        </span>
        {m.autoUpdate && (
          <span className="inline-flex items-center gap-1 text-[11px] font-semibold text-aurora-accent-primary whitespace-nowrap ml-auto">
            <span className="w-[5px] h-[5px] rounded-full bg-aurora-accent-primary flex-shrink-0 animate-pulse" />
            auto-update
          </span>
        )}
        <span
          className="text-[11px] font-semibold font-mono px-[9px] py-[3px] rounded-full bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default whitespace-nowrap overflow-hidden text-ellipsis max-w-[180px]"
          title={sourceLabel(m)}
        >
          {sourceLabel(m)}
        </span>
      </div>
    </button>
  )
}
