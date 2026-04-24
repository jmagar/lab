'use client'

import { useState } from 'react'
import { ChevronDown, Search, SlidersHorizontal, X } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import {
  AURORA_CONTROL_SURFACE,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  gatewayActionTone,
} from '@/components/gateway/gateway-theme'
interface ServerFiltersProps {
  search: string
  onSearchChange: (value: string) => void
  version: string
  onVersionChange: (value: string) => void
  updatedSince: string
  onUpdatedSinceChange: (value: string) => void
  hiddenOnly: boolean
  onHiddenOnlyChange: (value: boolean) => void
  tag: string
  onTagChange: (value: string) => void
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
  hiddenOnly,
  onHiddenOnlyChange,
  tag,
  onTagChange,
  totalLoaded,
  hasMore,
  isLoading,
}: ServerFiltersProps) {
  const activeExtraCount = (version ? 1 : 0) + (updatedSince ? 1 : 0) + (hiddenOnly ? 1 : 0) + (tag ? 1 : 0)
  const [expanded, setExpanded] = useState(activeExtraCount > 0)
  const hasAny = Boolean(search) || activeExtraCount > 0

  const handleClearAll = () => {
    onSearchChange('')
    onVersionChange('')
    onUpdatedSinceChange('')
    onHiddenOnlyChange(false)
    onTagChange('')
  }

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4')}>
      <div className="flex items-center gap-2">
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
          <Input
            aria-label="Search MCP servers"
            name="registry-search"
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
          aria-label={expanded ? 'Hide registry filters' : 'Open registry filters'}
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
            aria-label="Clear registry filters"
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
              name="registry-version"
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
              name="registry-updated-since"
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
            <p className={AURORA_MUTED_LABEL}>Tag</p>
            <Input
              aria-label="Filter by tag"
              name="registry-tag"
              placeholder="e.g. recommended"
              value={tag}
              onChange={(e) => onTagChange(e.target.value)}
              className={cn(
                AURORA_CONTROL_SURFACE,
                'h-9 border text-aurora-text-primary placeholder:text-aurora-text-muted',
              )}
            />
          </div>

          <div className="min-w-[180px] flex-1 space-y-1.5">
            <p className={AURORA_MUTED_LABEL}>Visibility</p>
            <Button
              type="button"
              variant={hiddenOnly ? 'default' : 'outline'}
              size="sm"
              onClick={() => onHiddenOnlyChange(!hiddenOnly)}
              className="h-9 w-full justify-start"
            >
              Hidden only
            </Button>
          </div>
        </div>
      )}

      {!isLoading && totalLoaded !== undefined && totalLoaded > 0 && (
        <p className="text-xs text-aurora-text-muted">
          {totalLoaded} server{totalLoaded === 1 ? '' : 's'} loaded{hasMore ? ' — scroll for more' : ''}
        </p>
      )}
    </div>
  )
}
