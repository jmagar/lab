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

  return query
}

export function matchesLogFilters(event: LogEvent, filters: LogFilterState): boolean {
  const text = filters.text.trim().toLowerCase()

  if (text && !`${event.message} ${event.action ?? ''}`.toLowerCase().includes(text)) {
    return false
  }
  if (filters.subsystems.length > 0 && !filters.subsystems.includes(event.subsystem)) {
    return false
  }
  if (filters.levels.length > 0 && !filters.levels.includes(event.level)) {
    return false
  }

  return true
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
    .slice(-Math.max(maxEntries, 1))
}
