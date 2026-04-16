import test from 'node:test'
import assert from 'node:assert/strict'
import { once } from 'node:events'
import { existsSync } from 'node:fs'
import http from 'node:http'
import { spawn } from 'node:child_process'

import { chromium } from 'playwright'

const PORT = 3101
const BASE_URL = `http://127.0.0.1:${PORT}`
const APP_DIR = new URL('../../', import.meta.url)
const OUT_DIR = new URL('../../out/', import.meta.url)

async function waitForServer(url: string) {
  const deadline = Date.now() + 10_000

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

test('gateway manage tools flow persists after a full reload in mock preview', async (t) => {
  assert.ok(existsSync(OUT_DIR), 'Run `pnpm build` before running browser tests so the static preview exists.')

  const server = spawn(
    'python3',
    ['-m', 'http.server', String(PORT), '-d', OUT_DIR.pathname],
    {
      cwd: APP_DIR,
      stdio: 'ignore',
    },
  )

  t.after(async () => {
    server.kill('SIGTERM')
    await once(server, 'exit').catch(() => undefined)
  })

  await waitForServer(`${BASE_URL}/gateway/?id=gw-2`)

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
