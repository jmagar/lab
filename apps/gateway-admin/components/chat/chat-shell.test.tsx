import test from 'node:test'
import assert from 'node:assert/strict'

import { integrateCreatedRun, shouldAutoCreateInitialRun } from './chat-shell'
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
