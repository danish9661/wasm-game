// Headless "visual engine" for Starfall.
//
// The agent (me) is not multimodal, so it cannot open a PNG. This tool turns a
// rendered frame into TEXT in two independent ways and runs automated layout /
// animation checks against the real, running game:
//
//   1. `get_frame_dump()`  -> a JSON scene graph (every sprite in screen space,
//      with sizes, colors, walk/attack state). Rendered as ASCII art so the
//      agent can read the *intended* layout.
//   2. The actual screenshot PNG -> decoded here (no deps, zlib only) and
//      downsampled to an ASCII luminance map, so the agent can see the *real*
//      pixels the player would see.
//
// Checks performed:
//   - Houses (H/C/U) render wider AND taller than the player sprite (P).
//   - No house sprite's box contains the player's center (player not "inside"
//     a building on screen).
//   - Houses are spaced apart (min world distance between houses > threshold).
//   - Moving (set_analog) changes the player's screen position / walk state.
//   - do_attack() drives the player's swing_t (attack animation) > 0.
//
// Run:  node tools/visualize.js

const puppeteer = require('puppeteer-core');
const zlib = require('zlib');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const PORT = 8000;
const PKG = path.resolve(__dirname, '..', 'pkg');

// ---------- minimal dependency-free PNG decoder (8-bit, no interlace) -------
function decodePNG(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error('not a PNG');
  let off = 8;
  let w = 0, h = 0, bitDepth = 0, colorType = 0;
  const idat = [];
  while (off < buf.length) {
    const len = buf.readUInt32BE(off); off += 4;
    const type = buf.toString('ascii', off, off + 4); off += 4;
    const data = buf.slice(off, off + len); off += len; off += 4; // skip crc
    if (type === 'IHDR') {
      w = data.readUInt32BE(0); h = data.readUInt32BE(4);
      bitDepth = data[8]; colorType = data[9];
    } else if (type === 'IDAT') {
      idat.push(data);
    } else if (type === 'IEND') break;
  }
  if (bitDepth !== 8) throw new Error('only 8-bit PNGs supported');
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 0;
  if (!channels) throw new Error('only RGB/RGBA PNGs supported');
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const stride = w * channels;
  const out = Buffer.alloc(h * stride);
  let pos = 0;
  const paeth = (a, b, c) => {
    const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
    return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
  };
  for (let y = 0; y < h; y++) {
    const ft = raw[pos++];
    for (let x = 0; x < stride; x++) {
      const cur = raw[pos++];
      const a = x >= channels ? out[y * stride + x - channels] : 0;
      const b = y > 0 ? out[(y - 1) * stride + x] : 0;
      const c = x >= channels && y > 0 ? out[(y - 1) * stride + x - channels] : 0;
      let v;
      switch (ft) {
        case 0: v = cur; break;
        case 1: v = cur + a; break;
        case 2: v = cur + b; break;
        case 3: v = cur + ((a + b) >> 1); break;
        case 4: v = cur + paeth(a, b, c); break;
        default: throw new Error('bad filter ' + ft);
      }
      out[y * stride + x] = v & 0xff;
    }
  }
  return { w, h, channels, data: out };
}

function pngToAscii(png, cols = 100, rows = 38) {
  const ramp = ' .:-=+*#%@';
  const cellW = png.w / cols, cellH = png.h / rows;
  let out = '';
  for (let r = 0; r < rows; r++) {
    let line = '';
    for (let c = 0; c < cols; c++) {
      let sum = 0, n = 0;
      const x0 = Math.floor(c * cellW), x1 = Math.floor((c + 1) * cellW);
      const y0 = Math.floor(r * cellH), y1 = Math.floor((r + 1) * cellH);
      for (let y = y0; y < y1; y++) {
        for (let x = x0; x < x1; x++) {
          const i = (y * png.w + x) * png.channels;
          const lum = 0.299 * png.data[i] + 0.587 * png.data[i + 1] + 0.114 * png.data[i + 2];
          sum += lum; n++;
        }
      }
      const avg = n ? sum / n : 0;
      line += ramp[Math.min(ramp.length - 1, Math.floor((avg / 255) * (ramp.length - 1)))];
    }
    out += line + '\n';
  }
  return out;
}

