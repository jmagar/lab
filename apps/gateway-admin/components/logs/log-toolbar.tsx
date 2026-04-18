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

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`
  }
  return `${bytes} B`
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
          label="Dropped"
          value={stats ? String(stats.dropped_event_count) : '...'}
          detail={stats ? formatBytes(stats.on_disk_bytes) : 'Loading stats'}
        />
        <MetricCard
          label="Stream"
          value={streamStatus.connected ? 'Live' : 'Offline'}
          detail={
            streamStatus.connected
              ? (streamStatus.paused ? `${streamStatus.buffered} buffered` : 'Receiving SSE')
              : (streamStatus.error ?? 'Disconnected')
          }
          accent={streamStatus.connected ? 'text-emerald-300' : 'text-rose-300'}
        />
      </div>

      <div className="rounded-[1.35rem] border border-[#2a556d] bg-[linear-gradient(180deg,rgba(20,44,60,0.18),transparent_30%),linear-gradient(180deg,rgba(17,38,53,0.98),rgba(15,33,46,0.98))] p-4 shadow-[0_12px_24px_rgba(0,0,0,0.18)]">
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center gap-3">
            <label className="relative min-w-[240px] flex-1">
              <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-[#90a9b9]" />
              <Input
                className="border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] pl-9 text-[#e5f2f8] placeholder:text-[#90a9b9] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)]"
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
              <SelectTrigger className="w-[150px] border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#e5f2f8] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)]">
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
              <SelectTrigger className="w-[130px] border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#e5f2f8] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)]">
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

            <Button variant="outline" className="border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#d6eaf3] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)] hover:bg-[#17364b] hover:text-[#edf9ff]" onClick={onTogglePause}>
              {streamStatus.paused ? <PlayCircle className="size-4" /> : <PauseCircle className="size-4" />}
              {streamStatus.paused ? 'Resume' : 'Pause'}
            </Button>

            <Button
              variant="outline"
              className="border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-[#d6eaf3] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)] hover:bg-[#17364b] hover:text-[#edf9ff]"
              onClick={onJumpToNewest}
              disabled={streamStatus.atLiveEdge && streamStatus.buffered === 0}
            >
              Jump to newest
            </Button>

            <Button variant="outline" className="border-[#29b6f6]/35 bg-[linear-gradient(180deg,rgba(13,32,45,0.98),rgba(10,24,34,0.98))] text-[#67cbfa] shadow-[0_0_0_1px_rgba(41,182,246,0.18),0_0_16px_rgba(41,182,246,0.08)] hover:bg-[#17364b] hover:text-[#edf9ff]" onClick={onRefresh} disabled={isRefreshing}>
              <RefreshCw className="size-4" />
              {isRefreshing ? 'Refreshing…' : 'Refresh'}
            </Button>
          </div>

          <div className="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
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
            <div className="rounded-lg border border-[#c78490]/35 bg-[#4b2630]/22 px-3 py-2 text-sm text-[#f2c8d0]">
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
    <div className="rounded-[1.35rem] border border-[#2a556d] bg-[linear-gradient(180deg,rgba(20,44,60,0.18),transparent_30%),linear-gradient(180deg,rgba(17,38,53,0.98),rgba(15,33,46,0.98))] p-4 shadow-[0_12px_24px_rgba(0,0,0,0.18)]">
      <p className="text-[11px] font-medium uppercase tracking-[0.18em] text-[#90a9b9]">{label}</p>
      <p className={`font-display mt-2 text-3xl font-semibold tracking-[-0.04em] tabular-nums ${accent ?? ''}`}>{value}</p>
      <p className="mt-1 text-sm text-[#90a9b9]">{detail}</p>
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
      <p className="text-[11px] font-medium uppercase tracking-[0.18em] text-[#90a9b9]">{label}</p>
      <div className="flex flex-wrap gap-2">
        {values.map((value) => {
          const active = selected.includes(value)
          return (
            <button
              type="button"
              key={value}
              aria-pressed={active}
              className={`inline-flex min-h-[38px] items-center gap-2 rounded-full border px-3 py-1.5 text-sm font-medium transition-[border-color,background,color,box-shadow,transform] hover:-translate-y-px ${
                active
                  ? 'border-[#29b6f6]/35 bg-[linear-gradient(180deg,rgba(13,32,45,0.98),rgba(10,24,34,0.98))] text-[#edf9ff] shadow-[0_0_0_1px_rgba(41,182,246,0.18),0_0_16px_rgba(41,182,246,0.08)]'
                  : 'border-[#2b5770] bg-[linear-gradient(180deg,rgba(10,23,32,0.94),rgba(9,20,28,0.98))] text-[#b9ced9] shadow-[inset_0_1px_0_rgba(255,255,255,0.025)]'
              }`}
              onClick={() => onToggle(value)}
            >
              <span
                className={`relative h-[18px] w-[18px] shrink-0 rounded-full border ${
                  active
                    ? 'border-[#67cbfa]/70 bg-[linear-gradient(180deg,rgba(41,182,246,0.18),rgba(12,41,56,0.94))]'
                    : 'border-[#2b5b74] bg-[linear-gradient(180deg,rgba(9,20,28,0.96),rgba(7,15,22,0.98))]'
                }`}
              >
                {active ? (
                  <span className="absolute inset-[4px] rounded-full bg-[linear-gradient(180deg,#8bddff,#29b6f6)] shadow-[0_0_10px_rgba(41,182,246,0.34)]" />
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
