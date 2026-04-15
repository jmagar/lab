import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildGatewayPatch,
  gatewayInputToSpec,
  normalizeGateway,
  normalizeServerView,
  previewExposurePolicy,
  probeStatusFromRuntime,
} from './gateway-adapter.ts'

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
  assert.equal(gateway.source, 'custom_gateway')
  assert.equal(gateway.surfaces?.mcp.enabled, true)
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

test('normalizeServerView maps unified server rows into list-friendly gateway cards', () => {
  const gateway = normalizeServerView({
    id: 'plex',
    name: 'plex',
    source: 'lab_service',
    configured: true,
    enabled: false,
    connected: false,
    warnings: [],
    config_summary: {
      transport: 'lab_service',
      target: 'plex',
    },
  })

  assert.equal(gateway.id, 'plex')
  assert.equal(gateway.transport, 'lab_service')
  assert.equal(gateway.source, 'lab_service')
  assert.equal(gateway.enabled, false)
  assert.equal(gateway.surfaces?.mcp.enabled, false)
  assert.equal(gateway.status.connected, false)
  assert.equal(gateway.config.url, undefined)
  assert.equal(gateway.config.command, undefined)
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

test('previewExposurePolicy treats an empty allowlist as expose-all', () => {
  const preview = previewExposurePolicy(
    ['alpha.read', 'beta.write'],
    []
  )

  assert.deepEqual(preview, {
    matched_tools: [
      { name: 'alpha.read', matched_by: '*' },
      { name: 'beta.write', matched_by: '*' },
    ],
    unmatched_patterns: [],
    filtered_tools: [],
    exposed_count: 2,
    filtered_count: 0,
  })
})

test('previewExposurePolicy supports leading wildcard patterns', () => {
  const preview = previewExposurePolicy(
    ['github.search_repos', 'gitlab.search_projects'],
    ['*search_*']
  )

  assert.deepEqual(preview, {
    matched_tools: [
      { name: 'github.search_repos', matched_by: '*search_*' },
      { name: 'gitlab.search_projects', matched_by: '*search_*' },
    ],
    unmatched_patterns: [],
    filtered_tools: [],
    exposed_count: 2,
    filtered_count: 0,
  })
})

test('probeStatusFromRuntime marks zero-capability gateways unhealthy', () => {
  assert.deepEqual(
    probeStatusFromRuntime({
      name: 'swag',
      tool_count: 0,
      resource_count: 0,
      prompt_count: 0,
      last_error: 'stdio handshake failed: unexpected HTTP response on stdout',
    }),
    {
      connected: false,
      healthy: false,
      last_error: 'stdio handshake failed: unexpected HTTP response on stdout',
    }
  )

  assert.deepEqual(
    probeStatusFromRuntime({
      name: 'fixture-stdio',
      tool_count: 3,
      resource_count: 0,
      prompt_count: 0,
    }),
    {
      connected: true,
      healthy: true,
    }
  )
})

test('normalizeGateway turns specific probe failures into actionable warnings', () => {
  const gateway = normalizeGateway(
    {
      config: {
        name: 'syslog-http',
        url: 'http://127.0.0.1:3100/mcp',
        proxy_resources: true,
      },
      runtime: {
        name: 'syslog-http',
        tool_count: 0,
        resource_count: 0,
        prompt_count: 0,
      },
    },
    {
      connected: false,
      healthy: false,
      last_error: 'Connection refused while probing http://127.0.0.1:3100/mcp',
    },
    {
      tools: [],
      resources: [],
      prompts: [],
    }
  )

  assert.equal(gateway.status.last_error, 'Connection refused while probing http://127.0.0.1:3100/mcp')
  assert.deepEqual(gateway.warnings, [
    {
      code: 'connection_error',
      message: 'Connection refused while probing http://127.0.0.1:3100/mcp',
      timestamp: gateway.warnings[0]?.timestamp,
    },
  ])
})

test('normalizeGateway humanizes auth failures for operator-facing UI', () => {
  const gateway = normalizeGateway(
    {
      config: {
        name: 'swag',
        url: 'https://swag.tootie.tv/mcp',
        proxy_resources: true,
      },
      runtime: {
        name: 'swag',
        tool_count: 0,
        resource_count: 0,
        prompt_count: 0,
      },
    },
    {
      connected: false,
      healthy: false,
      last_error:
        'Send message error Transport [rmcp::transport::worker::WorkerTransport<rmcp::transport::streamable_http_client::StreamableHttpClientWorker<reqwest::async_impl::client::Client>>] error: Auth required, when send initialize request',
    },
    {
      tools: [],
      resources: [],
      prompts: [],
    }
  )

  assert.equal(
    gateway.status.last_error,
    'Authentication is required by https://swag.tootie.tv/mcp. Configure `bearer_token_env` with a valid upstream token, then reload this gateway.'
  )
  assert.equal(gateway.warnings[0]?.code, 'auth_required')
})
