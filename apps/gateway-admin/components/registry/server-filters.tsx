'use client'

import { useState } from 'react'
import { ArrowDownUp, ChevronDown, Search, SlidersHorizontal, X } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import {
  AURORA_CONTROL_SURFACE,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  gatewayActionTone,
} from '@/components/gateway/gateway-theme'
import type { RegistrySortBy, RegistrySortOrder } from '@/lib/types/registry'

const SORT_OPTIONS: { value: RegistrySortBy; label: string }[] = [
  { value: 'updated', label: 'Updated' },
  { value: 'published', label: 'Published' },
  { value: 'name', label: 'Name' },
]

interface ServerFiltersProps {
  search: string
  onSearchChange: (value: string) => void
  version: string
  onVersionChange: (value: string) => void
  updatedSince: string
  onUpdatedSinceChange: (value: string) => void
  sortBy: RegistrySortBy | ''
  onSortByChange: (value: RegistrySortBy | '') => void
  order: RegistrySortOrder
  onOrderChange: (value: RegistrySortOrder) => void
  totalLoaded?: number
  hasMore?: boolean
  isLoading?: boolean
}

export function ServerFilters({
  search,
  onSearchChange,
  version,
  onVersionChange,
  updatedSince,
  onUpdatedSinceChange,
  sortBy,
  onSortByChange,
  order,
  onOrderChange,
  totalLoaded,
  hasMore,
  isLoading,
}: ServerFiltersProps) {
  const activeExtraCount = (version ? 1 : 0) + (updatedSince ? 1 : 0) + (sortBy ? 1 : 0)
  const [expanded, setExpanded] = useState(activeExtraCount > 0)
  const hasAny = Boolean(search) || activeExtraCount > 0

  const handleClearAll = () => {
    onSearchChange('')
    onVersionChange('')
    onUpdatedSinceChange('')
    onSortByChange('')
    onOrderChange('desc')
  }

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4')}>
      <div className="flex items-center gap-2">
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
          <Input
            aria-label="Search MCP servers"
            placeholder="Search servers by name or description"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            className={cn(
              AURORA_CONTROL_SURFACE,
              'h-11 border pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted',
            )}
          />
        </div>

        <Button
          variant="outline"
          size="sm"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
          aria-controls="registry-extra-filters"
          className={cn(
            gatewayActionTone(activeExtraCount > 0 ? 'accent' : 'default'),
            'h-11 shrink-0 gap-2 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
          )}
        >
          <SlidersHorizontal className="size-4" />
          <span className="hidden sm:inline">Filters</span>
          {activeExtraCount > 0 && (
            <span className="inline-flex size-5 items-center justify-center rounded-full bg-aurora-accent-strong/20 text-[11px] font-medium text-aurora-accent-strong">
              {activeExtraCount}
            </span>
          )}
          <ChevronDown
            className={cn('size-3.5 transition-transform', expanded && 'rotate-180')}
          />
        </Button>

        {hasAny && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleClearAll}
            className={cn(
              gatewayActionTone(),
              'h-11 shrink-0 gap-1 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg hover:text-aurora-text-primary',
            )}
          >
            <X className="size-4" />
            <span className="hidden sm:inline">Clear</span>
          </Button>
        )}
      </div>

      {expanded && (
        <div
          id="registry-extra-filters"
          className="flex flex-wrap gap-4 border-t border-aurora-border-strong/40 pt-3"
        >
          <div className="min-w-[180px] flex-1 space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Version</p>
            <Input
              aria-label="Filter by version"
              placeholder="e.g. 1.2.0"
              value={version}
              onChange={(e) => onVersionChange(e.target.value)}
              className={cn(
                AURORA_CONTROL_SURFACE,
                'h-9 border text-aurora-text-primary placeholder:text-aurora-text-muted',
              )}
            />
          </div>

          <div className="min-w-[180px] flex-1 space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Updated since</p>
            <Input
              aria-label="Filter by updated since date"
              type="date"
              value={updatedSince}
              onChange={(e) => onUpdatedSinceChange(e.target.value)}
              className={cn(
                AURORA_CONTROL_SURFACE,
                'h-9 border text-aurora-text-primary placeholder:text-aurora-text-muted',
              )}
            />
          </div>

          <div className="min-w-[180px] flex-1 space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Sort by</p>
            <div className="flex gap-1">
              {SORT_OPTIONS.map(({ value, label }) => (
                <button
                  key={value}
                  type="button"
                  aria-pressed={sortBy === value}
                  onClick={() => onSortByChange(sortBy === value ? '' : value)}
                  className={cn(
                    'h-9 flex-1 rounded border px-2 text-xs transition-colors',
                    sortBy === value
                      ? 'border-aurora-accent-strong/60 bg-aurora-accent-strong/15 text-aurora-accent-strong'
                      : 'border-aurora-border-strong/40 text-aurora-text-muted hover:border-aurora-border-strong hover:text-aurora-text-primary',
                  )}
                >
                  {label}
                </button>
              ))}
              <button
                type="button"
                aria-label={order === 'desc' ? 'Sort descending' : 'Sort ascending'}
                onClick={() => onOrderChange(order === 'desc' ? 'asc' : 'desc')}
                disabled={!sortBy}
                className={cn(
                  'h-9 rounded border px-2 transition-colors disabled:opacity-40',
                  'border-aurora-border-strong/40 text-aurora-text-muted hover:border-aurora-border-strong hover:text-aurora-text-primary',
                )}
              >
                <ArrowDownUp className={cn('size-3.5', order === 'asc' && 'rotate-180')} />
              </button>
            </div>
          </div>
        </div>
      )}

      {!isLoading && totalLoaded !== undefined && totalLoaded > 0 && (
        <p className="text-xs text-aurora-text-muted">
          {totalLoaded} server{totalLoaded === 1 ? '' : 's'} loaded{hasMore ? ' — scroll for more' : ''}
          {sortBy ? ` · sorted by ${sortBy} (${order})` : ''}
        </p>
      )}
    </div>
  )
}
