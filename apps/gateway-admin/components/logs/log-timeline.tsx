'use client'

import { ArrowDown, ChevronDown, ChevronRight } from 'lucide-react'

import {
  AURORA_DISPLAY_2,
  AURORA_LEVEL_TEXT,
  AURORA_MESSAGE_SURFACE,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
  AURORA_TAIL_ROW,
} from '@/components/aurora/tokens'
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
    <div className={AURORA_STRONG_PANEL}>
      <div className="flex items-center justify-between border-b border-aurora-border-strong px-5 py-4">
        <div>
          <p className={`${AURORA_DISPLAY_2} text-aurora-text-primary`}>Runtime stream</p>
          <p className="text-sm text-aurora-text-muted">Structured tail view. One line per event.</p>
        </div>
        <div className="flex items-center gap-3 text-xs uppercase tracking-[0.18em] text-aurora-text-muted">
          <span>{events.length} visible</span>
          {showLiveEdgeBadge ? (
            <span className="inline-flex items-center gap-1 rounded-full border border-aurora-accent-primary/35 bg-[linear-gradient(180deg,rgba(13,32,45,0.98),rgba(10,24,34,0.98))] px-2 py-1 text-[11px] tracking-[0.14em] text-aurora-accent-strong shadow-[var(--aurora-active-glow)]">
              <ArrowDown className="size-3.5" />
              Live edge
            </span>
          ) : null}
        </div>
      </div>

      <div className={`${AURORA_TAIL_ROW} hidden border-b border-aurora-border-strong bg-[rgba(7,17,26,0.48)] px-5 py-3 ${AURORA_MUTED_LABEL} sm:grid`}>
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
          <div className="px-5 py-12 text-center text-sm text-aurora-text-muted">
            No matching events. Relax the filters or wait for new local-master traffic.
          </div>
        ) : (
          events.map((event) => {
            const selected = event.event_id === selectedEventId
            const expanded = event.event_id === expandedEventId
            const detailsId = `log-event-details-${event.event_id}`

            return (
              <div
                key={event.event_id}
                className={cn(
                  'border-t border-aurora-border-strong/70 first:border-t-0',
                  selected && 'bg-aurora-accent-primary/12 shadow-[inset_2px_0_0_var(--aurora-accent-primary)]',
                )}
              >
                <div className="grid grid-cols-[minmax(0,1fr)_2.5rem] items-stretch gap-2 px-4 py-3 transition-colors hover:bg-[rgba(7,17,26,0.52)] sm:px-5">
                  <button
                    type="button"
                    className={`${AURORA_TAIL_ROW} min-w-0 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/45 focus-visible:ring-inset`}
                    onClick={() => onSelectEvent(event.event_id)}
                  >
                    <div className="font-mono text-xs leading-5 text-aurora-text-muted">
                      {timestampFormatter.format(new Date(event.ts))}
                    </div>
                    <div className="flex min-w-0 flex-wrap items-center gap-2 sm:contents">
                      <span className={`font-mono text-sm font-semibold ${AURORA_LEVEL_TEXT[event.level]}`}>
                        {event.level.toUpperCase()}
                      </span>
                      <span className="min-w-0 truncate font-mono text-sm text-aurora-text-muted">{event.subsystem}</span>
                    </div>
                    <div className="min-w-0">
                      <span className="block min-w-0 truncate font-mono text-sm text-aurora-text-primary">
                        {event.message}
                      </span>
                    </div>
                  </button>

                  <button
                    type="button"
                    className="inline-flex h-9 w-9 items-center justify-center self-start rounded-full border border-aurora-border-strong bg-[linear-gradient(180deg,rgba(18,40,56,0.96),rgba(14,31,44,0.98))] text-aurora-text-muted shadow-[0_8px_16px_rgba(0,0,0,0.16),var(--aurora-highlight-medium)] transition-colors hover:bg-[rgba(7,17,26,0.64)] hover:text-aurora-text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-aurora-accent-primary/45"
                    aria-controls={detailsId}
                    aria-expanded={expanded}
                    aria-label={expanded ? 'Collapse log details' : 'Expand log details'}
                    onClick={() => {
                      onSelectEvent(event.event_id)
                      onToggleExpanded(event.event_id)
                    }}
                  >
                    {expanded ? (
                      <ChevronDown aria-hidden="true" className="size-4" />
                    ) : (
                      <ChevronRight aria-hidden="true" className="size-4" />
                    )}
                  </button>
                </div>

                {expanded ? (
                  <div
                    id={detailsId}
                    aria-label="Expanded log message"
                    className={`${AURORA_TAIL_ROW} px-4 pb-4 sm:px-5`}
                    role="region"
                  >
                    <div className="hidden sm:block" />
                    <div className="hidden sm:block" />
                    <div className="hidden sm:block" />
                    <div className={`${AURORA_MESSAGE_SURFACE} px-4 py-3 font-mono text-sm leading-6 whitespace-pre-wrap break-words text-aurora-text-primary`}>
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
