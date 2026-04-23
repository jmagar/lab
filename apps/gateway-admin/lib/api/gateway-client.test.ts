import test from 'node:test'
import assert from 'node:assert/strict'

import { __setBrowserSessionStateForTests, getBrowserSessionState } from '../auth/session-store.ts'
import { GatewayApiError, gatewayApi } from './gateway-client.ts'

type RecordedRequest = {
  action: string
  params: Record<string, unknown>
}

const standardGatewayView = {
  config: {
    name: 'gateway-1',
    url: 'http://gateway.example',
    command: null,
    args: [],
    bearer_token_env: null,
    proxy_resources: false,
    expose_tools: null,
  },
  runtime: {
    name: 'gateway-1',
    tool_count: 1,
    resource_count: 1,
    prompt_count: 1,
    exposed_tool_count: 1,
    exposed_resource_count: 1,
    exposed_prompt_count: 1,
    last_error: null,
  },
}

function jsonResponse(body: unknown) {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: {
      'content-type': 'application/json',
    },
  })
}

async function withGatewayFetch(
  handlers: Record<string, (params: Record<string, unknown>) => Promise<unknown> | unknown>,
  run: (requests: RecordedRequest[]) => Promise<void>,
) {
  const originalFetch = globalThis.fetch
  const requests: RecordedRequest[] = []
  __setBrowserSessionStateForTests({ status: 'unauthenticated' })

  globalThis.fetch = (async (_input, init) => {
    const body = JSON.parse(String(init?.body ?? '{}')) as RecordedRequest
    requests.push(body)

    const handler = handlers[body.action]
    if (!handler) {
      throw new Error(`unexpected action: ${body.action}`)
    }

    const result = await handler(body.params)
    return result instanceof Response ? result : jsonResponse(result)
  }) as typeof fetch

  try {
    await run(requests)
  } finally {
    globalThis.fetch = originalFetch
  }
}

test('gatewayApi.create sends confirm=true with destructive gateway adds', async () => {
  await withGatewayFetch(
    {
      'gateway.add': () => standardGatewayView,
      'gateway.test': () => standardGatewayView.runtime,
      'gateway.discovered_tools': () => ['tool.alpha'],
      'gateway.discovered_resources': () => ['lab://resource.alpha'],
      'gateway.discovered_prompts': () => ['prompt.alpha'],
    },
    async (requests) => {
      await gatewayApi.create({
        name: 'gateway-1',
        transport: 'http',
        config: {
          url: 'http://gateway.example',
        },
      } as never)

      assert.equal(
        requests.find((request) => request.action === 'gateway.add')?.params.confirm,
        true,
      )
    },
  )
})

test('gatewayApi.create sends pasted bearer tokens as a separate payload field', async () => {
  await withGatewayFetch(
    {
      'gateway.add': () => standardGatewayView,
      'gateway.test': () => standardGatewayView.runtime,
      'gateway.discovered_tools': () => ['tool.alpha'],
      'gateway.discovered_resources': () => ['lab://resource.alpha'],
      'gateway.discovered_prompts': () => ['prompt.alpha'],
    },
    async (requests) => {
      await gatewayApi.create({
        name: 'github',
        transport: 'http',
        config: {
          url: 'https://api.githubcopilot.com/mcp/',
          bearer_token_value: 'ghp_secret',
        },
      } as never)

      assert.deepEqual(
        requests.find((request) => request.action === 'gateway.add')?.params.spec,
        {
          name: 'github',
          url: 'https://api.githubcopilot.com/mcp/',
          command: null,
          args: [],
          bearer_token_env: 'LAB_GW_GITHUB_AUTH_HEADER',
          proxy_resources: false,
          expose_tools: null,
        },
      )
      assert.equal(
        requests.find((request) => request.action === 'gateway.add')?.params.bearer_token_value,
        'ghp_secret',
      )
    },
  )
})

