'use client'

import * as React from 'react'
import Link from 'next/link'
import { startTransition, useDeferredValue } from 'react'
import { Copy, Database, HardDrive, Waypoints } from 'lucide-react'

import { AppHeader } from '@/components/app-header'
import { LogFilters } from '@/components/logs/log-filters'
import { LogStreamStatusCard } from '@/components/logs/log-stream-status'
import { LogTimeline } from '@/components/logs/log-timeline'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { fetchLogs, fetchLogStats } from '@/lib/api/logs-client'
import { connectLogStream } from '@/lib/api/logs-stream'
import {
  buildLogSearchQuery,
  matchesVisibleLogEvent,
  mergeTimelineEvents,
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

const timestampFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
})

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`
  }
  return `${bytes} B`
}

function retainedWindow(stats: LogStoreStats | null): string {
  if (!stats?.oldest_retained_ts || !stats.newest_retained_ts) {
    return 'Awaiting retained events'
  }
  return `${timestampFormatter.format(new Date(stats.oldest_retained_ts))} -> ${timestampFormatter.format(new Date(stats.newest_retained_ts))}`
}

function scrollViewportToBottom(viewport: HTMLDivElement | null) {
  viewport?.scrollTo({
    top: viewport.scrollHeight,
    behavior: 'smooth',
  })
}

function formatRetentionWindowDays(days: number): string {
  return `${days} day${days === 1 ? '' : 's'}`
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
        const uncoveredBufferedEvents = bufferedEventsRef.current.filter((event) => {
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

  const streamStatus: LogStreamStatus = {
    connected,
    paused: effectivePaused,
    atLiveEdge,
    buffered: bufferedEvents.length,
    lastEventTs,
    error: streamError,
  }

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

      <div className="flex-1 space-y-6 bg-linear-to-br from-slate-50 via-white to-sky-50/60 p-6">
        <section className="rounded-[28px] border border-slate-200/80 bg-linear-to-br from-slate-950 via-slate-900 to-sky-900 px-6 py-7 text-slate-50 shadow-xl shadow-slate-950/10">
          <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div className="max-w-3xl space-y-3">
              <p className="text-xs font-medium uppercase tracking-[0.26em] text-cyan-200/80">Local-master runtime logs</p>
              <h1 className="text-3xl font-semibold tracking-tight">One console for persisted history and live flow</h1>
              <p className="max-w-2xl text-sm text-slate-300 sm:text-base">
                This view is backed by the shared `dispatch::logs` contract. Historical queries come from the embedded store, while live arrivals use the in-process SSE stream without inventing a second schema.
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-3">
              <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Stored events</p>
                <p className="mt-2 text-2xl font-semibold tabular-nums">
                  {stats ? stats.total_event_count : 'Loading…'}
                </p>
              </div>
              <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Retention</p>
                <p className="mt-2 text-sm font-medium">
                  {stats
                    ? `${formatRetentionWindowDays(stats.retention.max_age_days)} / ${formatBytes(stats.retention.max_bytes)}`
                    : 'Loading…'}
                </p>
              </div>
              <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3">
                <p className="text-xs uppercase tracking-[0.18em] text-slate-400">Dropped</p>
                <p className="mt-2 text-2xl font-semibold tabular-nums">
                  {stats ? stats.dropped_event_count : 'Loading…'}
                </p>
              </div>
            </div>
          </div>
        </section>

        <div className="grid gap-6 xl:grid-cols-[minmax(0,360px)_minmax(0,1fr)]">
          <div className="space-y-6">
            <LogFilters
              filters={filters}
              windowPreset={windowPreset}
              onFiltersChange={setFilters}
              onWindowPresetChange={setWindowPreset}
              onRefresh={() => {
                setRefreshToken((value) => value + 1)
              }}
              isRefreshing={isRefreshing}
            />

            <LogStreamStatusCard
              status={streamStatus}
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

            <Card className="border-slate-200/80 bg-white/90 shadow-sm shadow-slate-900/5">
              <CardHeader>
                <CardTitle className="text-base">Contract preview</CardTitle>
                <CardDescription>
                  The same filter state maps cleanly to HTTP, MCP, and CLI without changing semantics.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-3 sm:grid-cols-2">
                  <div className="rounded-xl border border-slate-200 bg-slate-50 p-3">
                    <p className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                      <Database className="size-3.5" />
                      Store window
                    </p>
                    <p className="mt-2 text-sm font-medium text-slate-900">{retainedWindow(stats)}</p>
                  </div>
                  <div className="rounded-xl border border-slate-200 bg-slate-50 p-3">
                    <p className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                      <HardDrive className="size-3.5" />
                      On disk
                    </p>
                    <p className="mt-2 text-sm font-medium text-slate-900">{stats ? formatBytes(stats.on_disk_bytes) : 'Loading…'}</p>
                  </div>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-slate-950 p-3">
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <p className="flex items-center gap-2 text-xs font-medium uppercase tracking-[0.18em] text-slate-400">
                      <Waypoints className="size-3.5" />
                      POST /v1/logs
                    </p>
                    <Badge variant="outline" className="border-slate-700 text-slate-200">
                      logs.search
                    </Badge>
                  </div>
                  <pre className="overflow-x-auto text-xs text-slate-100">
                    {queryPreviewForAfterTs(filters, afterTs)}
                  </pre>
                </div>
              </CardContent>
            </Card>
          </div>

          <LogTimeline
            events={events}
            isLoading={isLoading}
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
          />
        </div>
      </div>
    </>
  )
}
