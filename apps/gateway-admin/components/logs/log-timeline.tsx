'use client'

import { ArrowDown, ChevronDown, ChevronRight } from 'lucide-react'

import {
  AURORA_DISPLAY_TITLE,
  AURORA_LEVEL_TEXT,
  AURORA_MUTED_LABEL,
  AURORA_STRONG_PANEL,
  AURORA_MESSAGE_SURFACE,
  AURORA_TAIL_ROW,
} from '@/components/logs/log-theme'
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
          <p className={`text-lg font-semibold text-aurora-text-primary ${AURORA_DISPLAY_TITLE}`}>Runtime stream</p>
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

      <div className={`${AURORA_TAIL_ROW} border-b border-aurora-border-strong bg-[rgba(7,17,26,0.48)] px-5 py-3 ${AURORA_MUTED_LABEL}`}>
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

            return (
              <div
                key={event.event_id}
                className={cn(
                  'border-t border-aurora-border-strong/70 first:border-t-0',
                  selected && 'bg-[rgba(41,182,246,0.12)] shadow-[inset_2px_0_0_#29b6f6]',
                )}
              >
                <div
                  className={`${AURORA_TAIL_ROW} px-5 py-3 transition-colors hover:bg-[rgba(7,17,26,0.52)]`}
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
                  <div className="font-mono text-xs leading-5 text-aurora-text-muted">
                    {timestampFormatter.format(new Date(event.ts))}
                  </div>
                  <div className={`font-mono text-sm font-semibold ${AURORA_LEVEL_TEXT[event.level]}`}>
                    {event.level.toUpperCase()}
                  </div>
                  <div className="font-mono text-sm text-aurora-text-muted">{event.subsystem}</div>
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
                        <ChevronDown className="mt-0.5 size-4 shrink-0 text-aurora-text-muted" />
                      ) : (
                        <ChevronRight className="mt-0.5 size-4 shrink-0 text-aurora-text-muted" />
                      )}
                      <span
                        className={cn(
                          'min-w-0 font-mono text-sm text-aurora-text-primary',
                          expanded ? 'whitespace-pre-wrap break-words leading-6' : 'truncate',
                        )}
                      >
                        {event.message}
                      </span>
                    </button>
                  </div>
                </div>

                {expanded ? (
                  <div className={`${AURORA_TAIL_ROW} px-5 pb-4`}>
                    <div />
                    <div />
                    <div />
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
