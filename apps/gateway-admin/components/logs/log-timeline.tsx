'use client'

import { ArrowDown, ChevronDown, ChevronRight } from 'lucide-react'

import { cn } from '@/lib/utils'
import type { LogEvent } from '@/lib/types/logs'

interface LogTimelineProps {
  events: LogEvent[]
  isLoading: boolean
  selectedEventId: string | null
  expandedEventId: string | null
  showLiveEdgeBadge: boolean
  viewportRef: React.RefObject<HTMLDivElement | null>
  onViewportScroll: () => void
  onSelectEvent: (eventId: string) => void
  onToggleExpanded: (eventId: string) => void
}

const levelStyles = {
  trace: 'text-[#90a9b9]',
  debug: 'text-[#67cbfa]',
  info: 'text-[#29b6f6]',
  warn: 'text-[#f7b955]',
  error: 'text-[#fb7185]',
} as const

const timestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'short',
  timeStyle: 'medium',
})

export function LogTimeline({
  events,
  isLoading,
  selectedEventId,
  expandedEventId,
  showLiveEdgeBadge,
  viewportRef,
  onViewportScroll,
  onSelectEvent,
  onToggleExpanded,
}: LogTimelineProps) {
  return (
    <div className="overflow-hidden rounded-[1.4rem] border border-[#2d5c76] bg-[linear-gradient(180deg,rgba(24,52,71,0.24),transparent_30%),linear-gradient(180deg,rgba(21,46,64,0.98),rgba(17,38,54,0.98))] shadow-[0_20px_38px_rgba(0,0,0,0.26)]">
      <div className="flex items-center justify-between border-b border-[#23485c] px-5 py-4">
        <div>
          <p className="font-display text-lg font-semibold tracking-[-0.02em] text-[#e5f2f8]">Runtime stream</p>
          <p className="text-sm text-[#90a9b9]">Structured tail view. One line per event.</p>
        </div>
        <div className="flex items-center gap-3 text-xs uppercase tracking-[0.18em] text-[#90a9b9]">
          <span>{events.length} visible</span>
          {showLiveEdgeBadge ? (
            <span className="inline-flex items-center gap-1 rounded-full border border-[#29b6f6]/35 bg-[linear-gradient(180deg,rgba(13,32,45,0.98),rgba(10,24,34,0.98))] px-2 py-1 text-[11px] tracking-[0.14em] text-[#67cbfa] shadow-[0_0_0_1px_rgba(41,182,246,0.18),0_0_16px_rgba(41,182,246,0.08)]">
              <ArrowDown className="size-3.5" />
              Live edge
            </span>
          ) : null}
        </div>
      </div>

      <div className="grid grid-cols-[170px_84px_130px_minmax(0,1fr)] gap-3 border-b border-[#23485c] bg-[rgba(7,17,26,0.48)] px-5 py-3 text-[11px] font-medium uppercase tracking-[0.18em] text-[#90a9b9]">
        <div>Timestamp</div>
        <div>Level</div>
        <div>Subsystem</div>
        <div>Message</div>
      </div>

      <div ref={viewportRef} className="max-h-[760px] overflow-y-auto" onScroll={onViewportScroll}>
        {isLoading ? (
          <div className="space-y-1 p-3">
            {Array.from({ length: 10 }, (_, index) => (
              <div key={index} className="h-11 animate-pulse rounded-lg bg-[rgba(7,17,26,0.52)]" />
            ))}
          </div>
        ) : events.length === 0 ? (
          <div className="px-5 py-12 text-center text-sm text-[#90a9b9]">
            No matching events. Relax the filters or wait for new local-master traffic.
          </div>
        ) : (
          events.map((event) => {
            const selected = event.event_id === selectedEventId
            const expanded = event.event_id === expandedEventId

            return (
              <div
                key={event.event_id}
                className={cn(
                  'border-t border-[#23485c]/70 first:border-t-0',
                  selected && 'bg-[#29b6f6]/12 shadow-[inset_2px_0_0_#29b6f6]',
                )}
              >
                <div
                  className="grid grid-cols-[170px_84px_130px_minmax(0,1fr)] gap-3 px-5 py-3 transition-colors hover:bg-[rgba(7,17,26,0.52)]"
                  onClick={() => onSelectEvent(event.event_id)}
                  onKeyDown={(keyEvent) => {
                    if (keyEvent.key === 'Enter' || keyEvent.key === ' ') {
                      keyEvent.preventDefault()
                      onSelectEvent(event.event_id)
                    }
                  }}
                  role="button"
                  tabIndex={0}
                >
                  <div className="font-mono text-xs leading-5 text-[#90a9b9]">
                    {timestampFormatter.format(new Date(event.ts))}
                  </div>
                  <div className={`font-mono text-sm font-semibold ${levelStyles[event.level]}`}>
                    {event.level.toUpperCase()}
                  </div>
                  <div className="font-mono text-sm text-[#90a9b9]">{event.subsystem}</div>
                  <div className="min-w-0">
                    <button
                      type="button"
                      className="flex w-full items-start gap-2 text-left"
                      onClick={(clickEvent) => {
                        clickEvent.stopPropagation()
                        onSelectEvent(event.event_id)
                        onToggleExpanded(event.event_id)
                      }}
                    >
                      {expanded ? (
                        <ChevronDown className="mt-0.5 size-4 shrink-0 text-[#90a9b9]" />
                      ) : (
                        <ChevronRight className="mt-0.5 size-4 shrink-0 text-[#90a9b9]" />
                      )}
                      <span
                        className={cn(
                          'min-w-0 font-mono text-sm text-[#e5f2f8]',
                          expanded ? 'whitespace-pre-wrap break-words leading-6' : 'truncate',
                        )}
                      >
                        {event.message}
                      </span>
                    </button>
                  </div>
                </div>

                {expanded ? (
                  <div className="grid grid-cols-[170px_84px_130px_minmax(0,1fr)] gap-3 px-5 pb-4">
                    <div />
                    <div />
                    <div />
                    <div className="rounded-[1rem] border border-[#2b5770] bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] px-4 py-3 font-mono text-sm leading-6 whitespace-pre-wrap break-words text-[#e5f2f8] shadow-[0_8px_16px_rgba(0,0,0,0.16),inset_0_1px_0_rgba(255,255,255,0.035)]">
                      {event.message}
                    </div>
                  </div>
                ) : null}
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
