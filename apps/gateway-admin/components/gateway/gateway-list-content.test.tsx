import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { GatewayListView } from './gateway-list-content'
import { SidebarProvider } from '@/components/ui/sidebar'
import type { Gateway } from '@/lib/types/gateway'

const gatewayFixtures: Gateway[] = [
  {
    id: 'gw_lab',
    name: 'Lab Core',
    transport: 'stdio',
    source: 'lab_service',
    configured: true,
    enabled: true,
    config: { command: 'lab service mcp --stdio --services chrome-dev-tools' },
    status: {
      healthy: true,
      connected: true,
      discovered_tool_count: 5,
      exposed_tool_count: 5,
      discovered_resource_count: 0,
      exposed_resource_count: 0,
      discovered_prompt_count: 0,
      exposed_prompt_count: 0,
    },
    discovery: {
      tools: [{ name: 'click', description: 'Click an element', exposed: true, matched_by: 'all' }],
      resources: [],
      prompts: [],
    },
    warnings: [],
    created_at: '2026-04-17T10:00:00Z',
    updated_at: '2026-04-18T10:00:00Z',
  },
  {
    id: 'gw_http',
    name: 'Arcane',
    transport: 'http',
    source: 'custom',
    configured: true,
    enabled: true,
    config: { url: 'https://arcane.example.com/mcp' },
    status: {
      healthy: false,
      connected: false,
      discovered_tool_count: 11,
      exposed_tool_count: 7,
      discovered_resource_count: 2,
      exposed_resource_count: 1,
      discovered_prompt_count: 1,
      exposed_prompt_count: 0,
    },
    discovery: {
      tools: [{ name: 'container.restart', description: 'Restart a container', exposed: false, matched_by: null }],
      resources: [],
      prompts: [],
    },
    warnings: [
      { code: 'unreachable', message: 'Server is not responding.', timestamp: '2026-04-18T11:00:00Z' },
    ],
    created_at: '2026-04-17T10:00:00Z',
    updated_at: '2026-04-18T10:00:00Z',
  },
]

test('gateway list view renders quick-lens cards and primary actions', () => {
  const markup = renderToStaticMarkup(
    <SidebarProvider>
      <GatewayListView
        summary={{ configured: 2, healthy: 1, disconnected: 1, tools: 2 }}
        showToolsView={false}
        gatewayFilters={{ primaryLens: 'configured', search: '', status: [], source: [], transport: [] }}
        toolFilters={{ search: '', gatewayIds: [], exposure: 'all', source: [], transport: [] }}
        gatewayOptions={gatewayFixtures.map((gateway) => ({ value: gateway.id, label: gateway.name }))}
        activeSearch=""
        mobileSheetOpen={false}
        density="comfortable"
        isLoading={false}
        itemsCount={gatewayFixtures.length}
        filteredGateways={gatewayFixtures}
        filteredToolRows={[
          {
            gatewayId: 'gw_lab',
            gatewayName: 'Lab Core',
            source: 'lab_service',
            sourceFacet: 'lab',
            transport: 'stdio',
            toolName: 'click',
            description: 'Click an element',
            exposed: true,
          },
        ]}
        onPrimaryLensChange={() => {}}
        onBackToGateways={() => {}}
        onMobileSheetOpenChange={() => {}}
        onDensityChange={() => {}}
        onSearchChange={() => {}}
        onGatewayFilterToggle={() => {}}
        onToolFilterToggle={() => {}}
        onExposureChange={() => {}}
        onClearFilters={() => {}}
        onCreate={() => {}}
        onEdit={() => {}}
        onTest={() => {}}
        onReload={() => {}}
        onDelete={() => {}}
      />
    </SidebarProvider>,
  )

  assert.match(markup, /Configured/)
  assert.match(markup, /Healthy/)
  assert.match(markup, /Disconnected/)
  assert.match(markup, /Discovered tools/)
  assert.match(markup, /aria-label="Comfortable density"/)
  assert.match(markup, /aria-label="Condensed density"/)
  assert.match(markup, /aria-label="Open filters"/)
  assert.match(markup, /Add Gateway/)
  assert.match(markup, />2</)
})
