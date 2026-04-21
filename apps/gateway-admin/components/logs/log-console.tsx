'use client'

import * as React from 'react'
import Link from 'next/link'
import { startTransition, useDeferredValue } from 'react'
import { Copy } from 'lucide-react'

import { AppHeader } from '@/components/app-header'
import { LogEventInspector } from '@/components/logs/log-event-inspector'
import { LogToolbar } from '@/components/logs/log-toolbar'
import { LogTimeline } from '@/components/logs/log-timeline'
import { Button } from '@/components/ui/button'
import { fetchLogs, fetchLogStats } from '@/lib/api/logs-client'
import { connectLogStream } from '@/lib/api/logs-stream'
import {
  AURORA_PAGE_FRAME,
  AURORA_PAGE_SHELL,
} from '@/components/aurora/tokens'
import {
  buildLogSearchQuery,
  resolveExpandedEventId,
  resolveSelectedEvent,
  matchesVisibleLogEvent,
  mergeTimelineEvents,
  toggleExpandedEventId,
} from '@/lib/dashboard/logs-console-state'
import type {
  LogEvent,
  LogFilterState,
  LogStoreStats,
  LogStreamStatus,
} from '@/lib/types/logs'

const DEFAULT_FILTERS: LogFilterState = {
  text: '',
  subsystems: [],
  levels: [],
  limit: 200,
}

const WINDOW_TO_MS: Record<string, number | null> = {
  '15m': 15 * 60 * 1000,
  '1h': 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  all: null,
}

const BUFFER_LIMIT = 500

function scrollViewportToBottom(viewport: HTMLDivElement | null) {
  viewport?.scrollTo({
    top: viewport.scrollHeight,
    behavior: 'smooth',
  })
}

function queryPreviewForAfterTs(filters: LogFilterState, afterTs: number | null): string {
  return JSON.stringify(
    {
      action: 'logs.search',
      params: {
        query: buildLogSearchQuery(filters, { afterTs }),
      },
    },
    null,
    2,
  )
}

