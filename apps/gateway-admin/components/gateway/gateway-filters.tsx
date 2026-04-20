'use client'

import { Search, X } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import {
  AURORA_CONTROL_SURFACE,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
} from '@/components/aurora/tokens'
import { gatewayActionTone } from './gateway-theme'

export type HealthFilter =
  | 'all'
  | 'active'
  | 'enabled'
  | 'disabled'
  | 'configured'
export type ConnectionFilter = 'all' | 'connected' | 'disconnected'
export type GatewayTypeFilter = 'all' | 'lab' | 'custom' | 'http' | 'stdio'

interface GatewayFiltersProps {
  search: string
  onSearchChange: (value: string) => void
  healthFilter: HealthFilter
  onHealthFilterChange: (value: HealthFilter) => void
  connectionFilter: ConnectionFilter
  onConnectionFilterChange: (value: ConnectionFilter) => void
  typeFilter: GatewayTypeFilter
  onTypeFilterChange: (value: GatewayTypeFilter) => void
}

export function GatewayFilters({
  search,
  onSearchChange,
  healthFilter,
  onHealthFilterChange,
  connectionFilter,
  onConnectionFilterChange,
  typeFilter,
  onTypeFilterChange,
}: GatewayFiltersProps) {
  const hasFilters =
    search || healthFilter !== 'all' || connectionFilter !== 'all' || typeFilter !== 'all'

  const clearFilters = () => {
    onSearchChange('')
    onHealthFilterChange('all')
    onConnectionFilterChange('all')
    onTypeFilterChange('all')
  }

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-4 p-4')}>
      <div className="flex justify-end">
        {hasFilters && (
          <Button
            variant="outline"
            size="sm"
            onClick={clearFilters}
            className={cn(
              gatewayActionTone(),
              'h-9 px-3 text-aurora-text-primary hover:bg-[#17364b] hover:text-aurora-text-primary',
            )}
          >
            <X className="mr-1 size-4" />
            Clear filters
          </Button>
        )}
      </div>

      <div className="grid gap-3 lg:grid-cols-[minmax(0,1.25fr)_repeat(3,minmax(0,188px))]">
        <div className="space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Search</p>
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
            <Input
              placeholder="Search gateways, commands, or endpoints"
              value={search}
              onChange={(e) => onSearchChange(e.target.value)}
              className={cn(
                AURORA_CONTROL_SURFACE,
                'h-11 border pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted',
              )}
            />
          </div>
        </div>

        <div className="space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>State</p>
          <Select value={healthFilter} onValueChange={(v) => onHealthFilterChange(v as HealthFilter)}>
            <SelectTrigger className={cn(AURORA_CONTROL_SURFACE, 'h-11 w-full border text-aurora-text-primary')}>
              <SelectValue placeholder="State" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All states</SelectItem>
              <SelectItem value="active">Active</SelectItem>
              <SelectItem value="enabled">Enabled</SelectItem>
              <SelectItem value="disabled">Deactivated</SelectItem>
              <SelectItem value="configured">Configured</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Connection</p>
          <Select value={connectionFilter} onValueChange={(v) => onConnectionFilterChange(v as ConnectionFilter)}>
            <SelectTrigger className={cn(AURORA_CONTROL_SURFACE, 'h-11 w-full border text-aurora-text-primary')}>
              <SelectValue placeholder="Connection" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All connections</SelectItem>
              <SelectItem value="connected">Connected</SelectItem>
              <SelectItem value="disconnected">Disconnected</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <p className={AURORA_MUTED_LABEL}>Type</p>
          <Select value={typeFilter} onValueChange={(v) => onTypeFilterChange(v as GatewayTypeFilter)}>
            <SelectTrigger className={cn(AURORA_CONTROL_SURFACE, 'h-11 w-full border text-aurora-text-primary')}>
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All types</SelectItem>
              <SelectItem value="lab">Lab</SelectItem>
              <SelectItem value="custom">Custom</SelectItem>
              <SelectItem value="http">HTTP</SelectItem>
              <SelectItem value="stdio">stdio</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
