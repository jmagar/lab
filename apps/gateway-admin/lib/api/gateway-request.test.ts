import test from 'node:test'
import assert from 'node:assert/strict'

import { confirmGatewayParams, gatewayHeaders, gatewayRequestInit } from './gateway-request.ts'

test('gatewayRequestInit omits credentialed requests when bearer auth is configured', () => {
  const init = gatewayRequestInit('gateway.list', {}, 'dev-token')

  assert.equal(init.credentials, undefined)
  assert.equal((init.headers as Record<string, string>).Authorization, 'Bearer dev-token')
  assert.equal(init.method, 'POST')
})

test('gatewayHeaders omits authorization when no token is provided', () => {
  const headers = gatewayHeaders(undefined) as Record<string, string>

  assert.equal(headers['Content-Type'], 'application/json')
  assert.equal('Authorization' in headers, false)
})

test('gatewayRequestInit keeps credentialed requests for session-auth setups', () => {
  const init = gatewayRequestInit('gateway.list', {}, undefined)

  assert.equal(init.credentials, 'include')
  assert.equal((init.headers as Record<string, string>)['Content-Type'], 'application/json')
})

test('confirmGatewayParams marks destructive gateway mutations for explicit confirmation', () => {
  assert.deepEqual(confirmGatewayParams({ id: 'plex' }), {
    confirm: true,
    id: 'plex',
  })
})
