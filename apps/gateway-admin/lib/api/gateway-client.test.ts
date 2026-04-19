import test from 'node:test'
import assert from 'node:assert/strict'

import { __setBrowserSessionStateForTests } from '../auth/session-store.ts'
import { gatewayApi } from './gateway-client.ts'

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
