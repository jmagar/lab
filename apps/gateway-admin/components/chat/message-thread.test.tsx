import test from 'node:test'
import assert from 'node:assert/strict'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

import { MessageThread, reduceSelectedMessageId, shouldShowWorkingAssistantBubble } from './message-thread'
import type { ACPMessage, ACPRun } from './types'

function run(): ACPRun {
  return {
    id: 'run-1',
    projectId: 'project-1',
    agentId: 'agent-1',
    provider: 'codex',
    title: 'Run',
    createdAt: new Date('2026-05-04T12:00:00Z'),
    updatedAt: new Date('2026-05-04T12:00:00Z'),
    status: 'idle',
    providerSessionId: 'provider-run-1',
    cwd: '/home/jmagar/workspace/lab',
  }
}

function runWithId(id: string): ACPRun {
  return {
    ...run(),
    id,
    providerSessionId: `provider-${id}`,
  }
}

function message(id: string, text: string, minute: string, overrides: Partial<ACPMessage> = {}): ACPMessage {
  return {
    id,
    runId: 'run-1',
    role: 'user',
    text,
    createdAt: new Date(`2026-05-04T12:${minute}:00Z`),
    isStreaming: false,
    thoughts: [],
    toolCalls: [],
    version: 1,
    ...overrides,
  }
}

test('message thread renders timestamp-ready bubbles with stable message ids', () => {
  const markup = renderToStaticMarkup(
    <MessageThread
      run={run()}
      messages={[message('m1', 'First', '10'), message('m2', 'Second', '20')]}
    />,
  )

  assert.match(markup, /data-message-id="m1"/)
  assert.match(markup, /data-message-id="m2"/)
  assert.match(markup, /12:10 PM UTC/)
  assert.match(markup, /12:20 PM UTC/)
  assert.match(markup, /opacity-0 group-hover\/bubble:opacity-100 group-focus-within\/bubble:opacity-100/)
})

test('working assistant bubble logic remains unchanged for running sessions', () => {
  assert.equal(shouldShowWorkingAssistantBubble(null, [], 'open'), false)
  assert.equal(shouldShowWorkingAssistantBubble({ ...run(), status: 'idle' }, [], 'open'), false)
  assert.equal(shouldShowWorkingAssistantBubble({ ...run(), status: 'waiting_for_permission' }, [], 'open'), false)
  assert.equal(shouldShowWorkingAssistantBubble({ ...run(), status: 'running' }, [], 'connecting'), true)
  assert.equal(shouldShowWorkingAssistantBubble({ ...run(), status: 'running' }, [], 'open'), true)
  assert.equal(shouldShowWorkingAssistantBubble({ ...run(), status: 'running' }, [], 'error'), false)
  assert.equal(
    shouldShowWorkingAssistantBubble(
      { ...run(), status: 'running' },
      [{ ...message('assistant-1', 'Streaming', '30'), role: 'assistant', isStreaming: true }],
      'open',
    ),
    false,
  )
})

test('timestamp selection state handles tap selection and dismiss paths', () => {
  assert.equal(reduceSelectedMessageId(null, { type: 'select', messageId: 'm1' }), 'm1')
  assert.equal(reduceSelectedMessageId('m1', { type: 'select', messageId: 'm2' }), 'm2')
  assert.equal(reduceSelectedMessageId('m1', { type: 'dismiss' }), null)
  assert.equal(reduceSelectedMessageId('m1', { type: 'run-change', runId: runWithId('run-2').id }), null)

  const selectedMarkup = renderToStaticMarkup(
    <MessageThread
      run={runWithId('run-2')}
      messages={[message('shared-id', 'Second run', '20', { runId: 'run-2' })]}
      connectionState="open"
    />,
  )
  assert.doesNotMatch(selectedMarkup, /\sopacity-100"/)
})
