import test from 'node:test'
import assert from 'node:assert/strict'

import type { DiscoveredTool } from '../types/gateway.ts'
import {
  EXPOSE_NONE_PATTERN,
  applyBulkExposureToDraft,
  buildExposurePolicyFromDraft,
  createExposureDraftFromTools,
  getDraftChangeDescription,
  getDraftExposureSummary,
  getExposureFilterCounts,
  filterToolsForExposureView,
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

test('getExposureFilterCounts reports all, enabled, and hidden buckets', () => {
  assert.deepEqual(getExposureFilterCounts(tools), {
    all: 3,
    enabled: 2,
    hidden: 1,
  })
})

test('filterToolsForExposureView narrows tools by exposure state and search query', () => {
  assert.deepEqual(
    filterToolsForExposureView(tools, 'hidden', ''),
    [tools[1]],
  )

  assert.deepEqual(
    filterToolsForExposureView(tools, 'all', 'gam'),
    [tools[2]],
  )
})

test('getDraftChangeDescription describes pending changes against the current exposure set', () => {
  assert.equal(
    getDraftChangeDescription(['alpha', 'gamma'], ['gamma', 'beta']),
    '1 tool will be enabled, 1 tool will be hidden.',
  )

  assert.equal(
    getDraftChangeDescription(['alpha', 'beta'], ['alpha', 'beta']),
    'No unsaved exposure changes.',
  )
})
