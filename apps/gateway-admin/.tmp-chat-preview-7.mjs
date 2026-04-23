import { chromium } from 'playwright';

async function capture(name, viewport) {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport });
  await page.addInitScript(() => {
    localStorage.setItem('theme', 'dark');
  });
  await page.goto('http://127.0.0.1:3000/chat/', { waitUntil: 'domcontentloaded', timeout: 15000 });
  await page.waitForTimeout(4500);
  await page.screenshot({ path: '/tmp/' + name + '.png', fullPage: true });
  await browser.close();
}

await capture('chat-preview-desktop', { width: 1568, height: 879 });
await capture('chat-preview-mobile', { width: 430, height: 932 });
console.log('done');
