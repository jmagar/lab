import type { LogEvent, LogFilterState, LogSearchQuery } from '../types/logs.ts'

export function buildLogSearchQuery(
  filters: LogFilterState,
  options?: { afterTs?: number | null },
): LogSearchQuery {
  const text = filters.text.trim()
  const query: LogSearchQuery = {
    limit: filters.limit > 0 ? filters.limit : undefined,
  }

  if (text) {
    query.text = text
  }
  if (filters.subsystems.length > 0) {
    query.subsystems = filters.subsystems
  }
  if (filters.levels.length > 0) {
    query.levels = filters.levels
  }
  if (options?.afterTs != null) {
    query.after_ts = options.afterTs
  }
  if (filters.source_node_ids?.length) {
    query.source_node_ids = filters.source_node_ids
  }

  return query
}

export function matchesLogFilters(event: LogEvent, filters: LogFilterState): boolean {
  const text = filters.text.trim().toLowerCase()

  if (text && !buildSearchableEventText(event).includes(text)) {
    return false
  }
  if (filters.subsystems.length > 0 && !filters.subsystems.includes(event.subsystem)) {
    return false
  }
  if (filters.levels.length > 0 && !filters.levels.includes(event.level)) {
    return false
  }
  if (
    filters.source_node_ids?.length
    && (event.source_node_id == null || !filters.source_node_ids.includes(event.source_node_id))
  ) {
    return false
  }

  return true
}

export function matchesVisibleLogEvent(
  event: LogEvent,
  filters: LogFilterState,
  options?: { afterTs?: number | null },
): boolean {
  if (options?.afterTs != null && event.ts < options.afterTs) {
    return false
  }

  return matchesLogFilters(event, filters)
}

function buildSearchableEventText(event: LogEvent): string {
  const structuredFields = JSON.stringify(event.fields_json ?? {})
  return [
    event.message,
    event.action ?? '',
    event.request_id ?? '',
    event.session_id ?? '',
    event.correlation_id ?? '',
    event.trace_id ?? '',
    event.span_id ?? '',
    event.instance ?? '',
    event.auth_flow ?? '',
    event.outcome_kind ?? '',
    event.source_kind ?? '',
    event.source_node_id ?? '',
    event.source_device_id ?? '',
    event.ingest_path ?? '',
    event.upstream_event_id ?? '',
    structuredFields,
  ]
    .join('\u0000')
    .toLowerCase()
}

export function mergeTimelineEvents(
  current: LogEvent[],
  incoming: LogEvent[],
  maxEntries: number,
): LogEvent[] {
  const merged = new Map<string, LogEvent>()

  for (const event of current) {
    merged.set(event.event_id, event)
  }
  for (const event of incoming) {
    merged.set(event.event_id, event)
  }

  return Array.from(merged.values())
    .sort((left, right) => {
      if (left.ts === right.ts) {
        return left.event_id.localeCompare(right.event_id)
      }
      return left.ts - right.ts
    })
    // Keep the newest-N window stable even if a caller passes 0 or a negative limit.
    .slice(-Math.max(maxEntries, 1))
}

export function getDefaultSelectedEventId(events: LogEvent[]): string | null {
  return events.at(-1)?.event_id ?? null
}

export function resolveSelectedEvent(
  events: LogEvent[],
  selectedEventId: string | null,
): LogEvent | null {
  if (events.length === 0) {
    return null
  }

  return events.find((event) => event.event_id === selectedEventId) ?? events.at(-1) ?? null
}

export function toggleExpandedEventId(
  expandedEventId: string | null,
  targetEventId: string,
): string | null {
  return expandedEventId === targetEventId ? null : targetEventId
}

export function resolveExpandedEventId(
  events: LogEvent[],
  expandedEventId: string | null,
): string | null {
  if (!expandedEventId) {
    return null
  }

  return events.some((event) => event.event_id === expandedEventId)
    ? expandedEventId
    : null
}
