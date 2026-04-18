import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { GatewayTable } from './gateway-table'
import type { Gateway } from '@/lib/types/gateway'

const gateway: Gateway = {
  id: 'gw_1',
  name: 'Plex Control Plane',
  transport: 'http',
  source: 'custom',
  configured: true,
  enabled: true,
  config: {
    url: 'https://plex.example.com/mcp',
  },
  status: {
    healthy: true,
    connected: true,
    last_error: 'Reload required to apply policy changes.',
    discovered_tool_count: 18,
    exposed_tool_count: 14,
    discovered_resource_count: 6,
    exposed_resource_count: 4,
    discovered_prompt_count: 3,
    exposed_prompt_count: 2,
  },
  discovery: {
    tools: [],
    resources: [],
    prompts: [],
  },
  warnings: [
    {
      code: 'policy_drift',
      message: 'Tool exposure differs from the last successful sync.',
      timestamp: '2026-04-17T13:00:00Z',
    },
  ],
  created_at: '2026-04-16T12:00:00Z',
  updated_at: '2026-04-17T12:00:00Z',
}

test('gateway table uses aurora lifted surfaces and muted operational pills', () => {
  const markup = renderToStaticMarkup(
    React.createElement(GatewayTable, {
      gateways: [gateway],
      onEdit: () => {},
      onTest: () => {},
      onReload: () => {},
      onDelete: () => {},
    }),
  )

  assert.match(markup, /bg-aurora-panel-strong/)
  assert.match(markup, /bg-aurora-control-surface/)
  assert.match(markup, /text-aurora-text-primary/)
  assert.match(markup, /text-aurora-warn/)
})
