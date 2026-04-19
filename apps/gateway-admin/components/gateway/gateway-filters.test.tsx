import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { GatewayFilters } from './gateway-filters'

test('gateway filters render aurora control surfaces and clear state affordance', () => {
  const markup = renderToStaticMarkup(
    React.createElement(GatewayFilters, {
      search: 'plex',
      onSearchChange: () => {},
      healthFilter: 'enabled',
      onHealthFilterChange: () => {},
      connectionFilter: 'connected',
      onConnectionFilterChange: () => {},
      typeFilter: 'http',
      onTypeFilterChange: () => {},
    }),
  )

  assert.match(markup, /bg-aurora-panel-medium/)
  assert.match(markup, /bg-aurora-control-surface/)
  assert.match(markup, /Search gateways/)
  assert.match(markup, /Clear filters/i)
})
