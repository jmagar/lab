import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { MessageBubble } from './message-bubble'
import type { ACPMessage } from './types'

function assistantMessage(overrides: Partial<ACPMessage> = {}): ACPMessage {
  return {
    id: 'message-1',
    runId: 'run-1',
    role: 'assistant',
    text: 'Done.',
    createdAt: new Date('2026-05-04T12:00:00Z'),
    isStreaming: true,
    thoughts: ['Checked the current chat surface.'],
    toolCalls: [
      {
        id: 'tool-1',
        title: 'Read CLAUDE.md',
        status: 'completed',
        kind: 'read',
        input: { path: 'CLAUDE.md' },
        locations: ['CLAUDE.md'],
      },
      {
        id: 'tool-2',
        title: 'Read README.md',
        status: 'completed',
        kind: 'read',
        input: { path: 'README.md' },
        locations: ['README.md'],
      },
    ],
    version: 1,
    ...overrides,
  }
}

test('renders reasoning summary separately from agent actions', () => {
  const markup = renderToStaticMarkup(<MessageBubble message={assistantMessage()} />)

  assert.match(markup, /Reasoning Summary/)
  assert.match(markup, /Reasoning/)
  assert.match(markup, /Checked the current chat surface/)
  assert.match(markup, /Agent Actions/)
  assert.match(markup, /Read 2 files/)
  assert.doesNotMatch(markup, /Chain of Thought/)

  assert.ok(
    markup.indexOf('Reasoning Summary') < markup.indexOf('Agent Actions'),
    'reasoning panel should render before the separate actions panel',
  )
})
