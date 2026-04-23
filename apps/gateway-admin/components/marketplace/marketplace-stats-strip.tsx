'use client'

import type { ReactNode } from 'react'
import { Download, ShoppingBag, RefreshCw } from 'lucide-react'
import type { Plugin, Marketplace } from '@/lib/types/marketplace'

interface MarketplaceStatsStripProps {
  plugins: Plugin[]
  marketplaces: Marketplace[]
  installedIds: Set<string>
  variant: 'browse' | 'marketplaces'
}

interface ChipProps {
  value: ReactNode
  label: string
  icon: ReactNode
  iconBg: string
  iconColor: string
  valueColor?: string
}

function StatChip({ value, label, icon, iconBg, iconColor, valueColor }: ChipProps) {
  return (
    <div className="flex items-center gap-[5px] px-[10px] py-[5px] rounded-[11px] transition-colors duration-150 hover:bg-aurora-hover-bg cursor-default">
      <div
        className="w-[18px] h-[18px] rounded-[5px] flex-shrink-0 flex items-center justify-center"
        style={{ background: iconBg, color: iconColor }}
      >
        <span className="w-[11px] h-[11px] [&>svg]:w-full [&>svg]:h-full">{icon}</span>
      </div>
      <span
        className="font-display text-[14px] font-extrabold tracking-[-0.03em] tabular-nums text-aurora-text-primary leading-none"
        style={valueColor ? { color: valueColor } : undefined}
      >
        {value}
      </span>
      <span className="text-[11px] font-medium text-aurora-text-muted leading-none hidden sm:inline">
        {label}
      </span>
    </div>
  )
}

function Divider() {
  return <div className="w-px h-[22px] bg-aurora-border-default flex-shrink-0 mx-px" />
}

export function MarketplaceStatsStrip({
  plugins,
  marketplaces,
  installedIds,
  variant,
}: MarketplaceStatsStripProps) {
  const installed = plugins.filter(p => installedIds.has(p.id))
  const updates = installed.filter(p => p.hasUpdate)
  const autoUpdateCount = marketplaces.filter(m => m.autoUpdate).length

  return (
    <div className="flex items-center gap-0.5 ml-auto flex-shrink-0 bg-aurora-control-surface border border-aurora-border-default rounded-aurora-1 p-0.5 overflow-hidden shadow-[var(--aurora-shadow-small),var(--aurora-highlight-medium)]">
      {variant === 'marketplaces' ? (
        <>
          <StatChip
            value={marketplaces.length}
            label="marketplaces"
            icon={<ShoppingBag />}
            iconBg="color-mix(in srgb, var(--aurora-accent-primary) 15%, transparent)"
            iconColor="var(--aurora-accent-primary)"
          />
          <Divider />
          <StatChip
            value={plugins.length}
            label="plugins"
            icon={<Download />}
            iconBg="color-mix(in srgb, var(--aurora-accent-strong) 12%, transparent)"
            iconColor="var(--aurora-accent-strong)"
          />
          <Divider />
          <StatChip
            value={autoUpdateCount}
            label="auto-update"
            icon={<RefreshCw />}
            iconBg="color-mix(in srgb, var(--aurora-success) 12%, transparent)"
            iconColor="var(--aurora-success)"
          />
        </>
      ) : (
        <>
          <StatChip
            value={(
              <>
                {installed.length}
                <span className="ml-0.5 text-[11px] font-medium text-aurora-text-muted">
                  /{plugins.length}
                </span>
              </>
            )}
            label="installed"
            icon={<Download />}
            iconBg="color-mix(in srgb, var(--aurora-accent-primary) 15%, transparent)"
            iconColor="var(--aurora-accent-primary)"
          />
          <Divider />
          <StatChip
            value={marketplaces.length}
            label="sources"
            icon={<ShoppingBag />}
            iconBg="color-mix(in srgb, var(--aurora-accent-strong) 12%, transparent)"
            iconColor="var(--aurora-accent-strong)"
          />
          <Divider />
          <StatChip
            value={updates.length}
            label={updates.length ? 'updates' : 'up to date'}
            icon={<RefreshCw />}
            iconBg={updates.length
              ? 'color-mix(in srgb, var(--aurora-warn) 15%, transparent)'
              : 'color-mix(in srgb, var(--aurora-border-default) 40%, transparent)'}
            iconColor={updates.length ? 'var(--aurora-warn)' : 'var(--aurora-text-muted)'}
            valueColor={updates.length ? 'var(--aurora-warn)' : undefined}
          />
        </>
      )}
    </div>
  )
}
