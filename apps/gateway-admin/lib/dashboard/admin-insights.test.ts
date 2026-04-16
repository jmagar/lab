import test from 'node:test'
import assert from 'node:assert/strict'

import { mockGateways } from '../api/mock-data.ts'
import {
  buildGatewayActivityFeed,
  buildGatewaySettingsSnapshot,
} from './admin-insights.ts'

test('buildGatewayActivityFeed sorts the newest gateway events first', () => {
  const events = buildGatewayActivityFeed(mockGateways)

  assert.ok(events.length > 0)
  assert.equal(events[0]?.gatewayName, 'database-server')
  assert.equal(events[0]?.kind, 'status')
  assert.match(events[0]?.detail ?? '', /Connection refused/i)
  assert.equal(events[1]?.kind, 'warning')
  assert.match(events[1]?.title ?? '', /CONNECTION_FAILED|DISCOVERY_FAILED/)
})

test('buildGatewaySettingsSnapshot summarizes gateway fleet posture and auth mode', () => {
  const snapshot = buildGatewaySettingsSnapshot(mockGateways, {
    hasApiToken: true,
    hasMockData: true,
  })

  assert.equal(snapshot.authModeLabel, 'API token')
  assert.equal(snapshot.runtimeLabel, 'Mock preview')
  assert.equal(snapshot.totalGateways, 5)
  assert.equal(snapshot.connectedGateways, 4)
  assert.equal(snapshot.disconnectedGateways, 1)
  assert.equal(snapshot.warningCount, 3)
  assert.equal(snapshot.proxyResourceGateways, 4)
  assert.equal(snapshot.bearerTokenGateways, 2)
})
