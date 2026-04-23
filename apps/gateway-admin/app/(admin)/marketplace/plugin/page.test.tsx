import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { PluginDetailContent } from '@/components/marketplace/plugin-detail-content'

test('plugin detail surface renders an empty state for a requested plugin id without loaded data', () => {
  const markup = renderToStaticMarkup(
    React.createElement(PluginDetailContent, {
      pluginId: 'plugin-lab@claude-homelab',
    }),
  )

  assert.match(markup, /Plugin details unavailable|Plugin not found|Plugin not selected/)
})