export function LogConsole() {
  const [filters, setFilters] = React.useState<LogFilterState>(DEFAULT_FILTERS)
  const [windowPreset, setWindowPreset] = React.useState('1h')
  const [events, setEvents] = React.useState<LogEvent[]>([])
  const [bufferedEvents, setBufferedEvents] = React.useState<LogEvent[]>([])
  const [stats, setStats] = React.useState<LogStoreStats | null>(null)
  const [isLoading, setIsLoading] = React.useState(true)
  const [isRefreshing, setIsRefreshing] = React.useState(false)
  const [connected, setConnected] = React.useState(false)
  const [streamError, setStreamError] = React.useState<string | null>(null)
  const [copyStatus, setCopyStatus] = React.useState<string | null>(null)
  const [manualPause, setManualPause] = React.useState(false)
  const [atLiveEdge, setAtLiveEdge] = React.useState(true)
  const [lastEventTs, setLastEventTs] = React.useState<number | null>(null)
  const [refreshToken, setRefreshToken] = React.useState(0)
  const [selectedEventId, setSelectedEventId] = React.useState<string | null>(null)
  const [expandedEventId, setExpandedEventId] = React.useState<string | null>(null)

  const deferredFilters = useDeferredValue(filters)
  const viewportRef = React.useRef<HTMLDivElement | null>(null)
  const filtersRef = React.useRef(filters)
  const effectivePaused = manualPause || !atLiveEdge
  const effectivePausedRef = React.useRef(effectivePaused)
  const bufferedEventsRef = React.useRef(bufferedEvents)
  const maxEntriesRef = React.useRef(filters.limit)
  const afterTsRef = React.useRef<number | null>(null)
  const afterTs = React.useMemo(() => {
    const windowMs = WINDOW_TO_MS[windowPreset]
    return windowMs == null ? null : Date.now() - windowMs
  }, [windowPreset, refreshToken])

  React.useLayoutEffect(() => {
    filtersRef.current = filters
    effectivePausedRef.current = effectivePaused
    bufferedEventsRef.current = bufferedEvents
    maxEntriesRef.current = filters.limit
    afterTsRef.current = afterTs
  }, [afterTs, bufferedEvents, filters, effectivePaused])

  React.useEffect(() => {
    if (!copyStatus) {
      return
    }

    const timeoutId = window.setTimeout(() => {
      setCopyStatus(null)
    }, 3000)

    return () => {
      window.clearTimeout(timeoutId)
    }
  }, [copyStatus])

  React.useEffect(() => {
    const controller = new AbortController()
    let disposed = false

    setIsLoading(true)
    setIsRefreshing(true)

    void Promise.all([
      fetchLogs(buildLogSearchQuery(deferredFilters, { afterTs }), {
        signal: controller.signal,
      }),
      fetchLogStats({ signal: controller.signal }),
    ])
      .then(([result, nextStats]) => {
        if (disposed) {
          return
        }

        const fetchedEvents = mergeTimelineEvents([], result.events, deferredFilters.limit)
        const fetchedEventIds = new Set(fetchedEvents.map((event) => event.event_id))
        // Drain the ref atomically: capture any SSE events that arrived during the await,
        // then reset it so they are not double-applied by subsequent state commits.
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
  }, [afterTs, deferredFilters])

  React.useEffect(() => {
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
            setBufferedEvents((current) =>
              mergeTimelineEvents(current, [event], BUFFER_LIMIT),
            )
            return
          }

          setEvents((current) =>
            mergeTimelineEvents(current, [event], maxEntriesRef.current),
          )
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
  }, [])

  React.useEffect(() => {
    if (effectivePaused || bufferedEvents.length === 0) {
      return
    }

    startTransition(() => {
      setEvents((current) =>
        mergeTimelineEvents(current, bufferedEvents, filters.limit),
      )
      setBufferedEvents([])
    })

    requestAnimationFrame(() => {
      scrollViewportToBottom(viewportRef.current)
    })
  }, [bufferedEvents, effectivePaused, filters.limit])

  React.useEffect(() => {
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
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Logs' },
        ]}
        actions={(
          <>
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                const preview = queryPreviewForAfterTs(filters, afterTs)
                navigator.clipboard
                  .writeText(preview)
                  .then(() => {
                    setCopyStatus('Query copied')
                  })
                  .catch((error: unknown) => {
                    console.warn('failed to copy logs query preview', error)
                    setCopyStatus('Copy failed')
                  })
              }}
            >
              <Copy className="size-4" />
              Copy query
            </Button>
            {copyStatus ? <span className="text-xs text-muted-foreground">{copyStatus}</span> : null}
            <Button variant="outline" size="sm" asChild>
              <Link href="/activity">Back to activity</Link>
            </Button>
          </>
        )}
      />

      <div className={`relative min-h-[calc(100vh-3.5rem)] w-full overflow-hidden bg-aurora-page-bg text-aurora-text-primary ${AURORA_PAGE_SHELL}`}>
        <div className={AURORA_PAGE_FRAME}>
        <LogToolbar
          filters={filters}
          windowPreset={windowPreset}
          stats={stats}
          streamStatus={streamStatus}
          isRefreshing={isRefreshing}
          onFiltersChange={setFilters}
          onWindowPresetChange={setWindowPreset}
          onRefresh={() => {
            setRefreshToken((value) => value + 1)
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

          <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_384px]">
          <LogTimeline
            events={events}
            isLoading={isLoading}
            selectedEventId={selectedEvent?.event_id ?? null}
            expandedEventId={expandedEventId}
            showLiveEdgeBadge={!effectivePaused && atLiveEdge}
            viewportRef={viewportRef}
            onViewportScroll={() => {
              const viewport = viewportRef.current
              if (!viewport) {
                return
              }

              const distanceToBottom =
                viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight
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
    </>
  )
}