// ---------- frame-dump -> ASCII scene graph ---------------------------------
function dumpToAscii(dump, cols = 110, rows = 44) {
  const sprites = dump.sprites || [];
  if (!sprites.length) return '(no sprites)';
  // Zoom into the player's neighbourhood so nearby buildings are readable
  // instead of being shrunk to a blob by far-away sprites. 1 tile ~ 32px (x)
  // and ~16px (y) in iso; cap the view at ~14 tiles from the player.
  const EXT_X = 14 * 32;
  const EXT_Y = 14 * 16;
  const visible = sprites.filter(s =>
    Math.abs(s.sx) <= EXT_X && Math.abs(s.sy) <= EXT_Y);
  const scale = Math.min((cols / 2 - 2) / EXT_X, (rows / 2 - 2) / EXT_Y);
  const grid = Array.from({ length: rows }, () => Array(cols).fill(' '));
  // depth-sort so nearer (larger sy) sprites draw on top
  const order = visible.map((s, i) => i).sort((a, b) => visible[a].sy - visible[b].sy);
  for (const i of order) {
    const s = visible[i];
    const cx = Math.round(cols / 2 + s.sx * scale);
    const cy = Math.round(rows / 2 + s.sy * scale);
    const hw = Math.max(0.4, s.hw * scale);
    const hh = Math.max(0.4, s.hh * scale);
    const ch = (s.label || '?').charAt(0);
    for (let dy = -Math.floor(hh); dy <= Math.ceil(hh); dy++) {
      for (let dx = -Math.floor(hw); dx <= Math.ceil(hw); dx++) {
        const gx = cx + dx, gy = cy + dy;
        if (gx >= 0 && gx < cols && gy >= 0 && gy < rows) grid[gy][gx] = ch;
      }
    }
  }
  // Always draw the player on top so it's never hidden behind buildings.
  if (dump.player) {
    const cx = Math.round(cols / 2 + dump.player.sx * scale);
    const cy = Math.round(rows / 2 + dump.player.sy * scale);
    for (let dy = -1; dy <= 1; dy++)
      for (let dx = -1; dx <= 1; dx++) {
        const gx = cx + dx, gy = cy + dy;
        if (gx >= 0 && gx < cols && gy >= 0 && gy < rows) grid[gy][gx] = 'P';
      }
  }
  return grid.map(r => r.join('')).join('\n');
}

// ---------- checks ----------------------------------------------------------
function runChecks(dump, dump2, dump3) {
  const results = [];
  const ok = (name, cond, detail) => results.push({ name, pass: !!cond, detail });

  const sprites = dump.sprites || [];
  const player = sprites.find(s => s.label === 'P');
  const houses = sprites.filter(s => ['H', 'C', 'U'].includes(s.label));
  ok('houses present', houses.length > 0, `found ${houses.length} houses`);
  if (player && houses.length) {
    const bigger = houses.every(h => h.hw > player.hw && h.hh > player.hh);
    ok('houses bigger than player', bigger,
      `player ${player.hw.toFixed(0)}x${player.hh.toFixed(0)} vs houses ` +
      houses.map(h => `${h.label} ${h.hw.toFixed(0)}x${h.hh.toFixed(0)}`).join(', '));
  }
  // player not visually inside any house box
  if (player && houses.length) {
    const inside = houses.some(h =>
      Math.abs(h.sx - player.sx) < h.hw && Math.abs(h.sy - player.sy) < h.hh);
    ok('player not inside a house', !inside, inside ? 'overlap detected' : 'clear');
    // player should not be embedded right next to a building either
    const minPH = Math.min(...houses.map(h =>
      Math.hypot(h.x - player.x, h.y - player.y)));
    ok('player not embedded in a house', minPH > 1.0,
      `nearest house ${minPH.toFixed(2)} tiles`);
  }
  // spacing: min world distance between houses
  if (houses.length >= 2) {
    let minD = Infinity;
    for (let i = 0; i < houses.length; i++)
      for (let j = i + 1; j < houses.length; j++) {
        const dx = houses[i].x - houses[j].x, dy = houses[i].y - houses[j].y;
        minD = Math.min(minD, Math.hypot(dx, dy));
      }
    ok('houses spaced apart', minD > 1.5, `min center distance ${minD.toFixed(2)} tiles`);
  }
  // movement: player world position changed between idle and moved frames
  if (dump2 && dump2.player) {
    const d = Math.hypot(dump2.player.x - dump.player.x, dump2.player.y - dump.player.y);
    ok('movement changes position', d > 0.5, `world delta ${d.toFixed(2)} tiles`);
  }
  // attack animation: swing_t (player.attack) went above 0 during a swing
  if (dump3 && dump3.player) {
    ok('attack drives swing_t', dump3.player.attack > 0.01,
      `swing_t=${dump3.player.attack.toFixed(2)}`);
  }
  return results;
}

