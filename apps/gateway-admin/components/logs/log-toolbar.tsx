'use client'

import { PauseCircle, PlayCircle, RefreshCw, Search } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { LOG_LEVELS, LOG_SUBSYSTEMS, type LogFilterState, type LogLevel, type LogStoreStats, type LogStreamStatus, type LogSubsystem } from '@/lib/types/logs'
import {
  AURORA_CONTROL_SURFACE,
  AURORA_DISPLAY_NUMBER,
  AURORA_MEDIUM_PANEL,
  AURORA_MUTED_LABEL,
  controlTone,
  pillTone,
} from '@/components/logs/log-theme'
import { cn } from '@/lib/utils'

interface LogToolbarProps {
  filters: LogFilterState
  windowPreset: string
  stats: LogStoreStats | null
  streamStatus: LogStreamStatus
  isRefreshing: boolean
  onFiltersChange: (next: LogFilterState) => void
  onWindowPresetChange: (value: string) => void
  onRefresh: () => void
  onTogglePause: () => void
  onJumpToNewest: () => void
}

const WINDOW_OPTIONS = [
  { value: '15m', label: 'Last 15m' },
  { value: '1h', label: 'Last hour' },
  { value: '24h', label: 'Last 24h' },
  { value: 'all', label: 'All retained' },
] as const

const LIMIT_OPTIONS = ['50', '100', '200', '500'] as const

function toggleValue<T extends string>(current: T[], value: T): T[] {
  return current.includes(value)
    ? current.filter((item) => item !== value)
    : [...current, value]
}

