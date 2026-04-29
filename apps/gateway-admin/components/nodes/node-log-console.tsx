'use client'

import { useDeferredValue, startTransition, useState, useRef, useEffect, useMemo, useLayoutEffect } from 'react'

import { RefreshCw } from 'lucide-react'

import { AURORA_DISPLAY_2, AURORA_MEDIUM_PANEL } from '@/components/aurora/tokens'
import { fetchLogStats, fetchLogs } from '@/lib/api/logs-client'
import { connectLogStream } from '@/lib/api/logs-stream'
import { Button } from '@/components/ui/button'
import { LogEventInspector } from '@/components/logs/log-event-inspector'
import { LogToolbar } from '@/components/logs/log-toolbar'
import { LogTimeline } from '@/components/logs/log-timeline'
import {
  buildLogSearchQuery,
  matchesVisibleLogEvent,
  mergeTimelineEvents,
  resolveExpandedEventId,
  resolveSelectedEvent,
  toggleExpandedEventId,
} from '@/lib/dashboard/logs-console-state'
import type { LogEvent, LogFilterState, LogStreamStatus, LogStoreStats } from '@/lib/types/logs'
import type { LogSearchQuery } from '@/lib/types/logs'

const DEFAULT_FILTERS: LogFilterState = {
  text: '',
  subsystems: [],
  levels: [],
  limit: 120,
}

const WINDOW_TO_MS: Record<string, number | null> = {
  '15m': 15 * 60 * 1000,
  '1h': 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  all: null,
}

const NODE_LOG_SOURCE_KINDS = ['syslog', 'application', 'journald', 'peer']
const BUFFER_LIMIT = 500

function buildNodeQuery(filters: LogFilterState, nodeId: string, afterTs: number | null): LogSearchQuery {
  const query = buildLogSearchQuery(filters, { afterTs })
  query.source_node_ids = [nodeId]
  query.source_kinds = [...NODE_LOG_SOURCE_KINDS]
  return query
}

function scrollViewportToBottom(viewport: HTMLDivElement | null) {
  viewport?.scrollTo({
    top: viewport.scrollHeight,
    behavior: 'smooth',
  })
}

interface NodeLogConsoleProps {
  nodeId: string
  windowClassName?: string
}

