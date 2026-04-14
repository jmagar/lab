import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildGatewayPatch,
  gatewayInputToSpec,
  normalizeGateway,
  previewExposurePolicy,
} from './gateway-adapter'

test('normalizeGateway maps backend views into UI gateway shape', () => {
  const gateway = normalizeGateway(
    {
      config: {
        name: 'fixture-http',
        url: 'http://127.0.0.1:9001/mcp',
        bearer_token_env: 'FIXTURE_TOKEN',
        proxy_resources: true,
      },
      runtime: {
        name: 'fixture-http',
        tool_count: 3,
        resource_count: 1,
        prompt_count: 1,
      },
    },
    {
      connected: true,
      healthy: true,
    },
    {
      tools: ['alpha.read', 'beta.write', 'gamma.run'],
      resources: ['resource://one'],
      prompts: ['prompt-one'],
    }
  )

  assert.equal(gateway.id, 'fixture-http')
  assert.equal(gateway.transport, 'http')
  assert.equal(gateway.status.connected, true)
  assert.equal(gateway.status.healthy, true)
  assert.equal(gateway.status.discovered_tool_count, 3)
  assert.equal(gateway.status.exposed_tool_count, 3)
  assert.deepEqual(
    gateway.discovery.tools.map((tool) => ({ name: tool.name, exposed: tool.exposed, matched_by: tool.matched_by })),
    [
      { name: 'alpha.read', exposed: true, matched_by: '*' },
      { name: 'beta.write', exposed: true, matched_by: '*' },
      { name: 'gamma.run', exposed: true, matched_by: '*' },
    ]
  )
  assert.deepEqual(gateway.discovery.resources, [
    {
      name: 'resource://one',
      uri: 'resource://one',
    },
  ])
  assert.deepEqual(gateway.discovery.prompts, [{ name: 'prompt-one' }])
})

test('normalizeGateway applies allowlist exposure patterns', () => {
  const gateway = normalizeGateway(
    {
      config: {
        name: 'fixture-stdio',
        command: 'npx',
        args: ['-y', 'server'],
        proxy_resources: false,
        expose_tools: ['alpha.*', 'gamma.run'],
      },
      runtime: {
        name: 'fixture-stdio',
        tool_count: 3,
        resource_count: 0,
        prompt_count: 0,
      },
    },
    {
      connected: false,
      healthy: false,
      last_error: 'connection refused',
    },
    {
      tools: ['alpha.read', 'beta.write', 'gamma.run'],
      resources: [],
      prompts: [],
    }
  )

  assert.equal(gateway.transport, 'stdio')
  assert.equal(gateway.status.connected, false)
  assert.equal(gateway.status.last_error, 'connection refused')
  assert.equal(gateway.status.exposed_tool_count, 2)
  assert.deepEqual(
    gateway.discovery.tools.map((tool) => ({ name: tool.name, exposed: tool.exposed, matched_by: tool.matched_by })),
    [
      { name: 'alpha.read', exposed: true, matched_by: 'alpha.*' },
      { name: 'beta.write', exposed: false, matched_by: null },
      { name: 'gamma.run', exposed: true, matched_by: 'gamma.run' },
    ]
  )
})

test('gatewayInputToSpec converts UI input into backend spec payload', () => {
  const spec = gatewayInputToSpec({
    name: 'fixture-http',
    transport: 'http',
    config: {
      url: 'http://127.0.0.1:9001/mcp',
      bearer_token_env: 'FIXTURE_TOKEN',
      proxy_resources: true,
      expose_tools: ['alpha.*'],
    },
  })

  assert.deepEqual(spec, {
    name: 'fixture-http',
    url: 'http://127.0.0.1:9001/mcp',
    command: null,
    args: [],
    bearer_token_env: 'FIXTURE_TOKEN',
    proxy_resources: true,
    expose_tools: ['alpha.*'],
  })
})

test('buildGatewayPatch clears the opposite transport fields when switching to stdio', () => {
  const patch = buildGatewayPatch({
    name: 'fixture-stdio',
    transport: 'stdio',
    config: {
      command: 'npx',
      args: ['-y', 'server'],
      bearer_token_env: '',
      proxy_resources: false,
    },
  })

  assert.deepEqual(patch, {
    name: 'fixture-stdio',
    url: null,
    command: 'npx',
    args: ['-y', 'server'],
    bearer_token_env: null,
    proxy_resources: false,
  })
})

test('previewExposurePolicy reports matches filtered tools and unmatched patterns', () => {
  const preview = previewExposurePolicy(
    ['alpha.read', 'beta.write', 'gamma.run'],
    ['alpha.*', 'gamma.run', 'missing.*']
  )

  assert.deepEqual(preview, {
    matched_tools: [
      { name: 'alpha.read', matched_by: 'alpha.*' },
      { name: 'gamma.run', matched_by: 'gamma.run' },
    ],
    unmatched_patterns: ['missing.*'],
    filtered_tools: ['beta.write'],
    exposed_count: 2,
    filtered_count: 1,
  })
})