export function LogToolbar({
  filters,
  windowPreset,
  stats,
  streamStatus,
  isRefreshing,
  onFiltersChange,
  onWindowPresetChange,
  onRefresh,
  onTogglePause,
  onJumpToNewest,
}: LogToolbarProps) {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          label="Stored events"
          value={stats ? String(stats.total_event_count) : '...'}
          detail={stats ? `${stats.retention.max_age_days} days retained` : 'Loading stats'}
        />
        <MetricCard
          label="Visible"
          value={String(filters.limit)}
          detail="Current line window"
        />
        <MetricCard
          label="Dropped events"
          value={stats ? String(stats.dropped_event_count) : '...'}
          detail={
            stats
              ? (stats.dropped_event_count > 0
                ? `${stats.dropped_event_count} dropped from live stream`
                : 'No drops observed')
              : 'Loading stats'
          }
        />
        <MetricCard
          label="Stream"
          value={streamStatus.connected ? 'Live' : 'Offline'}
          detail={
            streamStatus.connected
              ? (streamStatus.paused ? `${streamStatus.buffered} buffered` : 'Receiving SSE')
              : (streamStatus.error ?? 'Disconnected')
          }
          accent={streamStatus.connected ? 'text-aurora-accent-primary' : 'text-aurora-error'}
        />
      </div>

      <div className={`${AURORA_MEDIUM_PANEL} p-4 sm:p-5`}>
        <div className="flex flex-col gap-4">
          <div className="flex flex-wrap items-center gap-3">
            <label className="relative min-w-[240px] flex-1">
              <Search aria-hidden="true" className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-aurora-text-muted" />
              <Input
                aria-label="Search log events"
                className={cn(AURORA_CONTROL_SURFACE, 'pl-9 text-aurora-text-primary placeholder:text-aurora-text-muted')}
                placeholder="Search message, action, request id"
                value={filters.text}
                onChange={(event) => {
                  onFiltersChange({
                    ...filters,
                    text: event.target.value,
                  })
                }}
              />
            </label>

            <Select value={windowPreset} onValueChange={onWindowPresetChange}>
              <SelectTrigger className={cn(AURORA_CONTROL_SURFACE, 'w-[150px] text-aurora-text-primary')}>
                <SelectValue placeholder="Window" />
              </SelectTrigger>
              <SelectContent>
                {WINDOW_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select
              value={String(filters.limit)}
              onValueChange={(value) => {
                onFiltersChange({
                  ...filters,
                  limit: Number(value),
                })
              }}
            >
              <SelectTrigger className={cn(AURORA_CONTROL_SURFACE, 'w-[130px] text-aurora-text-primary')}>
                <SelectValue placeholder="Limit" />
              </SelectTrigger>
              <SelectContent>
                {LIMIT_OPTIONS.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value} lines
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button variant="outline" className={cn(controlTone(), 'rounded-[0.95rem] hover:bg-[#17364b] hover:text-aurora-text-primary')} onClick={onTogglePause}>
              {streamStatus.paused ? <PlayCircle className="size-4" /> : <PauseCircle className="size-4" />}
              {streamStatus.paused ? 'Resume' : 'Pause'}
            </Button>

            <Button
              variant="outline"
              className={cn(controlTone(), 'rounded-[0.95rem] hover:bg-[#17364b] hover:text-aurora-text-primary')}
              onClick={onJumpToNewest}
              disabled={streamStatus.atLiveEdge && streamStatus.buffered === 0}
            >
              Jump to newest
            </Button>

            <Button variant="outline" className={cn(controlTone('accent'), 'rounded-[0.95rem] hover:bg-[#17364b] hover:text-aurora-text-primary')} onClick={onRefresh} disabled={isRefreshing}>
              <RefreshCw className="size-4" />
              {isRefreshing ? 'Refreshing…' : 'Refresh'}
            </Button>
          </div>

          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <FilterGroup<LogLevel>
              label="Levels"
              values={LOG_LEVELS}
              selected={filters.levels}
              onToggle={(level) => {
                onFiltersChange({
                  ...filters,
                  levels: toggleValue(filters.levels, level),
                })
              }}
            />

            <FilterGroup<LogSubsystem>
              label="Subsystems"
              values={LOG_SUBSYSTEMS}
              selected={filters.subsystems}
              onToggle={(subsystem) => {
                onFiltersChange({
                  ...filters,
                  subsystems: toggleValue(filters.subsystems, subsystem),
                })
              }}
            />
          </div>

          {streamStatus.error ? (
            <div role="alert" className="rounded-[0.95rem] border border-aurora-error/28 bg-[rgba(76,42,52,0.18)] px-3 py-2 text-sm text-[#efd6dd] shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
              {streamStatus.error}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}

function MetricCard({
  label,
  value,
  detail,
  accent,
}: {
  label: string
  value: string
  detail: string
  accent?: string
}) {
  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'flex h-full flex-col justify-between p-4')}>
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <p className={cn(AURORA_DISPLAY_NUMBER, 'mt-2 text-[2rem] leading-none font-semibold', accent ?? '')}>{value}</p>
      <p className="mt-2 text-sm text-aurora-text-muted">{detail}</p>
    </div>
  )
}

function FilterGroup<T extends string>({
  label,
  values,
  selected,
  onToggle,
}: {
  label: string
  values: readonly T[]
  selected: T[]
  onToggle: (value: T) => void
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <p className={AURORA_MUTED_LABEL}>{label}</p>
      </div>
      <div className="flex flex-wrap gap-2">
        {values.map((value) => {
          const active = selected.includes(value)
          return (
            <button
              type="button"
              key={value}
              aria-pressed={active}
              className={`inline-flex min-h-[38px] items-center gap-2 rounded-full border px-3.5 py-1.5 text-sm font-medium tracking-[-0.01em] transition-[border-color,background,color,box-shadow,transform] hover:-translate-y-px ${pillTone(active)}`}
              onClick={() => onToggle(value)}
            >
              <span
                className={`relative h-[17px] w-[17px] shrink-0 rounded-full border ${
                  active
                    ? 'border-aurora-accent-strong/60 bg-[linear-gradient(180deg,rgba(41,182,246,0.16),rgba(12,41,56,0.94))]'
                    : 'border-aurora-border-strong bg-[linear-gradient(180deg,rgba(9,20,28,0.94),rgba(7,15,22,0.98))]'
                }`}
              >
                {active ? (
                  <span className="absolute inset-[4px] rounded-full bg-[linear-gradient(180deg,#a5e6ff,#29b6f6)] shadow-[0_0_8px_rgba(41,182,246,0.24)]" />
                ) : null}
              </span>
              <span className="capitalize">{value}</span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
