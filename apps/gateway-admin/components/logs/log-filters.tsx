'use client'

import { Filter, RotateCcw, Search } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { LOG_LEVELS, LOG_SUBSYSTEMS, type LogFilterState, type LogLevel, type LogSubsystem } from '@/lib/types/logs'

interface LogFiltersProps {
  filters: LogFilterState
  windowPreset: string
  onFiltersChange: (next: LogFilterState) => void
  onWindowPresetChange: (value: string) => void
  onRefresh: () => void
  isRefreshing: boolean
}

const WINDOW_OPTIONS = [
  { value: '15m', label: 'Last 15 minutes' },
  { value: '1h', label: 'Last hour' },
  { value: '24h', label: 'Last day' },
  { value: 'all', label: 'Full retention window' },
] as const

const LIMIT_OPTIONS = ['50', '100', '200', '500'] as const

function toggleValue<T extends string>(current: T[], value: T): T[] {
  return current.includes(value)
    ? current.filter((item) => item !== value)
    : [...current, value]
}

export function LogFilters({
  filters,
  windowPreset,
  onFiltersChange,
  onWindowPresetChange,
  onRefresh,
  isRefreshing,
}: LogFiltersProps) {
  return (
    <Card className="border-slate-200/80 bg-white/90 shadow-sm shadow-slate-900/5">
      <CardHeader className="gap-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2 text-base">
              <Filter className="size-4 text-sky-600" />
              Query
            </CardTitle>
            <CardDescription>
              Refine the persisted local-master history and the live stream with one shared filter set.
            </CardDescription>
          </div>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isRefreshing}>
            <RotateCcw className="size-4" />
            {isRefreshing ? 'Refreshing…' : 'Refresh'}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid gap-3 lg:grid-cols-[minmax(0,1.6fr)_220px_140px]">
          <label className="space-y-2">
            <span className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">Search</span>
            <div className="relative">
              <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                className="pl-9"
                placeholder="message, action, request identifiers"
                value={filters.text}
                onChange={(event) => {
                  onFiltersChange({
                    ...filters,
                    text: event.target.value,
                  })
                }}
              />
            </div>
          </label>

          <label className="space-y-2">
            <span className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">Time window</span>
            <Select value={windowPreset} onValueChange={onWindowPresetChange}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Choose a range" />
              </SelectTrigger>
              <SelectContent>
                {WINDOW_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>

          <label className="space-y-2">
            <span className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">Limit</span>
            <Select
              value={String(filters.limit)}
              onValueChange={(value) => {
                onFiltersChange({
                  ...filters,
                  limit: Number(value),
                })
              }}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Rows" />
              </SelectTrigger>
              <SelectContent>
                {LIMIT_OPTIONS.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value} events
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </label>
        </div>

        <div className="grid gap-5 lg:grid-cols-2">
          <div className="space-y-3">
            <div className="flex items-center justify-between gap-2">
              <span className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">Levels</span>
              {filters.levels.length > 0 ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    onFiltersChange({
                      ...filters,
                      levels: [],
                    })
                  }}
                >
                  Clear
                </Button>
              ) : null}
            </div>
            <div className="flex flex-wrap gap-2">
              {LOG_LEVELS.map((level) => (
                <label
                  key={level}
                  className="flex items-center gap-2 rounded-full border border-slate-200 bg-slate-50 px-3 py-1.5 text-sm"
                >
                  <Checkbox
                    checked={filters.levels.includes(level)}
                    onCheckedChange={() => {
                      onFiltersChange({
                        ...filters,
                        levels: toggleValue(filters.levels, level as LogLevel),
                      })
                    }}
                  />
                  <span className="capitalize">{level}</span>
                </label>
              ))}
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center justify-between gap-2">
              <span className="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">Subsystems</span>
              {filters.subsystems.length > 0 ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    onFiltersChange({
                      ...filters,
                      subsystems: [],
                    })
                  }}
                >
                  Clear
                </Button>
              ) : null}
            </div>
            <div className="flex flex-wrap gap-2">
              {LOG_SUBSYSTEMS.map((subsystem) => (
                <label
                  key={subsystem}
                  className="flex items-center gap-2 rounded-full border border-slate-200 bg-slate-50 px-3 py-1.5 text-sm"
                >
                  <Checkbox
                    checked={filters.subsystems.includes(subsystem)}
                    onCheckedChange={() => {
                      onFiltersChange({
                        ...filters,
                        subsystems: toggleValue(filters.subsystems, subsystem as LogSubsystem),
                      })
                    }}
                  />
                  <span>{subsystem}</span>
                </label>
              ))}
            </div>
          </div>
        </div>

        <div className="flex flex-wrap gap-2">
          {filters.levels.map((level) => (
            <Badge key={level} variant="outline" className="capitalize">
              level:{level}
            </Badge>
          ))}
          {filters.subsystems.map((subsystem) => (
            <Badge key={subsystem} variant="outline">
              subsystem:{subsystem}
            </Badge>
          ))}
          {filters.text.trim() ? (
            <Badge variant="outline">text:{filters.text.trim()}</Badge>
          ) : null}
        </div>
      </CardContent>
    </Card>
  )
}
