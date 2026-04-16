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

      <div className="grid w-full gap-3 sm:w-auto sm:grid-cols-2">
      <Select value={healthFilter} onValueChange={(v) => onHealthFilterChange(v as HealthFilter)}>
        <SelectTrigger className="w-full sm:w-[160px]">
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
        <SelectTrigger className="w-full sm:w-[140px]">
          <SelectValue placeholder="Transport" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All types</SelectItem>
          <SelectItem value="http">HTTP</SelectItem>
          <SelectItem value="stdio">stdio</SelectItem>
          <SelectItem value="lab_service">Lab</SelectItem>
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
