// Independent QA loop: drives the shipped build in a real browser and locks
// down the gameplay/UI defects found during the QA pass (see docs/QA_LOOP.md).
//
// Usage (playwright-core and Chrome required):
//   PLAYWRIGHT_MODULE=/tmp/opencode/node_modules/playwright-core/index.mjs \
//   CHROME_BIN=/usr/bin/google-chrome-stable node tests/qa_loop.mjs
//
// Serves pkg/ over HTTP, boots New Game, exercises controls and menus, checks
// desktop + mobile layout, saves/loads, day/night and screenshots every stage
// into docs/qa-evidence/. Exits non-zero on any failed assertion or console error.
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { pathToFileURL, fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('..', import.meta.url)));
const port = Number(process.env.QA_PORT || 8795);
const shots = resolve(root, 'docs/qa-evidence');
mkdirSync(shots, { recursive: true });
const playwrightModule = process.env.PLAYWRIGHT_MODULE || 'playwright-core';
const { chromium } = await import(
  playwrightModule.startsWith('/') || playwrightModule.startsWith('.')
    ? pathToFileURL(resolve(playwrightModule)).href
    : playwrightModule
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let pass = 0, fail = 0;
const ok = (m) => { pass++; console.log(`PASS ${m}`); };
const bad = (m) => { fail++; console.log(`FAIL ${m}`); };
const check = (cond, good, badMsg) => cond ? ok(good) : bad(badMsg);

const server = spawn('python3', ['-m', 'http.server', String(port), '-d', resolve(root, 'pkg')], { stdio: 'ignore' });
let browser;
const consoleErrors = [];

async function bootPage(page, query = '') {
  await page.goto(`http://127.0.0.1:${port}/index.html${query}`, { waitUntil: 'domcontentloaded' });
  // Import the exact module URL the page booted, including its ?v= stamp:
  // a second, uninitialised wasm-bindgen instance would fail silently.
  await page.evaluate(async () => {
    const text = [...document.querySelectorAll('script[type="module"]')]
      .map((s) => s.textContent || '').join('\n');
    const spec = text.match(/from\s+['"](\.\/wasm_game\.js[^'"]*)['"]/)?.[1] || './wasm_game.js';
    window.__m = await import(spec);
  });
  await page.waitForFunction(() => {
    try { return window.__m && window.__m.app_ready(); } catch { return false; }
  }, { timeout: 60000 });
}

try {
  for (let i = 0; i < 40; i++) {
    try { const r = await fetch(`http://127.0.0.1:${port}/index.html`); if (r.ok) break; } catch { /* retry */ }
    await sleep(200);
  }

  browser = await chromium.launch({
    executablePath: process.env.CHROME_BIN || '/usr/bin/google-chrome-stable',
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage', '--disable-extensions', '--no-first-run',
      '--disable-background-networking', '--enable-unsafe-webgpu', '--use-angle=swiftshader',
      '--enable-features=Vulkan', '--use-vulkan=swiftshader', '--js-flags=--max-old-space-size=256'],
  });

  // ---------- Desktop ----------
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  page.setDefaultTimeout(30000);
  page.on('console', (m) => { if (m.type() === 'error') consoleErrors.push(m.text()); });
  page.on('pageerror', (e) => consoleErrors.push(`pageerror: ${e.message}`));
  await bootPage(page);
  await sleep(400);

  const shot = async (name) => page.screenshot({ path: resolve(shots, `${name}.jpg`), type: 'jpeg', quality: 70 });
  const stats = () => page.evaluate(() => window.__m.get_stats());
  const field = async (re) => (String(await stats()).match(re) || [])[1] || null;
  const panelClass = (id) => page.locator(`#${id}`).getAttribute('class');
  const toasts = () => page.evaluate(() => [...document.querySelectorAll('#toasts .toast')].map((t) => t.textContent));
  const clearToasts = () => page.evaluate(() => { document.getElementById('toasts').innerHTML = ''; });

  check((await panelClass('bootmenu')).includes('overlay') && !(await panelClass('bootmenu')).includes('hidden'),
    'boot menu is shown on first load', 'boot menu missing');
  await shot('01-boot');

  // New Game + movement + attack.
  await page.click('#btn-new');
  await sleep(700);
  const posOf = () => page.evaluate(() => window.__m.get_stats().match(/player=\(([-\d.]+),([-\d.]+)\)/).slice(1).map(Number));
  const p0 = await posOf();
  await shot('02-new-game');
  check(p0.every(Number.isFinite) && Number(await field(/seed=(\d+)/)) > 0,
    `New Game starts a world (seed ${await field(/seed=(\d+)/)})`, 'New Game did not start');

  await page.keyboard.down('w'); await sleep(800); await page.keyboard.up('w'); await sleep(200);
  const p1 = await posOf();
  check(Math.abs(p1[0] - p0[0]) + Math.abs(p1[1] - p0[1]) > 0.3,
    `WASD moves the player (${p0} -> ${p1})`, `movement did not change position (${p0} -> ${p1})`);
  check(Number(await field(/kev=(\d+)/)) > 0, 'keydown events reach the engine', 'no keydown events reached the engine');

  const atk0 = Number(await field(/atk=(\d+)/));
  await page.keyboard.press('j'); await sleep(250);
  check(Number(await field(/atk=(\d+)/)) > atk0, 'J triggers an attack', 'J did not attack');

  // Inventory (I).
  await page.keyboard.press('i'); await sleep(200);
  check(!(await panelClass('inventory')).includes('hidden'), 'I opens Inventory', 'I did not open Inventory');
  await shot('03-inventory');
  await page.keyboard.press('i'); await sleep(200);
  check((await panelClass('inventory')).includes('hidden'), 'I closes Inventory', 'I did not close Inventory');

  // Regression: U must open Equipment without silently building a farm plot.
  const stBeforeU = Number(await field(/structures=(\d+)/));
  await page.keyboard.press('u'); await sleep(250);
  check(!(await panelClass('equipment')).includes('hidden'), 'U opens Equipment', 'U did not open Equipment');
  check(Number(await field(/structures=(\d+)/)) === stBeforeU,
    'U outside build mode does not build a Farm Plot', 'U built a Farm Plot while opening Equipment');
  check(await field(/build=(\d)/) === '0', 'Equipment key leaves build mode off', 'U left build mode on');
  await shot('04-equipment');

  // Regression: Esc closes the topmost panel before opening Settings.
  await page.keyboard.press('Escape'); await sleep(200);
  check((await panelClass('equipment')).includes('hidden') && (await panelClass('settings')).includes('hidden'),
    'Esc closes Equipment without opening Settings', 'Esc did not close the topmost panel');
  await page.keyboard.press('Escape'); await sleep(200);
  check(!(await panelClass('settings')).includes('hidden'), 'Esc opens Settings when no panel is up', 'Esc did not open Settings');
  await shot('05-settings');
  // Settings controls are live.
  await page.locator('#set-mute').check();
  check(await page.evaluate(() => JSON.parse(localStorage.getItem('starfall_settings')).muted) === true,
    'Settings mute persists to localStorage', 'mute setting did not persist');
  await page.locator('#set-mute').uncheck();
  await page.click('#btn-resume'); await sleep(150);

  // Regression: build hint must appear with the inventory closed.
  await page.keyboard.press('q'); await sleep(300);
  check(!(await panelClass('buildhint')).includes('hidden'),
    'Q shows the build hint with Inventory closed', 'build mode had no on-screen hint');
  await shot('06-build-mode');
  await page.keyboard.press('Escape'); await sleep(200);
  check((await panelClass('buildhint')).includes('hidden'), 'Esc exits build mode', 'Esc did not exit build mode');
  check((await panelClass('settings')).includes('hidden'), 'Esc does not open Settings while exiting build mode',
    'exiting build mode also opened Settings');

  // Round-2 regression: a rejected placement must explain itself. Before the
  // fix, pressing F with no resources / on a blocked tile did nothing at all.
  await clearToasts();
  const stBeforeF = Number(await field(/structures=(\d+)/));
  await page.keyboard.press('f'); await sleep(300);
  const stAfterF = Number(await field(/structures=(\d+)/));
  const buildFeedback = await toasts();
  check(stAfterF > stBeforeF || buildFeedback.some((t) => /can't place/i.test(t)),
    `F builds or explains why it failed (structures ${stBeforeF} -> ${stAfterF})`,
    `F was silently ignored (structures ${stBeforeF} -> ${stAfterF}, toasts ${JSON.stringify(buildFeedback)})`);
  await clearToasts();

  // Round-2 regression: the help text must advertise the live Codex glyph.
  // The mechanics merge left "(L)" behind even though Tab opens the Codex.
  const helpText = await page.evaluate(() => document.getElementById('help-panel').textContent);
  check(/Codex \(Tab\)/i.test(helpText) && !/Codex \(L\)/i.test(helpText),
    'help text advertises Tab for the Codex', 'help text still advertises L for the Codex');

  // Regression: codex uses Tab; L keeps its documented cooking binding.
  await clearToasts();
  await page.keyboard.press('Tab'); await sleep(250);
  check(!(await panelClass('codex')).includes('hidden'), 'Tab opens the Codex', 'Tab did not open the Codex');
  check(!(await toasts()).some((t) => /campfire/i.test(t)),
    'opening the Codex does not fire cook feedback', 'Codex shortcut fired cook feedback');
  await shot('07-codex');
  await page.keyboard.press('Escape'); await sleep(200);
  check((await panelClass('codex')).includes('hidden'), 'Esc closes the Codex', 'Esc did not close the Codex');
  await clearToasts();
  await page.keyboard.press('l'); await sleep(250);
  check((await panelClass('codex')).includes('hidden'), 'L no longer opens the Codex', 'L still opens the Codex');
  check((await toasts()).some((t) => /campfire/i.test(t)), 'L still cooks at a campfire', 'L lost its cook binding');
  await clearToasts();
  await page.keyboard.press('0'); await sleep(250);
  check((await toasts()).some((t) => /campfire/i.test(t)), '0 also cooks (documented hotkey)', 'Digit0 lost its cook binding');
  await clearToasts();

  // Regression: 3 toggles the renderer without selecting build slot 2.
  const mode0 = await field(/mode=(\S+)/);
  await page.keyboard.press('3'); await sleep(400);
  const mode1 = await field(/mode=(\S+)/);
  check(mode0 !== mode1 && /^(3d|2\.5d)$/.test(mode1),
    `3 toggles renderer (${mode0} -> ${mode1})`, `3 did not toggle renderer (${mode0} -> ${mode1})`);
  check(await field(/build=(\d)/) === '0', '3 does not silently enter build mode', '3 entered build mode');
  await shot('08-3d-toggle');
  await page.keyboard.press('3'); await sleep(300);
  check((await field(/mode=(\S+)/)) === mode0, '3 toggles back to the original renderer', '3 did not restore renderer');

  // Regression: maxhp/structures are parsed so low-HP feedback works.
  const parsed = await page.evaluate(() => window.__qa.parseStats(window.__m.get_stats()));
  check(parsed.maxhp > 0 && Number.isFinite(parsed.structures) && Number.isFinite(parsed.build),
    `stats parsed: maxhp=${parsed.maxhp} structures=${parsed.structures} build=${parsed.build}`,
    'get_stats fields missing from parseStats');
  const hurt = await page.evaluate(() => {
    const s = window.__m.get_stats().replace(/(\bhp=)[\d.]+/, '$110');
    window.__qa.updateHud(window.__qa.parseStats(s));
    return {
      critical: document.getElementById('bar-hp').parentElement.classList.contains('critical-low'),
      vignette: document.getElementById('vignette').className === '',
    };
  });
  check(hurt.critical && hurt.vignette, 'low HP lights the critical bar + vignette', 'low-HP feedback still dead');

  // Save / load round trip through the real UI.
  for (const [id, btn] of [['codex', '#codex-close'], ['inventory', '#inv-close'],
    ['equipment', '#eq-close'], ['help-panel', '#btn-help-close'], ['settings', '#btn-resume']]) {
    if (!(await panelClass(id)).includes('hidden')) { await page.click(btn); await sleep(120); }
  }
  await page.click('#gear'); await sleep(200);
  check(!(await panelClass('settings')).includes('hidden'), 'gear opens Settings for the save round-trip', 'gear did not open Settings');
  await page.click('#btn-save');
  await page.waitForFunction(async () => {
    try {
      const db = await new Promise((res, rej) => {
        const r = indexedDB.open('starfall', 1);
        r.onsuccess = () => res(r.result); r.onerror = () => rej(r.error);
      });
      const value = await new Promise((res) => {
        const q = db.transaction('saves', 'readonly').objectStore('saves').get('slot1');
        q.onsuccess = () => res(q.result); q.onerror = () => res(null);
      });
      return !!value;
    } catch { return false; }
  }, null, { timeout: 10000 });
  ok('Save Game commits to IndexedDB');
  await page.click('#btn-resume'); await sleep(150);
  const saved = await page.evaluate(() => window.__m.get_stats().match(/player=\(([-\d.]+),([-\d.]+)\).*seed=(\d+)/).slice(1));
  // The page autosaves on beforeunload, so a reload persists the live state;
  // the round trip below checks that Continue restores exactly that state.
  await page.reload({ waitUntil: 'domcontentloaded' });
  await bootPage(page);
  await sleep(250);
  check(!(await page.locator('#btn-continue').evaluate((e) => e.disabled)), 'Continue is enabled after a save', 'Continue disabled after save');
  await page.click('#btn-continue'); await sleep(500);
  const loaded = await page.evaluate(() => window.__m.get_stats().match(/player=\(([-\d.]+),([-\d.]+)\).*seed=(\d+)/).slice(1));
  check(saved[2] === loaded[2] && saved[0] === loaded[0] && saved[1] === loaded[1],
    `save/load restores seed + position (${saved} -> ${loaded})`, `save/load drifted (${saved} -> ${loaded})`);
  await shot('09-loaded');

  // Day/night: the shared clock keeps ticking (10-minute day, no debug setter).
  const t0 = await field(/time=(\d{2}:\d{2})/);
  await sleep(1600);
  const t1 = await field(/time=(\d{2}:\d{2})/);
  check(t0 !== t1 && /^\d{2}:\d{2}$/.test(t1), `day/night clock advances (${t0} -> ${t1})`, `clock frozen at ${t0}`);
  const weather = Number(await field(/weather=(\d)/));
  check(weather >= 0 && weather <= 4, `weather state is valid (${weather})`, `invalid weather ${weather}`);
  await shot('10-day-night');

  // ---------- Mobile ----------
  const mctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  const mp = await mctx.newPage();
  mp.on('console', (m) => { if (m.type() === 'error') consoleErrors.push(`mobile: ${m.text()}`); });
  mp.on('pageerror', (e) => consoleErrors.push(`mobile pageerror: ${e.message}`));
  await bootPage(mp);
  await mp.click('#btn-new'); await sleep(600);
  check(await mp.evaluate(() => document.body.classList.contains('mobile')),
    'mobile viewport switches to the phone layout', 'phone layout not selected');
  check(!(await mp.locator('#touch').getAttribute('class')).includes('hidden'),
    'touch controls are visible on a phone', 'touch controls hidden on phone');
  await mp.screenshot({ path: resolve(shots, '11-mobile-game.jpg'), type: 'jpeg', quality: 70 });

  const openFab = async () => { await mp.click('#touch-more'); await sleep(200); };
  const fabButton = (key) => mp.locator(`#touch .tbtn.act[data-key="${key}"]`);
  await openFab();
  check(await fabButton('KeyU').evaluate((e) => e.classList.contains('available')),
    'Equipment is offered in the mobile action deck', 'mobile deck has no Equipment action');
  await fabButton('KeyU').click(); await sleep(250);
  check(!(await mp.locator('#equipment').getAttribute('class')).includes('hidden'),
    'touch Equipment button opens the panel', 'touch Equipment button did nothing');
  const eqRect = await mp.locator('#equipment').evaluate((e) => { const r = e.getBoundingClientRect(); return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, w: innerWidth, h: innerHeight }; });
  check(eqRect.left >= -1 && eqRect.right <= eqRect.w + 1 && eqRect.top >= -1 && eqRect.bottom <= eqRect.h + 1,
    `mobile Equipment panel fits the viewport (${JSON.stringify(eqRect)})`, `mobile Equipment panel overflows (${JSON.stringify(eqRect)})`);
  await mp.screenshot({ path: resolve(shots, '12-mobile-equipment.jpg'), type: 'jpeg', quality: 70 });
  await mp.click('#eq-close'); await sleep(150);

  await openFab();
  check(await fabButton('Tab').count() === 1 && await fabButton('Tab').evaluate((e) => e.classList.contains('available')),
    'mobile Codex action uses the Tab binding', 'mobile Codex action has the wrong key');
  await fabButton('Tab').click(); await sleep(250);
  check(!(await mp.locator('#codex').getAttribute('class')).includes('hidden'),
    'touch Codex button opens the Bestiary', 'touch Codex button did nothing');
  await mp.screenshot({ path: resolve(shots, '13-mobile-codex.jpg'), type: 'jpeg', quality: 70 });
  await mp.click('#codex-close'); await sleep(150);

  await mp.click('#invbtn'); await sleep(250);
  await mp.screenshot({ path: resolve(shots, '14-mobile-inventory.jpg'), type: 'jpeg', quality: 70 });
  await mp.click('#inv-close'); await sleep(150);
  await mp.click('#gear'); await sleep(250);
  await mp.screenshot({ path: resolve(shots, '15-mobile-settings.jpg'), type: 'jpeg', quality: 70 });

  check(consoleErrors.length === 0, '0 console errors across desktop + mobile',
    `${consoleErrors.length} console error(s):\n${consoleErrors.slice(0, 8).join('\n')}`);
} catch (e) {
  bad(`QA loop threw: ${e.stack || e}`);
} finally {
  if (browser) await browser.close();
  server.kill('SIGTERM');
}

console.log(`\nQA loop: ${pass} passed, ${fail} failed. Screenshots: docs/qa-evidence/`);
process.exit(fail === 0 ? 0 : 1);