export function NodeLogConsole({ nodeId, windowClassName }: NodeLogConsoleProps) {
  const [filters, setFilters] = useState<LogFilterState>(() => ({
    ...DEFAULT_FILTERS,
  }))
  const [windowPreset, setWindowPreset] = useState('1h')
  const [events, setEvents] = useState<LogEvent[]>([])
  const [bufferedEvents, setBufferedEvents] = useState<LogEvent[]>([])
  const [stats, setStats] = useState<LogStoreStats | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [connected, setConnected] = useState(false)
  const [streamError, setStreamError] = useState<string | null>(null)
  const [manualPause, setManualPause] = useState(false)
  const [atLiveEdge, setAtLiveEdge] = useState(true)
  const [lastEventTs, setLastEventTs] = useState<number | null>(null)
  const [refreshNonce, setRefreshNonce] = useState(0)
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null)
  const [expandedEventId, setExpandedEventId] = useState<string | null>(null)

  const deferredFilters = useDeferredValue(filters)
  const viewportRef = useRef<HTMLDivElement | null>(null)
  const filtersRef = useRef(filters)
  const effectivePaused = manualPause || !atLiveEdge
  const effectivePausedRef = useRef(effectivePaused)
  const bufferedEventsRef = useRef(bufferedEvents)
  const maxEntriesRef = useRef(filters.limit)
  const afterTsRef = useRef<number | null>(null)
  const afterTs = useMemo(() => {
    const windowMs = WINDOW_TO_MS[windowPreset]
    return windowMs == null ? null : Date.now() - windowMs
  }, [windowPreset])

  useLayoutEffect(() => {
    filtersRef.current = filters
    effectivePausedRef.current = effectivePaused
    bufferedEventsRef.current = bufferedEvents
    maxEntriesRef.current = filters.limit
    afterTsRef.current = afterTs
  }, [afterTs, bufferedEvents, filters, effectivePaused])

  useEffect(() => {
    const controller = new AbortController()
    let disposed = false

    setIsLoading(true)
    setIsRefreshing(true)

    void Promise.all([
      fetchLogs(buildNodeQuery(deferredFilters, nodeId, afterTs), { signal: controller.signal }),
      fetchLogStats({ signal: controller.signal }),
    ])
      .then(([result, nextStats]) => {
        if (disposed) {
          return
        }

        const fetchedEvents = mergeTimelineEvents([], result.events, deferredFilters.limit)
        const fetchedEventIds = new Set(fetchedEvents.map((event) => event.event_id))
        const bufferedSnapshot = bufferedEventsRef.current
        bufferedEventsRef.current = []
        const uncoveredBufferedEvents = bufferedSnapshot.filter((event) => {
          if (fetchedEventIds.has(event.event_id)) {
            return false
          }
          return matchesVisibleLogEvent(event, deferredFilters, { afterTs })
        })

        startTransition(() => {
          setEvents(
            effectivePausedRef.current
              ? fetchedEvents
              : mergeTimelineEvents(
                  fetchedEvents,
                  uncoveredBufferedEvents,
                  deferredFilters.limit,
                ),
          )
          setBufferedEvents(effectivePausedRef.current ? uncoveredBufferedEvents : [])
          setStats(nextStats)
          setStreamError(null)
        })
      })
      .catch((error: unknown) => {
        if (disposed || (error instanceof DOMException && error.name === 'AbortError')) {
          return
        }
        startTransition(() => {
          setStreamError(error instanceof Error ? error.message : 'failed to load logs')
        })
      })
      .finally(() => {
        if (!disposed) {
          setIsLoading(false)
          setIsRefreshing(false)
        }
      })

    return () => {
      disposed = true
      controller.abort()
    }
  }, [afterTs, deferredFilters, nodeId, refreshNonce])

  useEffect(() => {
    const disconnect = connectLogStream({
      onOpen: () => {
        startTransition(() => {
          setConnected(true)
          setStreamError(null)
        })
      },
      onError: (message) => {
        startTransition(() => {
          setConnected(false)
          setStreamError(message)
        })
      },
      onLag: (skipped) => {
        startTransition(() => {
          setStreamError(`live stream lagged and dropped ${skipped} event${skipped === 1 ? '' : 's'}`)
        })
      },
      onEvent: (event) => {
        if (!matchesVisibleLogEvent(event, filtersRef.current, { afterTs: afterTsRef.current })) {
          return
        }

        startTransition(() => {
          setLastEventTs(event.ts)
          if (effectivePausedRef.current) {
            setBufferedEvents((current) => mergeTimelineEvents(current, [event], BUFFER_LIMIT))
            return
          }

          setEvents((current) => mergeTimelineEvents(current, [event], maxEntriesRef.current))
        })

        if (!effectivePausedRef.current) {
          requestAnimationFrame(() => {
            scrollViewportToBottom(viewportRef.current)
          })
        }
      },
    })

    return () => {
      disconnect()
    }
  }, [nodeId])

  useEffect(() => {
    if (effectivePaused || bufferedEvents.length === 0) {
      return
    }

    startTransition(() => {
      setEvents((current) => mergeTimelineEvents(current, bufferedEvents, filters.limit))
      setBufferedEvents([])
    })

    requestAnimationFrame(() => {
      scrollViewportToBottom(viewportRef.current)
    })
  }, [bufferedEvents, effectivePaused, filters.limit])

  useEffect(() => {
    const nextSelectedEvent = resolveSelectedEvent(events, selectedEventId)
    setSelectedEventId(nextSelectedEvent?.event_id ?? null)
    setExpandedEventId((current) => resolveExpandedEventId(events, current))
  }, [events, selectedEventId])

  const streamStatus: LogStreamStatus = {
    connected,
    paused: effectivePaused,
    atLiveEdge,
    buffered: bufferedEvents.length,
    lastEventTs,
    error: streamError,
  }
  const selectedEvent = resolveSelectedEvent(events, selectedEventId)

  return (
    <div className={AURORA_MEDIUM_PANEL}>
      <div className="px-4 py-4 md:px-6">
        <div className="mb-4 flex flex-wrap items-center justify-between gap-2 border-b border-aurora-border-strong pb-4">
          <div>
            <p className={`${AURORA_DISPLAY_2} text-aurora-text-primary`}>Node Logs</p>
            <p className="text-sm text-aurora-text-muted">Per-node syslog + deployed-binary logs.</p>
          </div>
          <Button
            variant="outline"
            size="sm"
            className="gap-2"
            onClick={() => {
              setRefreshNonce((value) => value + 1)
            }}
            disabled={isRefreshing}
          >
            <RefreshCw className="size-4" />
            <span>{isRefreshing ? 'Refreshing…' : 'Refresh logs'}</span>
          </Button>
        </div>

        <LogToolbar
          filters={filters}
          windowPreset={windowPreset}
          stats={stats}
          streamStatus={streamStatus}
          isRefreshing={isRefreshing}
          onFiltersChange={setFilters}
          onWindowPresetChange={setWindowPreset}
          onRefresh={() => {
            setRefreshNonce((value) => value + 1)
          }}
          onTogglePause={() => {
            setManualPause((value) => !value)
          }}
          onJumpToNewest={() => {
            setManualPause(false)
            setAtLiveEdge(true)
            requestAnimationFrame(() => {
              scrollViewportToBottom(viewportRef.current)
            })
          }}
        />

        <div className={`mt-5 grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px] ${windowClassName ?? ''}`}>
          <LogTimeline
            events={events}
            isLoading={isLoading}
            selectedEventId={selectedEvent?.event_id ?? null}
            expandedEventId={expandedEventId}
            showLiveEdgeBadge={!effectivePaused && atLiveEdge}
            viewportRef={viewportRef}
            onViewportScroll={() => {
              const viewport = viewportRef.current
              if (!viewport) return

              const distanceToBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight
              setAtLiveEdge(distanceToBottom < 24)
            }}
            onSelectEvent={setSelectedEventId}
            onToggleExpanded={(eventId) => {
              setExpandedEventId((current) => toggleExpandedEventId(current, eventId))
            }}
          />

          <LogEventInspector event={selectedEvent} />
        </div>
      </div>
    </div>
  )
}