// ---------- main -------------------------------------------------------------
(async () => {
  const server = spawn('python3', ['-m', 'http.server', String(PORT), '-d', PKG],
    { stdio: 'ignore' });
  await new Promise(r => setTimeout(r, 800));

  const browser = await puppeteer.launch({
    executablePath: '/usr/bin/google-chrome',
    headless: 'new',
    args: ['--no-sandbox', '--enable-unsafe-webgpu', '--use-gl=angle',
      '--use-angle=swiftshader', '--ignore-gpu-blocklist', '--window-size=1000,700',
      '--disable-background-timer-throttling', '--disable-renderer-backgrounding',
      '--disable-backgrounding-occluded-windows'],
  });
  const page = await browser.newPage();
  await page.setViewport({ width: 1000, height: 700 });
  const errors = [];
  page.on('pageerror', e => errors.push('PAGEERROR: ' + e.message));
  page.on('requestfailed', r => { if (!r.url().startsWith('data:')) errors.push('REQFAIL: ' + r.url()); });

  await page.goto(`http://localhost:${PORT}/index.html`, { waitUntil: 'load', timeout: 30000 });
  // wait for app + let the world settle. The page runs its own rAF loop, so we
  // just wait; manual window.step() would collide with that loop.
  // Wait until the WASM module is initialized (the App exists and produces a
  // non-empty frame). Calling exported fns before this throws "wasm is undefined".
  await page.waitForFunction(() => {
    try {
      const d = JSON.parse(window.get_frame_dump());
      return d && Array.isArray(d.sprites) && d.sprites.length > 0;
    } catch (e) { return false; }
  }, { timeout: 20000 });
  // Start a new game programmatically so the world + player exist and update().
  await page.evaluate(() => window.new_game());
  const sleep = (ms) => new Promise(r => setTimeout(r, ms));
  await sleep(2000);
  // Wait until the new game has a player (so movement/attack can be tested).
  await page.waitForFunction(() => {
    try {
      const d = JSON.parse(window.get_frame_dump());
      return d && d.player && Array.isArray(d.sprites) && d.sprites.length > 0;
    } catch (e) { return false; }
  }, { timeout: 20000 });

  const dump1 = await page.evaluate(() => JSON.parse(window.get_frame_dump()));
  const shot1 = await page.screenshot({ type: 'png' });

  // Drive the simulation manually (headless rAF is unreliable) so movement and
  // the attack animation advance deterministically.
  const stepN = (n, dt = 0.03) =>
    page.evaluate(([n, dt]) => { for (let i = 0; i < n; i++) window.step(dt); }, [n, dt]);

  // movement: hold "right" on the analog stick and step the world forward.
  await page.evaluate(() => window.set_analog(1, 0));
  await stepN(80, 0.03);
  await page.evaluate(() => window.set_analog(0, 0));
  const dump2 = await page.evaluate(() => JSON.parse(window.get_frame_dump()));

  // attack animation: trigger a swing, step until swing_t ramps up.
  let dump3 = dump2;
  await page.evaluate(() => window.do_attack());
  for (let i = 0; i < 40; i++) {
    await stepN(1, 0.016);
    const d = await page.evaluate(() => JSON.parse(window.get_frame_dump()));
    if (d.player && d.player.attack > 0.01) { dump3 = d; break; }
    dump3 = d;
  }

  // interior: walk up to the nearest house (it blocks movement, so we stop
  // adjacent) and press Enter to go inside; verify the interior scene loads.
  const houses = dump1.sprites.filter(s => ['H', 'C', 'U'].includes(s.label));
  let interiorDump = null;
  if (houses.length && dump1.player) {
    const target = houses
      .map(s => ({ s, d: Math.hypot(s.x - dump1.player.x, s.y - dump1.player.y) }))
      .sort((a, b) => a.d - b.d)[0].s;
    const dx = target.x - dump1.player.x, dy = target.y - dump1.player.y;
    const len = Math.hypot(dx, dy) || 1;
    await page.evaluate(([x, y]) => window.set_analog(x, y), [dx / len, dy / len]);
    await stepN(180, 0.03);
    await page.evaluate(() => window.set_analog(0, 0));
    await page.keyboard.press('Enter');
    await stepN(12, 0.03);
    interiorDump = await page.evaluate(() => JSON.parse(window.get_frame_dump()));
  }

  const checks = runChecks(dump1, dump2, dump3);

  // style histogram: what's actually in the scene (catches missing/duplicate art)
  const hist = {};
  for (const s of dump1.sprites) hist[s.label] = (hist[s.label] || 0) + 1;
  const histStr = Object.entries(hist).sort((a, b) => b[1] - a[1])
    .map(([k, v]) => `${k}:${v}`).join('  ');

  // interior check
  if (interiorDump) {
    const floor = interiorDump.sprites.some(s => s.label === '.');
    checks.push({
      name: 'entering a house loads interior',
      pass: interiorDump.interior === true && floor,
      detail: `interior=${interiorDump.interior} floorTiles=${floor}`,
    });
  } else {
    checks.push({ name: 'entering a house loads interior', pass: false, detail: 'no house found' });
  }

  console.log('\n===== FRAME DUMP (intended scene, ASCII) =====');
  console.log(dumpToAscii(dump1));
  console.log('\n===== SCREENSHOT (real pixels, ASCII) =====');
  console.log(pngToAscii(decodePNG(Buffer.from(shot1)), 100, 38));

  console.log('\n===== STYLE HISTOGRAM =====');
  console.log(histStr);

  console.log('\n===== CHECKS =====');
  let allPass = true;
  for (const c of checks) {
    console.log(`${c.pass ? 'PASS' : 'FAIL'}  ${c.name}  (${c.detail})`);
    if (!c.pass) allPass = false;
  }
  if (errors.length) {
    console.log('\n--- page errors ---');
    errors.forEach(e => console.log(e));
  }
  console.log(`\nRESULT: ${allPass && !errors.length ? 'OK' : 'PROBLEMS'}`);

  await browser.close();
  server.kill('SIGKILL');
  process.exit(allPass && !errors.length ? 0 : 1);
})().catch(e => { console.error('VISUALIZE FAILED', e); process.exit(1); });
