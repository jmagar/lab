'use client'

import { Search, X } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { cn } from '@/lib/utils'
import {
  AURORA_CONTROL_SURFACE,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  pillTone,
} from '@/components/aurora/tokens'
import { gatewayActionTone } from './gateway-theme'
import type {
  GatewaySourceFacet,
  GatewayStatusFacet,
  GatewayTransportFacet,
  ToolFilterState,
  ToolsExposureFilter,
} from './gateway-list-state'

export interface GatewayFiltersProps {
  mode: 'gateways' | 'tools'
  search: string
  gatewayFilters: {
    status: GatewayStatusFacet[]
    source: GatewaySourceFacet[]
    transport: GatewayTransportFacet[]
  }
  toolFilters: ToolFilterState
  gatewayOptions: Array<{ value: string; label: string }>
  mobileSheetOpen: boolean
  onMobileSheetOpenChange: (open: boolean) => void
  onSearchChange: (value: string) => void
  onGatewayFilterToggle: (group: 'status' | 'source' | 'transport', value: string) => void
  onToolFilterToggle: (group: 'gatewayIds' | 'source' | 'transport', value: string) => void
  onExposureChange: (value: ToolsExposureFilter) => void
  onClearFilters: () => void
}

interface FilterCheckboxProps {
  checked: boolean
  label: string
  onChange: () => void
}

const GATEWAY_STATUS_OPTIONS: Array<{ value: GatewayStatusFacet; label: string }> = [
  { value: 'configured', label: 'Configured' },
  { value: 'healthy', label: 'Healthy' },
  { value: 'disconnected', label: 'Disconnected' },
  { value: 'enabled', label: 'Enabled' },
  { value: 'disabled', label: 'Disabled' },
]

const SOURCE_OPTIONS: Array<{ value: GatewaySourceFacet; label: string }> = [
  { value: 'lab', label: 'Lab' },
  { value: 'custom', label: 'Custom' },
]

const TRANSPORT_OPTIONS: Array<{ value: GatewayTransportFacet; label: string }> = [
  { value: 'stdio', label: 'stdio' },
  { value: 'http', label: 'HTTP' },
]

const EXPOSURE_OPTIONS: Array<{ value: ToolsExposureFilter; label: string }> = [
  { value: 'all', label: 'All' },
  { value: 'exposed', label: 'Exposed only' },
  { value: 'hidden', label: 'Hidden only' },
]

function FilterCheckbox({ checked, label, onChange }: FilterCheckboxProps) {
  return (
    <label className="flex items-center gap-2 text-[13px] leading-[1.2] font-medium text-aurora-text-primary">
      <Checkbox checked={checked} onCheckedChange={onChange} className="border-aurora-border-strong bg-aurora-control-surface" />
      <span>{label}</span>
    </label>
  )
}