test('gatewayApi.update sends confirm=true with destructive gateway updates', async () => {
  await withGatewayFetch(
    {
      'gateway.update': () => standardGatewayView,
      'gateway.test': () => standardGatewayView.runtime,
      'gateway.discovered_tools': () => ['tool.alpha'],
      'gateway.discovered_resources': () => ['lab://resource.alpha'],
      'gateway.discovered_prompts': () => ['prompt.alpha'],
    },
    async (requests) => {
      await gatewayApi.update(
        'gateway-1',
        {
          name: 'gateway-1',
          transport: 'http',
          config: {
            url: 'http://gateway-updated.example',
          },
        } as never,
      )

      assert.equal(
        requests.find((request) => request.action === 'gateway.update')?.params.confirm,
        true,
      )
    },
  )
})

test('gatewayApi.update sends pasted bearer tokens as a separate payload field', async () => {
  await withGatewayFetch(
    {
      'gateway.update': () => standardGatewayView,
      'gateway.test': () => standardGatewayView.runtime,
      'gateway.discovered_tools': () => ['tool.alpha'],
      'gateway.discovered_resources': () => ['lab://resource.alpha'],
      'gateway.discovered_prompts': () => ['prompt.alpha'],
    },
    async (requests) => {
      await gatewayApi.update(
        'github',
        {
          name: 'github',
          transport: 'http',
          config: {
            url: 'https://api.githubcopilot.com/mcp/',
            bearer_token_value: 'ghp_secret',
          },
        } as never,
      )

      assert.equal(
        requests.find((request) => request.action === 'gateway.update')?.params.bearer_token_value,
        'ghp_secret',
      )
    },
  )
})

test('gatewayApi.remove sends confirm=true with destructive gateway removals', async () => {
  await withGatewayFetch(
    {
      'gateway.remove': () => ({ ok: true }),
    },
    async (requests) => {
      await gatewayApi.remove('gateway-1')

      assert.equal(
        requests.find((request) => request.action === 'gateway.remove')?.params.confirm,
        true,
      )
    },
  )
})

test('gatewayApi.reload sends confirm=true with destructive gateway reloads', async () => {
  await withGatewayFetch(
    {
      'gateway.get': () => standardGatewayView,
      'gateway.reload': () => ({ ok: true }),
    },
    async (requests) => {
      await gatewayApi.reload('gateway-1')

      assert.equal(
        requests.find((request) => request.action === 'gateway.reload')?.params.confirm,
        true,
      )
    },
  )
})

test('gatewayApi.setExposurePolicy sends confirm=true when updating a gateway config', async () => {
  await withGatewayFetch(
    {
      'gateway.server.get': () => ({
        id: 'gateway-1',
        name: 'gateway-1',
        source: 'custom_gateway',
      }),
      'gateway.update': () => ({ ok: true }),
    },
    async (requests) => {
      await gatewayApi.setExposurePolicy('gateway-1', {
        mode: 'allowlist',
        patterns: ['tool.alpha'],
      })

      assert.equal(
        requests.find((request) => request.action === 'gateway.update')?.params.confirm,
        true,
      )
    },
  )
})

test('gatewayApi.list does not refresh browser session for non-csrf validation errors', async () => {
  __setBrowserSessionStateForTests({
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 123,
    csrfToken: 'csrf-old',
  })

  const urls: string[] = []
  globalThis.fetch = (async (input) => {
    const url = String(input)
    urls.push(url)

    if (url === '/v1/gateway') {
      return new Response(
        JSON.stringify({
          kind: 'validation_failed',
          message: 'missing required parameter `name`',
        }),
        {
          status: 422,
          headers: {
            'content-type': 'application/json',
            'x-request-id': 'req-gateway-validation-1',
          },
        },
      )
    }

    throw new Error(`unexpected fetch: ${url}`)
  }) as typeof fetch

  await assert.rejects(
    gatewayApi.list(),
    (error: unknown) => {
      assert.ok(error instanceof GatewayApiError)
      assert.equal(error.code, 'validation_failed')
      return true
    },
  )

  assert.deepEqual(getBrowserSessionState(), {
    status: 'authenticated',
    user: { sub: 'browser-user', email: 'browser@example.com' },
    expiresAt: 123,
    csrfToken: 'csrf-old',
  })
  assert.deepEqual(urls, ['/v1/gateway'])
})

