import { chromium } from 'playwright';

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1568, height: 879 } });
await page.goto('http://127.0.0.1:3101/chat/', { waitUntil: 'networkidle' });
await page.screenshot({ path: '/tmp/chat-preview.png', fullPage: true });
console.log(await page.locator('body').innerText());
await browser.close();
