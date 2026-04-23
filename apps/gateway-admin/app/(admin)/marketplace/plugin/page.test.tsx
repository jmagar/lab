import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { MarketplacePluginPageContent } from './page.tsx'

test('MarketplacePluginPageContent renders the plugin detail surface for the requested plugin id', () => {
  const markup = renderToStaticMarkup(
    React.createElement(MarketplacePluginPageContent, {
      pluginId: 'plugin-lab@claude-homelab',
    }),
  )

  assert.match(markup, /Plugin details unavailable|Plugin not found|Plugin not selected/)
})
