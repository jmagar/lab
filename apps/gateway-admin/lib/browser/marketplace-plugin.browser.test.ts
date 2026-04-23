import test from 'node:test'
import assert from 'node:assert/strict'
import { once } from 'node:events'
import http from 'node:http'
import { spawn, type ChildProcess } from 'node:child_process'

import { chromium } from 'playwright'

const PORT = 3102
const BASE_URL = `http://127.0.0.1:${PORT}`
const APP_DIR = new URL('../../', import.meta.url)
let previewServer: ChildProcess | null = null
let previewServerReady: Promise<void> | null = null

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
    ['-lc', `LAB_ALLOWED_DEV_ORIGINS=127.0.0.1 NEXT_PUBLIC_API_TOKEN=dev-token pnpm exec next build && python3 -m http.server ${PORT} --directory out --bind 127.0.0.1`],
    {
      cwd: APP_DIR,
      stdio: 'ignore',
      env: process.env,
    },
  )

  previewServerReady = waitForServer(`${BASE_URL}/marketplace/plugin/?id=plugin-lab%40claude-homelab`)
  await previewServerReady
}

test.after(async () => {
  if (!previewServer) return

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

async function mockMarketplaceApi(page: import('playwright').Page) {
  await page.route('**/v1/marketplace', async (route) => {
    const request = route.request()
    if (request.method() !== 'POST') {
      await route.fulfill({ status: 405, body: 'method not allowed' })
      return
    }

    const payload = request.postDataJSON() as { action?: string; params?: Record<string, unknown> }
    const action = payload.action ?? ''

    const plugin = {
      id: 'plugin-lab@claude-homelab',
      name: 'plugin-lab',
      marketplaceId: 'claude-homelab',
      mkt: 'claude-homelab',
      version: '1.0.0',
      ver: '1.0.0',
      description: 'Agents, commands, skills, and scripts for plugin scaffolding.',
      desc: 'Agents, commands, skills, and scripts for plugin scaffolding.',
      tags: ['dev-tools', 'scaffolding', 'templates'],
      installed: true,
      hasUpdate: false,
      installedAt: '2026-04-06T02:21:05.676Z',
      updatedAt: '2026-04-06T02:21:05.676Z',
    }

    const marketplace = {
      id: 'claude-homelab',
      name: 'CLAUDE-HOMELAB',
      owner: 'jmagar',
      githubOwner: 'jmagar',
      ghUser: 'jmagar',
      repo: 'jmagar/claude-homelab',
      source: 'github',
      desc: 'Homelab marketplace',
      description: 'Homelab marketplace',
      autoUpdate: false,
      autoUpdateEnabled: false,
      totalPlugins: 4,
      pluginCount: 4,
      lastUpdated: '2026-04-06T02:21:05.676Z',
      lastUpdatedAt: '2026-04-06T02:21:05.676Z',
    }

    const artifacts = [
      {
        path: 'README.md',
        content: '# plugin-lab\n\nScaffold and review plugin projects.',
        lang: 'markdown',
      },
      {
        path: 'bin/CLAUDE.md',
        content: '# templates/bin/\n\nExecutable helper guidance.',
        lang: 'markdown',
      },
      {
        path: '.claude-plugin/plugin.json',
        content: '{"name":"plugin-lab"}',
        lang: 'json',
      },
    ]

    const workspace = {
      pluginId: plugin.id,
      deployTarget: '/home/jmagar/.claude/plugins/cache/claude-homelab/plugin-lab/1.0.6',
      hasDirtyFiles: false,
      files: [
        {
          path: 'bin/CLAUDE.md',
          lang: 'markdown',
          content: '# templates/bin/\n\nExecutable helper guidance.',
          savedContent: '# templates/bin/\n\nExecutable helper guidance.',
          dirty: false,
        },
        {
          path: '.claude-plugin/plugin.json',
          lang: 'json',
          content: '{"name":"plugin-lab"}',
          savedContent: '{"name":"plugin-lab"}',
          dirty: false,
        },
      ],
    }

    const preview = {
      changed: ['bin/CLAUDE.md'],
      skipped: [],
      removed: [],
      entries: [
        {
          path: 'bin/CLAUDE.md',
          status: 'changed',
          beforeContent: '# templates/bin/\n\nOld helper guidance.',
          afterContent: '# templates/bin/\n\nExecutable helper guidance.',
        },
      ],
      target: workspace.deployTarget,
    }

    let body: unknown
    switch (action) {
      case 'sources.list':
        body = [marketplace]
        break
      case 'plugins.list':
        body = [plugin]
        break
      case 'plugin.artifacts':
        body = artifacts
        break
      case 'plugin.workspace':
        body = workspace
        break
      case 'plugin.deploy.preview':
        body = preview
        break
      default:
        await route.fulfill({
          status: 400,
          contentType: 'application/json',
          body: JSON.stringify({ kind: 'unknown_action', action }),
        })
        return
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(body),
    })
  })
}

test('marketplace plugin detail route renders info and files in mock preview', { concurrency: false }, async (t) => {
  await startPreviewServer()

  const browser = await chromium.launch({ headless: true })
  t.after(async () => {
    await browser.close()
  })

  const page = await browser.newPage({ viewport: { width: 1360, height: 960 } })
  await mockMarketplaceApi(page)

  await page.goto(`${BASE_URL}/marketplace/plugin/?id=plugin-lab%40claude-homelab`, { waitUntil: 'networkidle' })
  await page.evaluate(() => {
    window.localStorage.clear()
  })
  await page.reload({ waitUntil: 'networkidle' })

  await assert.doesNotReject(() => page.getByText('plugin-lab').first().waitFor())
  await assert.doesNotReject(() => page.getByText('CLAUDE-HOMELAB').first().waitFor())
  await assert.doesNotReject(() => page.getByRole('button', { name: 'Files' }).waitFor())
  await assert.doesNotReject(() => page.getByText('Scaffold and review plugin projects.').waitFor())

  await page.getByRole('button', { name: 'Files' }).click()

  await assert.doesNotReject(() => page.getByText('Deploy target:').waitFor())
  await assert.doesNotReject(() => page.getByText('/home/jmagar/.claude/plugins/cache/claude-homelab/plugin-lab/1.0.6').waitFor())
  await assert.doesNotReject(() => page.getByText('bin/CLAUDE.md', { exact: true }).first().waitFor())
  await assert.doesNotReject(() => page.getByRole('button', { name: 'Preview deploy' }).waitFor())
  await assert.doesNotReject(() => page.getByRole('button', { name: 'Save' }).waitFor())
  await assert.doesNotReject(() => page.getByRole('button', { name: 'Deploy', exact: true }).waitFor())
})
