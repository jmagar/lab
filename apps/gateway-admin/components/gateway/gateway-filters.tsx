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
  | 'configured'
  | 'enabled'
  | 'disabled'
  | 'connected'
  | 'disconnected'
export type TransportFilter = 'all' | 'http' | 'stdio' | 'lab_service'

interface GatewayFiltersProps {
  search: string
  onSearchChange: (value: string) => void
  healthFilter: HealthFilter
  onHealthFilterChange: (value: HealthFilter) => void
  transportFilter: TransportFilter
  onTransportFilterChange: (value: TransportFilter) => void
}

export function GatewayFilters({
  search,
  onSearchChange,
  healthFilter,
  onHealthFilterChange,
  transportFilter,
  onTransportFilterChange,
}: GatewayFiltersProps) {
  const hasFilters = search || healthFilter !== 'all' || transportFilter !== 'all'

  const clearFilters = () => {
    onSearchChange('')
    onHealthFilterChange('all')
    onTransportFilterChange('all')
  }

  return (
    <div className="flex items-center gap-3">
      <div className="relative flex-1 max-w-sm">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
        <Input
          placeholder="Search gateways..."
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-9"
        />
      </div>

      <Select value={healthFilter} onValueChange={(v) => onHealthFilterChange(v as HealthFilter)}>
        <SelectTrigger className="w-[140px]">
          <SelectValue placeholder="Health" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All status</SelectItem>
          <SelectItem value="active">Active</SelectItem>
          <SelectItem value="configured">Configured</SelectItem>
          <SelectItem value="enabled">Enabled</SelectItem>
          <SelectItem value="disabled">Disabled</SelectItem>
          <SelectItem value="connected">Connected</SelectItem>
          <SelectItem value="disconnected">Disconnected</SelectItem>
        </SelectContent>
      </Select>

      <Select value={transportFilter} onValueChange={(v) => onTransportFilterChange(v as TransportFilter)}>
        <SelectTrigger className="w-[120px]">
          <SelectValue placeholder="Transport" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All types</SelectItem>
          <SelectItem value="http">HTTP</SelectItem>
          <SelectItem value="stdio">stdio</SelectItem>
          <SelectItem value="lab_service">Lab</SelectItem>
        </SelectContent>
      </Select>

      {hasFilters && (
        <Button variant="ghost" size="sm" onClick={clearFilters} className="h-9 px-2">
          <X className="size-4 mr-1" />
          Clear
        </Button>
      )}
    </div>
  )
}
