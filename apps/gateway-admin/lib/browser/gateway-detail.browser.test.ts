import test from 'node:test'
import assert from 'node:assert/strict'
import { once } from 'node:events'
import http from 'node:http'
import { spawn } from 'node:child_process'

import { chromium } from 'playwright'

const PORT = 3101
const BASE_URL = `http://127.0.0.1:${PORT}`
const APP_DIR = new URL('../../', import.meta.url)

async function waitForServer(url: string) {
  const deadline = Date.now() + 20_000

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

async function startPreviewServer(t: test.TestContext) {
  const server = spawn(
    '/usr/bin/zsh',
    ['-lc', `LAB_ALLOWED_DEV_ORIGINS=127.0.0.1 NEXT_PUBLIC_MOCK_DATA=true NEXT_PUBLIC_API_TOKEN=dev-token pnpm dev --hostname 127.0.0.1 --port ${PORT}`],
    {
      cwd: APP_DIR,
      stdio: 'ignore',
      env: process.env,
    },
  )

  t.after(async () => {
    server.kill('SIGTERM')
    await once(server, 'exit').catch(() => undefined)
  })

  await waitForServer(`${BASE_URL}/gateway/?id=gw-2`)
}

test('gateway manage tools flow persists after a full reload in mock preview', { concurrency: false }, async (t) => {
  await startPreviewServer(t)

  const browser = await chromium.launch({ headless: true })
  t.after(async () => {
    await browser.close()
  })

  const page = await browser.newPage()
  await page.goto(`${BASE_URL}/gateway/?id=gw-2`, { waitUntil: 'networkidle' })
  await page.evaluate(() => {
    window.localStorage.clear()
  })
  await page.reload({ waitUntil: 'networkidle' })

  await page.getByRole('button', { name: 'Manage Tools' }).click()
  await page.getByRole('checkbox', { name: 'Select all visible' }).click()
  await page.getByRole('button', { name: 'Disable selected' }).click()
  await page.getByRole('button', { name: 'Save changes' }).click()

  await page.getByText('Tool exposure updated successfully').waitFor()
  await assert.doesNotReject(() =>
    page.locator('p, div').filter({ hasText: /^0\/12$/ }).first().waitFor(),
  )

  await page.reload({ waitUntil: 'networkidle' })

  await assert.doesNotReject(() => page.getByRole('button', { name: 'Manage Tools' }).waitFor())
  await assert.doesNotReject(() =>
    page.locator('p, div').filter({ hasText: /^0\/12$/ }).first().waitFor(),
  )
  await assert.doesNotReject(() => page.getByText('12 hidden').waitFor())
})

test('gateway detail uses a compact summary and client config block in mock preview', { concurrency: false }, async (t) => {
  await startPreviewServer(t)

  const browser = await chromium.launch({ headless: true })
  t.after(async () => {
    await browser.close()
  })

  const page = await browser.newPage({ viewport: { width: 1360, height: 960 } })
  await page.goto(`${BASE_URL}/gateway/?id=gw-2`, { waitUntil: 'networkidle' })
  await page.evaluate(() => {
    window.localStorage.clear()
  })
  await page.reload({ waitUntil: 'networkidle' })

  await assert.doesNotReject(() => page.getByText('12/12').first().waitFor())
  await assert.doesNotReject(() => page.getByText('3/3').first().waitFor())
  await assert.doesNotReject(() => page.getByText('2/2').first().waitFor())
  await assert.doesNotReject(() => page.getByText('Expose resources').first().waitFor())
  await assert.doesNotReject(() => page.getByText('"name": "github-server"').waitFor())
  await assert.doesNotReject(() => page.getByText('"type": "http"').waitFor())
  await assert.doesNotReject(() => page.getByText('"url": "http://localhost:3001/mcp"').waitFor())

  assert.equal(await page.getByText('TOOL SURFACE').count(), 0)
  assert.equal(await page.getByText('BEARER ENV').count(), 0)
  assert.equal(await page.getByText('LAB CONTROLS').count(), 0)

  const hasHorizontalOverflow = await page.evaluate(() => {
    const root = document.documentElement
    return root.scrollWidth > root.clientWidth
  })

  assert.equal(hasHorizontalOverflow, false)
})

test('gateway list stays compact without horizontal overflow in mock preview', { concurrency: false }, async (t) => {
  await startPreviewServer(t)

  const browser = await chromium.launch({ headless: true })
  t.after(async () => {
    await browser.close()
  })

  const page = await browser.newPage({ viewport: { width: 1360, height: 960 } })
  await page.goto(`${BASE_URL}/gateways/`, { waitUntil: 'networkidle' })
  await page.evaluate(() => {
    window.localStorage.clear()
  })
  await page.reload({ waitUntil: 'networkidle' })

  await assert.doesNotReject(() => page.getByText('576').first().waitFor())
  await assert.doesNotReject(() => page.getByText('29/29').first().waitFor())

  assert.equal(await page.getByText('Status').count(), 0)
  assert.equal(await page.getByText('TOOLS').count(), 0)
  assert.equal(await page.getByText('EXPOSED').count(), 0)

  const hasHorizontalOverflow = await page.evaluate(() => {
    const root = document.documentElement
    return root.scrollWidth > root.clientWidth
  })

  assert.equal(hasHorizontalOverflow, false)
})
