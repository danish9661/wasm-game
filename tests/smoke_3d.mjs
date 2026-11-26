// Headless Playwright smoke test for the 3D vertical slice in pkg/.
//
// Usage:
//   PLAYWRIGHT_MODULE=/path/to/playwright-core/index.mjs \
//   CHROME_BIN=/usr/bin/google-chrome-stable \
//   node tests/smoke_3d.mjs
//
// Serves pkg/ over HTTP, loads the shipped page, and asserts that the 3D
// scene is active on the game canvas, drawn pixels are non-background, the
// 2.5D toggle still works, and the browser reports zero console errors.
import { spawn } from 'node:child_process';
import { decodePng } from './png_analyze.mjs';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const root = resolve(new URL('..', import.meta.url).pathname);
const port = Number(process.env.SMOKE_PORT || 8791);
const playwrightModule =
  process.env.PLAYWRIGHT_MODULE || 'playwright-core';
const { chromium } = await import(
  playwrightModule.startsWith('/') || playwrightModule.startsWith('.')
    ? pathToFileURL(resolve(playwrightModule)).href
    : playwrightModule
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let pass = 0;
let fail = 0;
const ok = (m) => { pass++; console.log(`PASS ${m}`); };
const bad = (m) => { fail++; console.log(`FAIL ${m}`); };

const server = spawn('python3', ['-m', 'http.server', String(port), '-d', resolve(root, 'pkg')], {
  stdio: 'ignore',
});
let browser;

try {
  // Wait for the static server.
  let up = false;
  for (let i = 0; i < 30 && !up; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${port}/index.html`);
      up = r.ok;
    } catch { await sleep(200); }
  }
  if (!up) throw new Error('static server did not start');

  browser = await chromium.launch({
    executablePath: process.env.CHROME_BIN || '/usr/bin/google-chrome-stable',
    headless: true,
    args: [
      '--no-sandbox',
      '--disable-dev-shm-usage',
      '--disable-extensions',
      '--no-first-run',
      '--disable-background-networking',
      '--enable-unsafe-webgpu',
      '--use-angle=swiftshader',
      '--enable-features=Vulkan',
      '--use-vulkan=swiftshader',
      '--js-flags=--max-old-space-size=256',
    ],
  });
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  const consoleErrors = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  page.on('pageerror', (e) => consoleErrors.push(`pageerror: ${e.message}`));

  await page.goto(`http://127.0.0.1:${port}/index.html`, { waitUntil: 'domcontentloaded' });
  // Import the *same* module URL the page booted (including its ?v= cache
  // stamp), otherwise wasm-bindgen gets a second, uninitialised instance.
  await page.evaluate(async () => {
    const scripts = [...document.querySelectorAll('script[type="module"]')];
    const text = scripts.map((s) => s.textContent || '').join('\n');
    const spec = text.match(/from\s+['"](\.\/wasm_game\.js[^'"]*)['"]/)?.[1] || './wasm_game.js';
    window.__m = await import(spec);
  });

  // WebGPU init + GLB fetch + first frames.
  await page.waitForFunction(() => {
    try { return !!window.__m && window.__m.app_ready(); } catch { return false; }
  }, { timeout: 60000 });
  await page.waitForFunction(() => {
    try { return window.__m.render3d_enabled(); } catch { return false; }
  }, { timeout: 30000 });
  ok('game canvas switched to the 3D scene');

  await page.waitForFunction(() => /fps=\d+/.test(document.getElementById('log')?.textContent || ''), { timeout: 30000 });
  const stats = await page.evaluate(() => window.__m.get_stats());
  (/mode=3d/.test(stats)) ? ok('stats report mode=3d') : bad(`stats missing mode=3d: ${stats.slice(0, 200)}`);

  const sceneStatus = await page.evaluate(() => window.__m.render3d_status());
  console.log(`       3D scene: ${sceneStatus}`);
  (/models/.test(sceneStatus) && /instances/.test(sceneStatus))
    ? ok('GLB models + instanced groups loaded') : bad(`bad 3D scene status: ${sceneStatus}`);

  // Sample the actual presented #game canvas. Headless screenshots do not
  // composite WebGPU, but toDataURL reads the real surface backing.
  await sleep(500);
  const dataUrl = await page.evaluate(() =>
    document.getElementById('game').toDataURL('image/png'));
  const { width, height, pixels } = decodePng(Buffer.from(dataUrl.split(',')[1], 'base64'));
  const colors = new Set();
  let nonbg = 0;
  let total = 0;
  for (let y = 0; y < height; y += 3) {
    for (let x = 0; x < width; x += 3) {
      const i = (y * width + x) * 4;
      const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
      colors.add(((r >> 4) << 8) | ((g >> 4) << 4) | (b >> 4));
      if (!(r < 32 && g < 36 && b < 48)) nonbg++;
      total++;
    }
  }
  const px = { distinct: colors.size, nonbgPct: (100 * nonbg) / total };
  console.log(`       canvas: ${px.distinct} colours, ${px.nonbgPct.toFixed(1)}% non-background`);
  (px.distinct > 15 && px.nonbgPct > 1)
    ? ok(`3D frame drew pixels (${px.distinct} colours, ${px.nonbgPct.toFixed(1)}% non-background)`)
    : bad(`3D frame looks blank: ${JSON.stringify(px)}`);

  // The legacy path must still be reachable at runtime.
  await page.evaluate(() => window.__m.set_render3d(false));
  await sleep(500);
  const stats2d = await page.evaluate(() => window.__m.get_stats());
  (/mode=2\.5d/.test(stats2d))
    ? ok('2.5D fallback toggle works') : bad(`toggle failed: ${stats2d.slice(0, 200)}`);
  await page.evaluate(() => window.__m.set_render3d(true));
  await sleep(500);
  const stats3d = await page.evaluate(() => window.__m.get_stats());
  (/mode=3d/.test(stats3d))
    ? ok('3D mode can be re-enabled') : bad(`re-enable failed: ${stats3d.slice(0, 200)}`);

  // The original 2.5D renderer must still boot cleanly behind the flag.
  const legacy = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  legacy.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(`?2d: ${msg.text()}`);
  });
  legacy.on('pageerror', (e) => consoleErrors.push(`?2d pageerror: ${e.message}`));
  await legacy.goto(`http://127.0.0.1:${port}/index.html?2d`, { waitUntil: 'domcontentloaded' });
  // The inline boot imports its own module instance; use the page's HUD log.
  await legacy.waitForFunction(
    () => document.getElementById('log')?.textContent?.includes('webgpu initialized'),
    { timeout: 60000 },
  );
  const legacyStatus = await legacy.evaluate(async () => {
    const text = [...document.querySelectorAll('script[type="module"]')]
      .map((s) => s.textContent || '').join('\n');
    const spec = text.match(/from\s+['"](\.\/wasm_game\.js[^'"]*)['"]/)?.[1] || './wasm_game.js';
    const m = await import(spec);
    await new Promise((r) => setTimeout(r, 800));
    return { enabled: m.render3d_enabled(), stats: m.get_stats() };
  });
  (!legacyStatus.enabled && /mode=2\.5d/.test(legacyStatus.stats))
    ? ok('?2d boots the legacy 2.5D renderer') : bad(`?2d fallback failed: ${legacyStatus.stats.slice(0, 200)}`);
  await legacy.close();

  consoleErrors.length === 0
    ? ok('0 console errors')
    : bad(`${consoleErrors.length} console error(s):\n${consoleErrors.slice(0, 8).join('\n')}`);
} catch (e) {
  bad(`smoke test threw: ${e.stack || e}`);
} finally {
  if (browser) await browser.close();
  server.kill('SIGTERM');
}

console.log(`\n3D smoke: ${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
