'use client'

import { ArrowDown, ArrowRight, CircleDot, Hash, Radio } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { LogEvent } from '@/lib/types/logs'

interface LogTimelineProps {
  events: LogEvent[]
  isLoading: boolean
  showLiveEdgeBadge: boolean
  viewportRef: React.RefObject<HTMLDivElement | null>
  onViewportScroll: () => void
}

const levelStyles = {
  trace: 'border-slate-300 bg-slate-100 text-slate-700',
  debug: 'border-cyan-300 bg-cyan-50 text-cyan-700',
  info: 'border-sky-300 bg-sky-50 text-sky-700',
  warn: 'border-amber-300 bg-amber-50 text-amber-700',
  error: 'border-rose-300 bg-rose-50 text-rose-700',
} as const

export function LogTimeline({
  events,
  isLoading,
  showLiveEdgeBadge,
  viewportRef,
  onViewportScroll,
}: LogTimelineProps) {
  const timestampFormatter = new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'medium',
  })

  return (
    <div className="overflow-hidden rounded-2xl border border-slate-200/80 bg-white shadow-sm shadow-slate-900/5">
      <div className="flex items-center justify-between border-b border-slate-200/80 px-5 py-4">
        <div>
          <p className="text-sm font-semibold">Unified timeline</p>
          <p className="text-sm text-muted-foreground">
            Historical store results and live arrivals land in the same chronological feed.
          </p>
        </div>
        <Badge variant="outline" className="tabular-nums">
          {events.length} visible
        </Badge>
      </div>

      <div ref={viewportRef} className="max-h-[720px] overflow-y-auto px-5 py-4" onScroll={onViewportScroll}>
        {isLoading ? (
          <div className="space-y-3">
            {Array.from({ length: 6 }, (_, index) => (
              <div key={index} className="h-28 animate-pulse rounded-2xl border border-slate-200 bg-slate-50" />
            ))}
          </div>
        ) : events.length === 0 ? (
          <div className="flex min-h-[320px] flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-slate-300 bg-slate-50/80 px-6 text-center">
            <Radio className="size-8 text-slate-400" />
            <div className="space-y-1">
              <p className="font-medium text-slate-700">No matching events</p>
              <p className="text-sm text-muted-foreground">
                Relax the filters or wait for new local-master runtime traffic.
              </p>
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {events.map((event, index) => (
              <article
                key={event.event_id}
                className="rounded-2xl border border-slate-200/80 bg-linear-to-br from-white to-slate-50/90 p-4"
              >
                <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div className="space-y-3">
                    <div className="flex flex-wrap items-center gap-2">
                      <Badge variant="outline" className={cn('capitalize', levelStyles[event.level])}>
                        {event.level}
                      </Badge>
                      <Badge variant="outline">{event.subsystem}</Badge>
                      {event.action ? <Badge variant="outline">{event.action}</Badge> : null}
                      {showLiveEdgeBadge && index === events.length - 1 ? (
                        <Badge variant="outline" className="border-cyan-300 bg-cyan-50 text-cyan-700">
                          <ArrowDown className="size-3.5" />
                          live edge
                        </Badge>
                      ) : null}
                    </div>
                    <p className="text-base font-medium leading-6 text-slate-900">{event.message}</p>
                    <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
                      <span className="inline-flex items-center gap-1">
                        <CircleDot className="size-3.5" />
                        {timestampFormatter.format(new Date(event.ts))}
                      </span>
                      <span className="inline-flex items-center gap-1">
                        <ArrowRight className="size-3.5" />
                        {event.surface}
                      </span>
                      {event.request_id ? (
                        <span className="inline-flex items-center gap-1">
                          <Hash className="size-3.5" />
                          {event.request_id}
                        </span>
                      ) : null}
                    </div>
                  </div>
                  {Object.keys(event.fields_json ?? {}).length > 0 ? (
                    <pre className="max-w-full overflow-x-auto rounded-xl border border-slate-200 bg-slate-950 px-3 py-2 text-xs text-slate-100 lg:max-w-[420px]">
                      {JSON.stringify(event.fields_json, null, 2)}
                    </pre>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
