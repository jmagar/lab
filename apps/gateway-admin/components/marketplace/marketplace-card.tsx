'use client'

import { cn } from '@/lib/utils'
import type { Plugin } from '@/lib/types/marketplace'

interface MarketplaceCardProps {
  plugin: Plugin
  ghUser?: string
  selected?: boolean
  onClick: () => void
}

function PluginAvatar({ ghUser, name }: { ghUser?: string; name: string }) {
  const initials = name
    .replace(/-/g, ' ')
    .split(' ')
    .filter(Boolean)
    .map(w => w[0])
    .join('')
    .toUpperCase()
    .slice(0, 2)

  if (!ghUser) {
    return (
      <div className="flex items-center justify-center w-10 h-10 rounded-[11px] flex-shrink-0 border border-white/[0.06] bg-aurora-panel-medium font-display text-sm font-black text-aurora-text-muted">
        {initials}
      </div>
    )
  }

  return (
    <div className="w-10 h-10 rounded-[11px] flex-shrink-0 overflow-hidden border border-white/[0.06]">
      <img
        src={`https://github.com/${ghUser}.png?size=80`}
        alt={ghUser}
        className="w-full h-full object-cover"
        onError={e => {
          const el = e.currentTarget
          el.style.display = 'none'
          const fallback = el.parentElement
          if (fallback) fallback.textContent = initials
        }}
      />
    </div>
  )
}

function StatusBadge({ plugin }: { plugin: Plugin }) {
  if (!plugin.installed) return null
  if (plugin.hasUpdate) {
    return (
      <span className="inline-flex items-center gap-[5px] text-[11px] font-bold text-aurora-warn bg-[color-mix(in_srgb,var(--aurora-warn)_7%,transparent)] border border-[color-mix(in_srgb,var(--aurora-warn)_25%,transparent)] rounded-full px-[10px] py-[3px] whitespace-nowrap">
        <span className="w-[5px] h-[5px] rounded-full bg-current flex-shrink-0" />
        Update
      </span>
    )
  }
  return (
    <span className="inline-flex items-center gap-[5px] text-[11px] font-bold text-aurora-success bg-[color-mix(in_srgb,var(--aurora-success)_7%,transparent)] border border-[color-mix(in_srgb,var(--aurora-success)_22%,transparent)] rounded-full px-[10px] py-[3px] whitespace-nowrap">
      <span className="w-[5px] h-[5px] rounded-full bg-current flex-shrink-0" />
      Installed
    </span>
  )
}

export function MarketplaceCard({ plugin, ghUser, selected, onClick }: MarketplaceCardProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') onClick() }}
      className={cn(
        'relative overflow-hidden rounded-aurora-3 border p-[18px] cursor-pointer',
        'flex flex-col gap-3',
        'bg-aurora-panel-medium border-aurora-border-strong',
        'shadow-aurora-medium',
        'transition-[border-color,background,box-shadow,transform] duration-150',
        'before:absolute before:inset-0 before:rounded-aurora-3 before:pointer-events-none',
        'before:bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-text-primary)_1.5%,transparent)_0%,transparent_60%)]',
        'hover:-translate-y-px hover:bg-aurora-panel-strong hover:border-aurora-accent-deep hover:shadow-aurora-strong',
        selected && 'border-aurora-accent-primary bg-aurora-panel-strong shadow-aurora-strong',
      )}
    >
      <div className="flex items-center gap-3">
        <PluginAvatar ghUser={ghUser} name={plugin.name} />
        <div className="flex-1 min-w-0">
          <div className="font-display text-[14px] font-extrabold tracking-[-0.02em] text-aurora-text-primary truncate">
            {plugin.name}
          </div>
          <div className="text-[11px] text-aurora-text-muted mt-0.5 font-medium">{plugin.mkt}</div>
        </div>
      </div>

      <p className="text-[13px] text-aurora-text-muted leading-[1.55] line-clamp-2">
        {plugin.desc}
      </p>

      {plugin.tags.length > 0 && (
        <div className="flex gap-1 flex-wrap">
          {plugin.tags.slice(0, 3).map(t => (
            <span
              key={t}
              className="text-[10px] font-bold uppercase tracking-[0.14em] px-[9px] py-[3px] rounded-full bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default leading-[1.2]"
            >
              {t}
            </span>
          ))}
        </div>
      )}

      <div className="flex items-center justify-between gap-2">
        <span className="text-[11px] font-semibold bg-aurora-control-surface text-aurora-text-muted border border-aurora-border-default rounded-full px-[10px] py-[3px]">
          v{plugin.ver}
        </span>
        <StatusBadge plugin={plugin} />
      </div>
    </div>
  )
}
