import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildLogSearchQuery,
  matchesLogFilters,
  matchesVisibleLogEvent,
  mergeTimelineEvents,
} from './logs-console-state.ts'
import type { LogEvent, LogFilterState } from '../types/logs.ts'

function eventWith(overrides: Partial<LogEvent> = {}): LogEvent {
  return {
    event_id: overrides.event_id ?? 'evt-1',
    ts: overrides.ts ?? 1_713_225_600_000,
    level: overrides.level ?? 'info',
    subsystem: overrides.subsystem ?? 'gateway',
    surface: overrides.surface ?? 'api',
    action: overrides.action ?? 'logs.search',
    message: overrides.message ?? 'gateway event',
    request_id: overrides.request_id ?? null,
    session_id: overrides.session_id ?? null,
    correlation_id: overrides.correlation_id ?? null,
    trace_id: overrides.trace_id ?? null,
    span_id: overrides.span_id ?? null,
    instance: overrides.instance ?? 'default',
    auth_flow: overrides.auth_flow ?? null,
    outcome_kind: overrides.outcome_kind ?? 'ok',
    fields_json: overrides.fields_json ?? {},
    source_kind: overrides.source_kind ?? null,
    source_node_id: overrides.source_node_id ?? null,
    source_device_id: overrides.source_device_id ?? null,
    ingest_path: overrides.ingest_path ?? null,
    upstream_event_id: overrides.upstream_event_id ?? null,
  }
}

test('buildLogSearchQuery trims text and omits empty filters', () => {
  const filters: LogFilterState = {
    text: '  gateway timeout  ',
    subsystems: ['gateway'],
    levels: [],
    limit: 50,
  }

  assert.deepEqual(buildLogSearchQuery(filters), {
    text: 'gateway timeout',
    subsystems: ['gateway'],
    limit: 50,
  })
})

test('matchesLogFilters applies text, subsystem, and level filters', () => {
  const filters: LogFilterState = {
    text: 'timeout',
    subsystems: ['gateway'],
    levels: ['warn'],
    limit: 100,
  }

  assert.equal(
    matchesLogFilters(
      eventWith({ message: 'gateway timeout detected', level: 'warn' }),
      filters,
    ),
    true,
  )
  assert.equal(
    matchesLogFilters(
      eventWith({ message: 'gateway ok', level: 'warn' }),
      filters,
    ),
    false,
  )
  assert.equal(
    matchesLogFilters(
      eventWith({ message: 'gateway timeout detected', level: 'info' }),
      filters,
    ),
    false,
  )
})

test('matchesLogFilters can match request identifiers and structured fields', () => {
  const filters: LogFilterState = {
    text: 'req-123',
    subsystems: [],
    levels: [],
    limit: 100,
  }

  assert.equal(
    matchesLogFilters(
      eventWith({ request_id: 'req-123' }),
      filters,
    ),
    true,
  )
  assert.equal(
    matchesLogFilters(
      eventWith({ fields_json: { correlation_id: 'req-123' } }),
      filters,
    ),
    true,
  )
})

test('matchesVisibleLogEvent applies the same filter contract plus the active time window', () => {
  const filters: LogFilterState = {
    text: 'timeout',
    subsystems: ['gateway'],
    levels: ['warn'],
    limit: 100,
  }

  assert.equal(
    matchesVisibleLogEvent(
      eventWith({ ts: 200, message: 'gateway timeout detected', level: 'warn' }),
      filters,
      { afterTs: 100 },
    ),
    true,
  )
  assert.equal(
    matchesVisibleLogEvent(
      eventWith({ ts: 50, message: 'gateway timeout detected', level: 'warn' }),
      filters,
      { afterTs: 100 },
    ),
    false,
  )
})

test('mergeTimelineEvents deduplicates, sorts chronologically, and keeps the newest entries', () => {
  const merged = mergeTimelineEvents(
    [
      eventWith({ event_id: 'evt-1', ts: 10, message: 'oldest' }),
      eventWith({ event_id: 'evt-2', ts: 20, message: 'middle' }),
    ],
    [
      eventWith({ event_id: 'evt-2', ts: 20, message: 'middle replacement' }),
      eventWith({ event_id: 'evt-3', ts: 30, message: 'newest' }),
    ],
    2,
  )

  assert.deepEqual(
    merged.map((event) => [event.event_id, event.message]),
    [
      ['evt-2', 'middle replacement'],
      ['evt-3', 'newest'],
    ],
  )
})

test('mergeTimelineEvents breaks equal timestamps by event identifier', () => {
  const merged = mergeTimelineEvents(
    [eventWith({ event_id: 'evt-b', ts: 10, message: 'b' })],
    [eventWith({ event_id: 'evt-a', ts: 10, message: 'a' })],
    10,
  )

  assert.deepEqual(
    merged.map((event) => event.event_id),
    ['evt-a', 'evt-b'],
  )
})
