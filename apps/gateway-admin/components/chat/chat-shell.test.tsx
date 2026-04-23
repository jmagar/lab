import test from 'node:test'
import assert from 'node:assert/strict'

import {
  ensurePromptRunId,
  integrateCreatedRun,
  sessionCreationOptionsForIntent,
  shouldAutoCreateInitialRun,
} from './chat-shell'
import type { ACPRun } from './types'

function run(id: string, title = id): ACPRun {
  return {
    id,
    projectId: 'workspace',
    agentId: 'codex',
    provider: 'codex',
    title,
    createdAt: new Date('2026-04-23T00:00:00Z'),
    updatedAt: new Date('2026-04-23T00:00:00Z'),
    status: 'idle',
    providerSessionId: `provider-${id}`,
    cwd: '/tmp/workspace',
  }
}

test('shouldAutoCreateInitialRun only allows bootstrap when provider is ready and no run is selected', () => {
  assert.equal(shouldAutoCreateInitialRun(false, 0, null), false)
  assert.equal(shouldAutoCreateInitialRun(true, 1, null), false)
  assert.equal(shouldAutoCreateInitialRun(true, 0, 'run-1'), false)
  assert.equal(shouldAutoCreateInitialRun(true, 0, null), true)
})

test('integrateCreatedRun prepends the new run and removes stale duplicates', () => {
  const created = run('run-2', 'Fresh session')
  const current = [run('run-1', 'Older session'), run('run-2', 'Stale duplicate')]

  assert.deepEqual(integrateCreatedRun(current, created), [created, current[0]!])
})

test('sessionCreationOptionsForIntent only closes the panel for non-bootstrap mobile flows', () => {
  assert.equal(sessionCreationOptionsForIntent('bootstrap', true), undefined)
  assert.deepEqual(sessionCreationOptionsForIntent('manual', true), { closeSessionPanel: true })
  assert.deepEqual(sessionCreationOptionsForIntent('send', false), { closeSessionPanel: false })
})

test('ensurePromptRunId reuses the selected run without creating a new session', async () => {
  let createCalls = 0

  const runId = await ensurePromptRunId(
    'run-7',
    async () => {
      createCalls += 1
      return run('run-created')
    },
    true,
  )

  assert.equal(runId, 'run-7')
  assert.equal(createCalls, 0)
})

test('ensurePromptRunId creates a mobile send session when no run is selected', async () => {
  let receivedOptions: { closeSessionPanel?: boolean } | undefined

  const runId = await ensurePromptRunId(
    null,
    async (options) => {
      receivedOptions = options
      return run('run-3', 'Prompt bootstrap')
    },
    true,
  )

  assert.equal(runId, 'run-3')
  assert.deepEqual(receivedOptions, { closeSessionPanel: true })
})
