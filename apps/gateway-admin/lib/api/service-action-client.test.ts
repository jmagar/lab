import test from 'node:test'
import assert from 'node:assert/strict'

import { safeFanout } from './service-action-client'

test('safeFanout returns per-item failures without rejecting the whole fan-out', async () => {
  const results = await safeFanout([1, 2, 3], async (item) => {
    if (item === 2) {
      throw new Error('bad item')
    }
    return item * 10
  })

  assert.deepEqual(
    results.map((result) => result.ok),
    [true, false, true],
  )
  assert.deepEqual(
    results.map((result) => result.item),
    [1, 2, 3],
  )
  assert.equal(results[0].ok && results[0].value, 10)
  assert.equal(!results[1].ok && results[1].error instanceof Error, true)
  assert.equal(results[2].ok && results[2].value, 30)
})
