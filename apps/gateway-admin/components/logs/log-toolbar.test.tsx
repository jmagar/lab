import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { LogToolbar } from './log-toolbar'
import type { LogFilterState, LogStoreStats, LogStreamStatus } from '@/lib/types/logs'

const filters: LogFilterState = {
  text: 'health',
  levels: ['info', 'warn'],
  subsystems: ['gateway', 'api'],
  limit: 200,
}

const stats: LogStoreStats = {
  on_disk_bytes: 268435456,
  oldest_retained_ts: 1713378000000,
  newest_retained_ts: 1713381600000,
  total_event_count: 189,
  dropped_event_count: 2,
  retention: {
    max_age_days: 7,
    max_bytes: 536870912,
  },
}

const streamStatus: LogStreamStatus = {
  connected: true,
  paused: false,
  atLiveEdge: true,
  buffered: 0,
  lastEventTs: 1713381600000,
  error: null,
}

test('log toolbar uses display typography for metrics and pressed pill buttons for filters', () => {
  const markup = renderToStaticMarkup(
    React.createElement(LogToolbar, {
      filters,
      windowPreset: '1h',
      stats,
      streamStatus,
      isRefreshing: false,
      onFiltersChange: () => {},
      onWindowPresetChange: () => {},
      onRefresh: () => {},
      onTogglePause: () => {},
      onJumpToNewest: () => {},
    }),
  )

  assert.match(markup, /font-display/)
  assert.match(markup, /aria-pressed="true"/)
  assert.match(markup, /aria-pressed="false"/)
  assert.match(markup, /aria-label="Search log events"/)
  // Pills: match the exact label text rendered inside the pill buttons
  assert.match(markup, /\binfo\b/i)
  assert.match(markup, /\bwarn\b/i)
  assert.match(markup, /\bgateway\b/i)
  assert.match(markup, /\btrace\b/i)
  assert.match(markup, /2 dropped from live stream/)
})
