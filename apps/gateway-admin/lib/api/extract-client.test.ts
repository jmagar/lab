import test from 'node:test'
import assert from 'node:assert/strict'

import { __setBrowserSessionStateForTests } from '../auth/session-store.ts'
import { ExtractApiError, extractApi } from './extract-client.ts'

test('extractApi.scan performs a fleet scan when uri is omitted', async () => {
  __setBrowserSessionStateForTests({ status: 'unauthenticated' })

  let requestUrl = ''
  let requestInit: RequestInit | undefined
  globalThis.fetch = async (input, init) => {
    requestUrl = String(input)
    requestInit = init
    return new Response(
      JSON.stringify({
        target: { mode: 'fleet' },
        found: [],
        creds: [],
        warnings: [],
      }),
      { status: 200 },
    )
  }

  const report = await extractApi.scan()

  assert.equal(requestUrl, '/v1/extract')
  assert.equal(requestInit?.method, 'POST')
  assert.deepEqual(JSON.parse(String(requestInit?.body)), {
    action: 'scan',
    params: {},
  })
  assert.equal(report.target.mode, 'fleet')
})

test('extractApi.scan performs a targeted scan when uri is provided', async () => {
  __setBrowserSessionStateForTests({ status: 'unauthenticated' })

  let requestInit: RequestInit | undefined
  globalThis.fetch = async (_input, init) => {
    requestInit = init
    return new Response(
      JSON.stringify({
        target: { mode: 'targeted', uri: 'squirts:/mnt/appdata' },
        found: ['overseerr'],
        creds: [],
        warnings: [],
      }),
      { status: 200 },
    )
  }

  await extractApi.scan('squirts:/mnt/appdata')

  assert.deepEqual(JSON.parse(String(requestInit?.body)), {
    action: 'scan',
    params: { uri: 'squirts:/mnt/appdata' },
  })
})

test('extractApi.scan surfaces backend errors with status and kind', async () => {
  __setBrowserSessionStateForTests({ status: 'unauthenticated' })

  globalThis.fetch = async () =>
    new Response(
      JSON.stringify({
        kind: 'invalid_param',
        message: 'missing required param',
      }),
      { status: 400 },
    )

  await assert.rejects(
    extractApi.scan(''),
    (error: unknown) =>
      error instanceof ExtractApiError
      && error.status === 400
      && error.code === 'invalid_param',
  )
})
