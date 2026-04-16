import test from 'node:test'
import assert from 'node:assert/strict'

import type { DiscoveredTool } from '../types/gateway.ts'
import {
  EXPOSE_NONE_PATTERN,
  applyBulkExposureToDraft,
  buildExposurePolicyFromDraft,
  createExposureDraftFromTools,
  getDraftExposureSummary,
} from './tool-exposure-draft.ts'

const tools: DiscoveredTool[] = [
  { name: 'alpha', exposed: true, matched_by: '*', description: 'Alpha tool' },
  { name: 'beta', exposed: false, matched_by: null, description: 'Beta tool' },
  { name: 'gamma', exposed: true, matched_by: 'gamma', description: 'Gamma tool' },
]

test('createExposureDraftFromTools seeds selected names from exposed tools', () => {
  assert.deepEqual(createExposureDraftFromTools(tools), ['alpha', 'gamma'])
})

test('applyBulkExposureToDraft enables and disables selected tools deterministically', () => {
  const afterEnable = applyBulkExposureToDraft(['alpha'], ['beta', 'gamma'], true)
  assert.deepEqual(afterEnable, ['alpha', 'beta', 'gamma'])

  const afterDisable = applyBulkExposureToDraft(afterEnable, ['alpha', 'gamma'], false)
  assert.deepEqual(afterDisable, ['beta'])
})

test('buildExposurePolicyFromDraft maps all selected tools to expose_all', () => {
  assert.deepEqual(
    buildExposurePolicyFromDraft(['alpha', 'beta'], ['beta', 'alpha']),
    { mode: 'expose_all', patterns: [] },
  )
})

test('buildExposurePolicyFromDraft maps no selected tools to expose none sentinel', () => {
  assert.deepEqual(
    buildExposurePolicyFromDraft(['alpha', 'beta'], []),
    { mode: 'allowlist', patterns: [EXPOSE_NONE_PATTERN] },
  )
})

test('getDraftExposureSummary reports exposed over total counts', () => {
  assert.equal(getDraftExposureSummary(['alpha', 'beta', 'gamma'], ['beta']).label, '1/3')
})