export function GatewayFilters({
  mode,
  search,
  gatewayFilters,
  toolFilters,
  gatewayOptions,
  mobileSheetOpen,
  onMobileSheetOpenChange,
  onSearchChange,
  onGatewayFilterToggle,
  onToolFilterToggle,
  onExposureChange,
  onClearFilters,
}: GatewayFiltersProps) {
  const hasGatewayFilters =
    search.length > 0 ||
    gatewayFilters.status.length > 0 ||
    gatewayFilters.source.length > 0 ||
    gatewayFilters.transport.length > 0

  const hasToolFilters =
    search.length > 0 ||
    toolFilters.gatewayIds.length > 0 ||
    toolFilters.exposure !== 'all' ||
    toolFilters.source.length > 0 ||
    toolFilters.transport.length > 0

  const hasFilters = mode === 'tools' ? hasToolFilters : hasGatewayFilters

  const filterGroups = (
    <div className="space-y-4">
      <div className="space-y-1.5">
        <p className={AURORA_MUTED_LABEL}>Search</p>
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
          <Input
            placeholder={mode === 'tools' ? 'Search tools, descriptions, or gateways' : 'Search gateways, commands, or endpoints'}
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            className={cn(
              AURORA_CONTROL_SURFACE,
              'h-11 border pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted',
            )}
          />
        </div>
      </div>

      {mode === 'gateways' ? (
        <>
          <FilterGroup label="Status">
            {GATEWAY_STATUS_OPTIONS.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={gatewayFilters.status.includes(option.value)}
                label={option.label}
                onChange={() => onGatewayFilterToggle('status', option.value)}
              />
            ))}
          </FilterGroup>

          <FilterGroup label="Source">
            {SOURCE_OPTIONS.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={gatewayFilters.source.includes(option.value)}
                label={option.label}
                onChange={() => onGatewayFilterToggle('source', option.value)}
              />
            ))}
          </FilterGroup>

          <FilterGroup label="Transport">
            {TRANSPORT_OPTIONS.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={gatewayFilters.transport.includes(option.value)}
                label={option.label}
                onChange={() => onGatewayFilterToggle('transport', option.value)}
              />
            ))}
          </FilterGroup>
        </>
      ) : (
        <>
          <FilterGroup label="Gateway">
            {gatewayOptions.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={toolFilters.gatewayIds.includes(option.value)}
                label={option.label}
                onChange={() => onToolFilterToggle('gatewayIds', option.value)}
              />
            ))}
          </FilterGroup>

          <FilterGroup label="Exposure">
            <div className="flex flex-wrap gap-2">
              {EXPOSURE_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => onExposureChange(option.value)}
                  className={cn(
                    'rounded-full border px-3 py-1.5 text-[13px] leading-[1.2] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/34',
                    pillTone(toolFilters.exposure === option.value),
                  )}
                  aria-pressed={toolFilters.exposure === option.value}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </FilterGroup>

          <FilterGroup label="Source">
            {SOURCE_OPTIONS.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={toolFilters.source.includes(option.value)}
                label={option.label}
                onChange={() => onToolFilterToggle('source', option.value)}
              />
            ))}
          </FilterGroup>

          <FilterGroup label="Transport">
            {TRANSPORT_OPTIONS.map((option) => (
              <FilterCheckbox
                key={option.value}
                checked={toolFilters.transport.includes(option.value)}
                label={option.label}
                onChange={() => onToolFilterToggle('transport', option.value)}
              />
            ))}
          </FilterGroup>
        </>
      )}
    </div>
  )

  return (
    <>
      <div className={cn(AURORA_MEDIUM_PANEL, 'hidden space-y-4 p-4 lg:block')}>
        <div className="flex items-center justify-between gap-3">
          <p className={AURORA_MUTED_LABEL}>Filters</p>
          {hasFilters ? (
            <Button
              variant="outline"
              size="sm"
              onClick={onClearFilters}
              className={cn(gatewayActionTone(), 'h-9 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg')}
            >
              <X className="mr-1 size-4" />
              Clear filters
            </Button>
          ) : null}
        </div>
        {filterGroups}
      </div>

      <Sheet open={mobileSheetOpen} onOpenChange={onMobileSheetOpenChange}>
        <SheetContent
          side="bottom"
          className="max-h-[80vh] overflow-y-auto border-aurora-border-strong bg-aurora-panel-strong text-aurora-text-primary aurora-scrollbar"
        >
          <SheetHeader className="px-0 pt-0">
            <SheetTitle className="font-display text-[19px] leading-[1.12] font-bold tracking-[-0.02em] text-aurora-text-primary">
              Filters
            </SheetTitle>
            <SheetDescription className="text-sm text-aurora-text-muted">
              {mode === 'tools' ? 'Refine the aggregated tools inventory.' : 'Refine the current gateway lens.'}
            </SheetDescription>
          </SheetHeader>

          <div className="space-y-4 px-0 pb-2">
            {hasFilters ? (
              <Button
                variant="outline"
                size="sm"
                onClick={onClearFilters}
                className={cn(gatewayActionTone(), 'h-9 px-3 text-aurora-text-primary hover:bg-aurora-hover-bg')}
              >
                <X className="mr-1 size-4" />
                Clear filters
              </Button>
            ) : null}
            {filterGroups}
          </div>
        </SheetContent>
      </Sheet>
    </>
  )
}

function FilterGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2.5">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <div className="space-y-2">{children}</div>
    </div>
  )
}
