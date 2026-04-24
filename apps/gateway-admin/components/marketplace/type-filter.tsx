'use client'

import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { ItemTypeFilter } from '@/lib/marketplace/types'

interface TypeFilterProps {
  value: ItemTypeFilter
  onChange: (value: ItemTypeFilter) => void
  counts: {
    all: number
    plugin: number
    mcp_server: number
    acp_agent: number
  }
}

const TABS: { id: ItemTypeFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'plugin', label: 'Plugins' },
  { id: 'mcp_server', label: 'MCP Servers' },
  { id: 'acp_agent', label: 'ACP Agents' },
]

export function TypeFilter({ value, onChange, counts }: TypeFilterProps) {
  return (
    <Tabs
      value={value}
      onValueChange={(v) => onChange(v as ItemTypeFilter)}
      className="w-fit"
    >
      <TabsList className="h-auto p-[3px]">
        {TABS.map(({ id, label }) => (
          <TabsTrigger key={id} value={id} className="gap-1.5 px-3 py-1.5 text-[13px]">
            {label}
            <span className="tab-badge rounded-full border border-transparent bg-aurora-control-surface px-[7px] py-px text-[11px] font-bold text-aurora-text-muted transition-[background,color] duration-150 data-[state=active]:bg-[color-mix(in_srgb,var(--aurora-accent-primary)_15%,transparent)] data-[state=active]:text-aurora-accent-primary">
              {counts[id]}
            </span>
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  )
}