test('gatewayApi destructive mutations send confirm=true', async () => {
  const actions: Array<{ action: string; params: Record<string, unknown> }> = []

  globalThis.fetch = (async (input, init) => {
    const url = String(input)
    if (url !== '/v1/gateway') {
      throw new Error(`unexpected fetch: ${url}`)
    }

    const payload = JSON.parse(String(init?.body ?? '{}')) as {
      action: string
      params: Record<string, unknown>
    }
    actions.push(payload)

    if (payload.action === 'gateway.get') {
      return new Response(
        JSON.stringify({
          config: { name: 'plex', proxy_resources: false },
          runtime: { tool_count: 1 },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }

    if (payload.action === 'gateway.add' || payload.action === 'gateway.update') {
      return new Response(
        JSON.stringify({
          config: {
            name: 'plex',
            url: 'https://lab.example.com/mcp',
            proxy_resources: false,
          },
          runtime: {
            tool_count: 1,
            resource_count: 0,
            prompt_count: 0,
            exposed_tool_count: 1,
            exposed_resource_count: 0,
            exposed_prompt_count: 0,
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      )
    }

    if (payload.action === 'gateway.remove' || payload.action === 'gateway.reload') {
      return new Response('null', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }

    if (
      payload.action === 'gateway.discovered_tools' ||
      payload.action === 'gateway.discovered_resources' ||
      payload.action === 'gateway.discovered_prompts'
    ) {
      return new Response('[]', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })
    }

    throw new Error(`unexpected action: ${payload.action}`)
  }) as typeof fetch

  await gatewayApi.create({
    name: 'plex',
    transport: 'http',
    config: { url: 'https://lab.example.com/mcp' },
  })
  await gatewayApi.update('plex', { name: 'plex-updated' })
  await gatewayApi.remove('plex')
  await gatewayApi.reload('plex')

  const destructiveActions = actions.filter(({ action }) =>
    ['gateway.add', 'gateway.update', 'gateway.remove', 'gateway.reload'].includes(action),
  )

  assert.equal(destructiveActions.length, 4)
  for (const action of destructiveActions) {
    assert.equal(action.params.confirm, true)
  }
})

test('gatewayApi.get applies virtual-server MCP policy to in-process tool exposure', async () => {
  await withGatewayFetch(
    {
      'gateway.server.get': () => ({
        id: 'github-chat',
        name: 'github-chat',
        source: 'in_process',
        configured: true,
        enabled: true,
        connected: true,
        discovered_tool_count: 2,
        exposed_tool_count: 1,
        discovered_resource_count: 0,
        exposed_resource_count: 0,
        discovered_prompt_count: 0,
        exposed_prompt_count: 0,
        surfaces: {
          cli: { enabled: false, connected: false },
          api: { enabled: false, connected: false },
          mcp: { enabled: true, connected: true },
          webui: { enabled: false, connected: false },
        },
        warnings: [],
        config_summary: {
          transport: 'in_process',
          target: 'github-chat',
        },
      }),
      'gateway.service_config.get': () => ({
        service: 'github-chat',
        configured: true,
        fields: [],
      }),
      'gateway.service_actions': () => ([
        { name: 'index_repository', description: 'Index a GitHub repository', destructive: false },
        { name: 'query_repository', description: 'Query a GitHub repository', destructive: false },
      ]),
      'gateway.virtual_server.get_mcp_policy': () => ({
        allowed_actions: ['query_repository'],
      }),
    },
    async (requests) => {
      const gateway = await gatewayApi.get('github-chat')

      assert.deepEqual(
        gateway.discovery.tools.map((tool) => ({
          name: tool.name,
          exposed: tool.exposed,
          matched_by: tool.matched_by,
        })),
        [
          { name: 'index_repository', exposed: false, matched_by: null },
          { name: 'query_repository', exposed: true, matched_by: 'query_repository' },
        ],
      )

      assert.deepEqual(
        requests.map((request) => request.action),
        [
          'gateway.server.get',
          'gateway.service_config.get',
          'gateway.service_actions',
          'gateway.virtual_server.get_mcp_policy',
        ],
      )
    },
  )
})
