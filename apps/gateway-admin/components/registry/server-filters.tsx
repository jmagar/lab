'use client'

import { Search, X } from 'lucide-react'
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
  totalCount?: number
  isLoading?: boolean
}

export function ServerFilters({
  search,
  onSearchChange,
  version,
  onVersionChange,
  updatedSince,
  onUpdatedSinceChange,
  totalCount,
  isLoading,
}: ServerFiltersProps) {
  const hasExtraFilters = version !== '' || updatedSince !== ''

  const handleClearAll = () => {
    onSearchChange('')
    onVersionChange('')
    onUpdatedSinceChange('')
  }

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4')}>
      {/* Row 1: search */}
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0 flex-1 space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Search</p>
          <div className="relative">
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
        </div>

        {(search || hasExtraFilters) && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleClearAll}
            className={cn(
              gatewayActionTone(),
              'mt-6 h-9 shrink-0 px-3 text-aurora-text-primary hover:bg-[#17364b] hover:text-aurora-text-primary',
            )}
          >
            <X className="mr-1 size-4" />
            Clear
          </Button>
        )}
      </div>

      {/* Row 2: version + updated since */}
      <div className="flex flex-wrap gap-4">
        <div className="min-w-[160px] flex-1 space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Version</p>
          <Input
            aria-label="Filter by version"
            placeholder="Filter by version"
            value={version}
            onChange={(e) => onVersionChange(e.target.value)}
            className={cn(
              AURORA_CONTROL_SURFACE,
              'h-9 border text-aurora-text-primary placeholder:text-aurora-text-muted',
            )}
          />
        </div>

        <div className="min-w-[160px] flex-1 space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Updated since</p>
          <Input
            aria-label="Filter by updated since date"
            type="date"
            placeholder="YYYY-MM-DD"
            value={updatedSince}
            onChange={(e) => onUpdatedSinceChange(e.target.value)}
            className={cn(
              AURORA_CONTROL_SURFACE,
              'h-9 border text-aurora-text-primary placeholder:text-aurora-text-muted',
            )}
          />
        </div>
      </div>

      {!isLoading && totalCount !== undefined && (
        <p className="text-xs text-aurora-text-muted">
          {totalCount === 0 ? 'No servers found' : `${totalCount} server${totalCount === 1 ? '' : 's'}`}
        </p>
      )}
    </div>
  )
}
