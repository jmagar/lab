import test from 'node:test'
import assert from 'node:assert/strict'
import { once } from 'node:events'
import http from 'node:http'
import { spawn, type ChildProcess } from 'node:child_process'

import { chromium } from 'playwright'

const PORT = 3103
const BASE_URL = `http://127.0.0.1:${PORT}`
const APP_DIR = new URL('../../', import.meta.url)
let previewServer: ChildProcess | null = null
let previewServerReady: Promise<void> | null = null

type BrowserSession = {
  id: string
  providerSessionId: string
  provider: string
  title: string
  cwd: string
  createdAt: string
  updatedAt: string
  status: 'idle'
  agentName: string
  agentVersion: string
  resumable: boolean
}

function session(id: string, title: string): BrowserSession {
  return {
    id,
    providerSessionId: `provider-${id}`,
    provider: 'codex',
    title,
    cwd: '/home/jmagar/workspace/lab',
    createdAt: '2026-04-23T00:00:00Z',
    updatedAt: '2026-04-23T00:00:00Z',
    status: 'idle',
    agentName: 'Codex ACP',
    agentVersion: 'live',
    resumable: true,
  }
}

function sseMessage(sessionId: string, seq: number, text: string) {
  return `data: ${JSON.stringify({
    id: `evt-${sessionId}-${seq}`,
    seq,
    sessionId,
    provider: 'codex',
    kind: 'message.chunk',
    createdAt: '2026-04-23T00:00:00Z',
    role: 'assistant',
    text,
  })}\n\n`
}

async function waitForServer(url: string) {
  const deadline = Date.now() + 60_000

  while (Date.now() < deadline) {
    try {
      const status = await new Promise<number>((resolve, reject) => {
        const request = http.get(url, (response) => {
          resolve(response.statusCode ?? 0)
          response.resume()
        })
        request.on('error', reject)
      })

      if (status >= 200 && status < 500) {
        return
      }
    } catch {
      // Retry until deadline.
    }

    await new Promise((resolve) => setTimeout(resolve, 200))
  }

  throw new Error(`Timed out waiting for preview server at ${url}`)
}

async function startPreviewServer() {
  if (previewServerReady) {
    await previewServerReady
    return
  }

  previewServer = spawn(
    '/usr/bin/zsh',
    ['-lc', `LAB_ALLOWED_DEV_ORIGINS=127.0.0.1 NEXT_PUBLIC_MOCK_DATA=true pnpm exec next build && python3 -m http.server ${PORT} --directory out --bind 127.0.0.1`],
    {
      cwd: APP_DIR,
      stdio: 'ignore',
      env: process.env,
    },
  )

  previewServerReady = waitForServer(`${BASE_URL}/chat/`)
  await previewServerReady
}

test.after(async () => {
  if (!previewServer) {
    return
  }

  previewServer.kill('SIGTERM')
  await Promise.race([
    once(previewServer, 'exit').catch(() => undefined),
    new Promise((resolve) => setTimeout(resolve, 2_000)),
  ])

  if (previewServer.exitCode === null && !previewServer.killed) {
    previewServer.kill('SIGKILL')
    await once(previewServer, 'exit').catch(() => undefined)
  }
})

test('chat shell bootstraps an initial session and resumes session streams from the last sequence on reselection', { concurrency: false }, async (t) => {
  await startPreviewServer()

  const browser = await chromium.launch({ headless: true })
  t.after(async () => {
    await browser.close()
  })

  const page = await browser.newPage({ viewport: { width: 1360, height: 960 } })
  const sessions: BrowserSession[] = []
  const streamSince = new Map<string, string[]>()

  await page.route('**/v1/acp/**', async (route) => {
    const request = route.request()
    const url = new URL(request.url())

    if (url.pathname === '/v1/acp/provider') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          provider: {
            provider: 'codex',
            ready: true,
            command: 'npx',
            args: ['@zed-industries/codex-acp'],
            message: 'ready',
          },
        }),
      })
      return
    }

    if (url.pathname === '/v1/acp/sessions' && request.method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ sessions }),
      })
      return
    }

    if (url.pathname === '/v1/acp/sessions' && request.method() === 'POST') {
      const created =
        sessions.length === 0
          ? session('session-1', 'Bootstrap session')
          : session('session-2', 'Second session')
      sessions.unshift(created)

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ session: created }),
      })
      return
    }

    const eventMatch = url.pathname.match(/^\/v1\/acp\/sessions\/([^/]+)\/events$/)
    if (eventMatch && request.method() === 'GET') {
      const sessionId = decodeURIComponent(eventMatch[1]!)
      const since = url.searchParams.get('since') ?? '0'
      streamSince.set(sessionId, [...(streamSince.get(sessionId) ?? []), since])

      let body = ''
      if (sessionId === 'session-1' && since === '0') {
        body = sseMessage(sessionId, 1, 'Hello session 1')
      } else if (sessionId === 'session-1' && since === '1') {
        body = sseMessage(sessionId, 2, 'Resumed session 1')
      } else if (sessionId === 'session-2' && since === '0') {
        body = sseMessage(sessionId, 1, 'Hello session 2')
      }

      await route.fulfill({
        status: 200,
        contentType: 'text/event-stream',
        body,
      })
      return
    }

    await route.fulfill({
      status: 404,
      contentType: 'application/json',
      body: JSON.stringify({ message: `Unhandled ACP request: ${url.pathname}` }),
    })
  })

  await page.goto(`${BASE_URL}/chat/`, { waitUntil: 'networkidle' })

  await assert.doesNotReject(() => page.getByText('Bootstrap session').first().waitFor())
  await assert.doesNotReject(() => page.getByText('Hello session 1').waitFor())

  await page.getByRole('button', { name: 'Start new session' }).click()

  await assert.doesNotReject(() => page.getByText('Second session').first().waitFor())
  await assert.doesNotReject(() => page.getByText('Hello session 2').waitFor())

  await page.locator('button').filter({ hasText: 'Bootstrap session' }).first().click()

  await assert.doesNotReject(() => page.getByText('Resumed session 1').waitFor())
  assert.deepEqual(streamSince.get('session-1'), ['0', '1'])
  assert.deepEqual(streamSince.get('session-2'), ['0'])
})
