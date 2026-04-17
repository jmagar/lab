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
    <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
      <div className="relative w-full sm:max-w-sm sm:flex-1">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
        <Input
          placeholder="Search gateways..."
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-9"
        />
      </div>

      <div className="grid w-full gap-3 sm:w-auto sm:grid-cols-3">
      <Select value={healthFilter} onValueChange={(v) => onHealthFilterChange(v as HealthFilter)}>
        <SelectTrigger className="w-full sm:w-[160px]">
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

      <Select value={connectionFilter} onValueChange={(v) => onConnectionFilterChange(v as ConnectionFilter)}>
        <SelectTrigger className="w-full sm:w-[160px]">
          <SelectValue placeholder="Connection" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All connections</SelectItem>
          <SelectItem value="connected">Connected</SelectItem>
          <SelectItem value="disconnected">Disconnected</SelectItem>
        </SelectContent>
      </Select>

      <Select value={typeFilter} onValueChange={(v) => onTypeFilterChange(v as GatewayTypeFilter)}>
        <SelectTrigger className="w-full sm:w-[140px]">
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

      {hasFilters && (
        <Button variant="ghost" size="sm" onClick={clearFilters} className="h-9 self-start px-2 sm:self-auto">
          <X className="size-4 mr-1" />
          Clear
        </Button>
      )}
    </div>
  )
}
