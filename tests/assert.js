// CDP assertion runner for the E2E test.
// Connects to headless Chrome, loads the game page, asserts init + loop +
// rendering, and monitors Chrome's RSS (kills the test with a clear failure
// if RAM exceeds the cap). Always exits, never leaves Chrome running.
import { readFileSync } from 'node:fs';

const DEBUG_PORT = process.argv[2] || '9333';
const TIMEOUT_S = Number(process.argv[3] || 45);
const CHROME_PID = Number(process.env.E2E_CHROME_PID || 0);
const MAX_RSS_MB = Number(process.env.E2E_MAX_RSS_MB || 1200);

let passed = 0;
let failed = 0;
const ok = (m) => { passed++; console.log(`\x1b[1;32mPASS\x1b[0m ${m}`); };
const bad = (m) => { failed++; console.log(`\x1b[1;31mFAIL\x1b[0m ${m}`); };

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const deadline = Date.now() + TIMEOUT_S * 1000;
const timeLeft = () => Math.max(0, deadline - Date.now());

// --- Chrome RSS watchdog -------------------------------------------------
const rssMB = () => {
  try {
    const status = readFileSync(`/proc/${CHROME_PID}/status`, 'utf8');
    const m = status.match(/VmRSS:\s+(\d+)\s+kB/);
    return m ? Math.round(Number(m[1]) / 1024) : 0;
  } catch { return 0; }
};
const rssTimer = setInterval(() => {
  const rss = rssMB();
  if (rss > MAX_RSS_MB) {
    console.log(`\x1b[1;31mFAIL\x1b[0m chrome RSS ${rss}MB > cap ${MAX_RSS_MB}MB — killing test`);
    process.exit(1);
  }
}, 500);

// --- CDP plumbing ---------------------------------------------------------
let ws;
const pending = new Map();
const events = [];
const send = (method, params = {}) => new Promise((res) => {
  const id = ++msgId;
  pending.set(id, res);
  ws.send(JSON.stringify({ id, method, params }));
});

// Headless Chrome's Page.captureScreenshot does NOT composite WebGPU canvas
// content -- it returns the grey page background instead. So sprite-visibility
// checks read the actual presented canvas pixels via toDataURL (the real frame
// the user sees). Returns base64 PNG data string.
async function captureCanvas() {
  const r = await send('Runtime.evaluate', {
    expression: "document.querySelector('#game').toDataURL('image/png').split(',')[1]",
    returnByValue: true,
  });
  const data = r?.result?.result?.value;
  if (!data) throw new Error('canvas capture failed');
  return data;
}

async function connect() {
  let list = null;
  for (let i = 0; i < 10 && !list; i++) {
    try {
      list = await fetch(`http://127.0.0.1:${DEBUG_PORT}/json`).then(r => r.json());
    } catch { await sleep(500); }
  }
  if (!list) throw new Error('chrome /json unreachable');
  const page = list.find(t => t.type === 'page' && t.url.includes('localhost'));
  if (!page) throw new Error('game page target not found');
  ws = new WebSocket(page.webSocketDebuggerUrl);
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
    else if (m.method) events.push(m);
  };
  await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
  return page;
}

let msgId = 0;
const evaluate = async (expr) => {
  const r = await send('Runtime.evaluate', { expression: expr, returnByValue: true });
  return r.result?.result?.value;
};

// --- wait helpers ---------------------------------------------------------
async function waitFor(fn, what) {
  while (timeLeft() > 0) {
    try { if (await fn()) return true; } catch {}
    await sleep(300);
  }
  const dump = await evaluate(`({log: document.getElementById('log')?.textContent, state: document.readyState, url: location.href})`).catch(() => null);
  console.log(`\x1b[1;31mFAIL\x1b[0m timeout waiting for: ${what}`);
  if (dump) console.log(`       page state: ${JSON.stringify(dump)}`);
  return false;
}

// --- tests ----------------------------------------------------------------
try {
  await connect();
  await send('Runtime.enable');
  await send('Page.enable');
  // deterministic start: reload the page so wasm init happens while we listen
  await send('Page.reload', { ignoreCache: true });
  await sleep(500);

  // 1. wasm + WebGPU initialized (from console events; DOM poll has a race)
  const inited = await waitFor(
    () => Promise.resolve(events.some(e =>
      e.method === 'Runtime.consoleAPICalled' &&
      e.params.args?.some(a => String(a.value ?? '').includes('webgpu initialized'))
    ) || evaluate(`document.getElementById('log').textContent`).then(t => String(t).includes('webgpu initialized'))),
    'wasm + WebGPU init'
  );
  inited ? ok('wasm module loaded, WebGPU initialized') : bad('WebGPU init');

  // 2. canvas resized (not the 300x150 default)
  const size = await evaluate(`({w: document.getElementById('game').width, h: document.getElementById('game').height})`);
  (size && size.w > 800) ? ok(`canvas resized to ${size.w}x${size.h}`) : bad(`canvas not resized: ${JSON.stringify(size)}`);

  // 3. game loop running (fps line appears)
  const loop = await waitFor(
    () => evaluate(`document.getElementById('log').textContent`).then(t => /fps=\d+/.test(String(t))),
    'game loop (fps log)'
  );
  loop ? ok('game loop running') : bad('game loop never ticked');

  // 4. tiles generated
  const q = await send('Runtime.evaluate', { expression: `(document.getElementById('log').textContent.match(/quads=(\\d+)/)||[])[1]`, returnByValue: true });
  const quads = q.result?.result?.value;
  if (q.result?.exceptionDetails) console.log('QUADS EXC:', JSON.stringify(q.result.exceptionDetails.exception?.description).slice(0, 300));
  (quads && Number(quads) > 100) ? ok(`tile mesh generated (${quads} quads)`) : bad(`no quads: ${quads} :: log=${JSON.stringify(await evaluate('document.getElementById(\'log\').textContent'))}`);

  // 5. pixels actually drawn (GPU readback of the offscreen target)
  const mReady = await waitFor(() => evaluate(`import('./wasm_game.js').then(m => { window.__m = m; return true; })`), 'wasm module handle');
  await evaluate(`window.__m.trigger_readback()`);
  let stats = '';
  const rbDone = await waitFor(async () => {
    const s = await evaluate(`window.__m.readback_stats()`);
    if (s && !s.includes('pending') && !s.includes('queued') && !s.includes('map error') && !s.includes('no app')) {
      stats = s;
      return true;
    }
    return false;
  }, 'readback completion');
  let px = { distinct: 0, nonBackground: 0 };
  if (stats) {
    const m = stats.match(/distinct=(\d+) nonbg=([\d.]+)%/);
    px = m ? { distinct: Number(m[1]), nonBackground: Number(m[2]) / 100 } : { distinct: 0, nonBackground: 0 };
    console.log(`       readback: ${stats}`);
  }
  (mReady && rbDone && px.distinct > 15 && px.nonBackground > 0.01)
    ? ok(`renderer drew pixels (${px.distinct} colors, ${(px.nonBackground * 100).toFixed(1)}% non-background)`)
    : bad(`screen looks blank: ${JSON.stringify(px)}`);

  // 6. player sprite rendered (warm orange quad visible on screen).
  //    We verify from the actual presented canvas frame (toDataURL PNG bytes),
  //    since headless Page.captureScreenshot does not composite WebGPU canvases.
  const { decodePng, countOrange } = await import('./png_analyze.mjs');
  const shot1 = { result: { data: await captureCanvas() } };
  if (shot1.result?.data) {
    const { width, height, pixels } = decodePng(Buffer.from(shot1.result.data, 'base64'));
    const center = pixels[((Math.floor(height / 2) * width + Math.floor(width / 2)) * 4)];
    const orangeCount = countOrange(width, height, pixels);
    console.log(`       shot center r=${center} orange_px=${orangeCount}`);
    (orangeCount > 20)
      ? ok(`player sprite on screen (${orangeCount} sampled pixels)`)
      : bad(`player sprite not visible: orange_px=${orangeCount} center_r=${center}`);
  } else {
    bad('no screenshot captured');
  }

  // 7. WASD movement: hold each direction until one moves the player
  const playerPos = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const m = String(s).match(/player=\((-?[\d.]+),(-?[\d.]+)\)/);
    return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
  };
  const before = await playerPos();
  let moved = false;
  let dirName = '';
  for (const [code, key, vk] of [['KeyD', 'd', 68], ['KeyS', 's', 83], ['KeyA', 'a', 65], ['KeyW', 'w', 87]]) {
    if (moved) break;
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key, code, windowsVirtualKeyCode: vk });
    await sleep(1200);
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key, code, windowsVirtualKeyCode: vk });
    const after = await playerPos();
    if (after && before && (after.x !== before.x || after.y !== before.y)) {
      moved = true;
      dirName = key;
      console.log(`       player moved with "${key}": (${before.x.toFixed(2)},${before.y.toFixed(2)}) -> (${after.x.toFixed(2)},${after.y.toFixed(2)})`);
    }
  }
  moved
    ? ok(`player movement works (held "${dirName}", position changed)`)
    : bad(`player never moved (was at ${JSON.stringify(before)})`);

  // 8. player still rendered after moving (camera followed and re-centered)
  await sleep(800);
  const shot2 = { result: { data: await captureCanvas() } };
  if (shot2.result?.data) {
    const { width, height, pixels } = decodePng(Buffer.from(shot2.result.data, 'base64'));
    const orangeCount = countOrange(width, height, pixels);
    (orangeCount > 20)
      ? ok(`player sprite visible after movement (${orangeCount} sampled pixels)`)
      : bad(`player sprite lost after movement: orange_px=${orangeCount}`);
  } else {
    bad('no screenshot after movement');
  }

  // 9-11. gathering: hunt for a node, render-check it, and farm resources.
  //   Seed 1337: grass near spawn carries bushes (1 chop = 1 wood), so first
  //   walk SE until a node is in range, verify it renders, then chop until we
  //   have 5 wood; then walk SE to the stone range at (~33,-32) for 1 stone.
  const near = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const m = String(s).match(/near=(\w+)@\((-?\d+),(-?\d+)\)/);
    return m ? { kind: m[1], tx: Number(m[2]), ty: Number(m[3]) } : null;
  };
  const invOf = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const m = String(s).match(/inv=\(w(\d+),s(\d+),f(\d+)\)/);
    return m ? { wood: Number(m[1]), stone: Number(m[2]), food: Number(m[3]) } : null;
  };
  const pressE = async () => {
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'e', code: 'KeyE', windowsVirtualKeyCode: 69 });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'e', code: 'KeyE', windowsVirtualKeyCode: 69 });
    await sleep(250);
  };
  const pressKey = async (code, vk) => {
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
    await sleep(250);
  };
  const walk = async (code, vk, ms) => {
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
    await sleep(ms);
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
    await sleep(150);
  };
  const walkUntilNear = async (maxBursts) => {
    for (let i = 0; i < maxBursts; i++) {
      const n = await near();
      if (n) return n;
      await walk('KeyA', 65, 1000);
    }
    return null;
  };

  let foundNode = await walkUntilNear(3);
  if (!foundNode) { foundNode = await walkUntilNear(12); }
  foundNode
    ? ok(`resource node found nearby (${foundNode.kind} at ${foundNode.tx},${foundNode.ty})`)
    : bad('no resource node found nearby');

  if (foundNode) {
    const shot3 = { result: { data: await captureCanvas() } };
    if (shot3.result?.data) {
      const { width, height, pixels } = decodePng(Buffer.from(shot3.result.data, 'base64'));
      let nodePx = 0;
      const isWood = foundNode.kind !== 'Rock';
      for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
        const i = (y * width + x) * 4;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        if (isWood) {
          if (r < 120 && g > 80 && g < 140 && b < 70 && g > r) nodePx++;
        } else {
          if (r > 120 && b > r + 10 && Math.abs(r - g) < 12) nodePx++;
        }
      }
      (nodePx > 4)
        ? ok(`${foundNode.kind} sprite rendered (${nodePx} sampled pixels)`)
        : bad(`${foundNode.kind} sprite not visible: node_px=${nodePx}`);
    } else {
      bad('no screenshot for node check');
    }
  }

  // farm 5 wood from bushes/trees, then 1 stone from the rock range
  for (let i = 0; i < 30; i++) {
    const inv = await invOf();
    if (inv && inv.wood >= 5 && inv.stone >= 1) break;
    if (inv && inv.wood < 5) {
      const n = await near();
      if (!n) { await walk('KeyA', 65, 1000); continue; }
      if (n.kind !== 'Rock') {
        await pressE();
        if ((await invOf()).wood > inv.wood) continue; // bush done, find next
      } else {
        await walk('KeyA', 65, 1000); // skip rocks while farming wood
      }
    } else if (inv) {
      // have 5 wood, need stone: keep walking SE to the rock range
      const n = await near();
      if (n && n.kind === 'Rock') {
        await pressE();
      } else {
        await walk('KeyA', 65, 1000);
      }
    }
  }
  const invFarmed = await invOf();
  (invFarmed && invFarmed.wood >= 5 && invFarmed.stone >= 1)
    ? ok(`gathering works (inventory w${invFarmed.wood} s${invFarmed.stone})`)
    : bad(`gathering shortfall: ${JSON.stringify(invFarmed)}`);

  // story beat 1: the harvest completes "Gather 5 wood and 1 stone"
  const questS = async () => {
    const s = String(await evaluate(`window.__m.get_stats()`));
    return Number((s.match(/quest=S(\d)/) || [])[1] || -1);
  };
  (await questS()) === 1
    ? ok('quest advanced to S1 (gathering done)')
    : bad(`quest not S1 after gathering (S${await questS()})`);

  // 12. build a wall (V) at the player's tile: costs 2 wood.
  //     clear any live bush underfoot first, or the build is refused
  await pressE();
  const structBefore = Number((String(await evaluate(`window.__m.get_stats()`)).match(/structures=(\d+)/) || [])[1] || 0);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'v', code: 'KeyV', windowsVirtualKeyCode: 86 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'v', code: 'KeyV', windowsVirtualKeyCode: 86 });
  await sleep(600);
  const structAfter = Number((String(await evaluate(`window.__m.get_stats()`)).match(/structures=(\d+)/) || [])[1] || 0);
  (structAfter > structBefore)
    ? ok(`wall built (structures ${structBefore} -> ${structAfter})`)
    : bad(`wall not built (structures ${structBefore} -> ${structAfter})`);

  // 13. the wall renders (tan) in the screenshot; sand variation (188,168,109)
  //     is the closest lookalike, so also require b>115 and r-b<45
  if (structAfter > structBefore) {
    const shot4 = { result: { data: await captureCanvas() } };
    if (shot4.result?.data) {
      const { width, height, pixels } = decodePng(Buffer.from(shot4.result.data, 'base64'));
      let wallPx = 0;
      for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
        const i = (y * width + x) * 4;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        if (r > 150 && g > 130 && g < 170 && b > 115 && r - b < 45 && r > g && g > b) wallPx++;
      }
      (wallPx > 10)
        ? ok(`wall sprite rendered (${wallPx} sampled pixels)`)
        : bad(`wall sprite not visible: wall_px=${wallPx}`);
    } else {
      bad('no screenshot for wall check');
    }
  }

  // 14. build a campfire (F): step aside first (the wall blocks its tile),
  //     and clear any live bush on the new tile before building
  const posBefore = String(await evaluate(`window.__m.get_stats()`)).match(/player=\((-?[\d.]+),(-?[\d.]+)\)/);
  await walk('KeyD', 68, 700);
  const posAfter = String(await evaluate(`window.__m.get_stats()`)).match(/player=\((-?[\d.]+),(-?[\d.]+)\)/);
  console.log(`       campfire step: (${posBefore?.[1]},${posBefore?.[2]}) -> (${posAfter?.[1]},${posAfter?.[2]})`);
  await pressE();
  const dbg = await evaluate(`window.__m.get_stats()`);
  console.log(`       campfire setup :: ${dbg}`);
  await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'f', code: 'KeyF', windowsVirtualKeyCode: 70 });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'f', code: 'KeyF', windowsVirtualKeyCode: 70 });
  await sleep(600);
  const structCamp = Number((String(await evaluate(`window.__m.get_stats()`)).match(/structures=(\d+)/) || [])[1] || 0);
  (structCamp > structAfter)
    ? ok(`campfire built (structures ${structAfter} -> ${structCamp})`)
    : bad(`campfire not built (structures ${structAfter} -> ${structCamp})`);

  // story beat 2: wall + campfire complete "Build a wall and a campfire"
  (await questS()) === 2
    ? ok('quest advanced to S2 (shelter built)')
    : bad(`quest not S2 after campfire (S${await questS()})`);

  // 15. the campfire renders (hot orange) in the screenshot
  if (structCamp > structAfter) {
    const shot5 = { result: { data: await captureCanvas() } };
    if (shot5.result?.data) {
      const { width, height, pixels } = decodePng(Buffer.from(shot5.result.data, 'base64'));
      let firePx = 0;
      for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
        const i = (y * width + x) * 4;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        if (r > 200 && g < 90 && b < 50) firePx++;
      }
      (firePx > 4)
        ? ok(`campfire sprite rendered (${firePx} sampled pixels)`)
        : bad(`campfire sprite not visible: fire_px=${firePx}`);
    } else {
      bad('no screenshot for campfire check');
    }
  }

  // 16-20. combat: find the nearest slime (seed 1337 spawns one at (-17,-2),
//   ~19 tiles west), fight it, and eat the meat it drops.
  const mob = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const m = String(s).match(/mob=(\w+)@\((-?\d+),(-?\d+)\)/);
    return m ? { kind: m[1], tx: Number(m[2]), ty: Number(m[3]) } : null;
  };
  const statsOf = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const hp = Number((String(s).match(/hp=([\d.]+)/) || [])[1] || 0);
    const hunger = Number((String(s).match(/hunger=([\d.]+)/) || [])[1] || 0);
    const food = Number((String(s).match(/inv=\(w\d+,s\d+,f(\d+)\)/) || [])[1] || 0);
    return { hp, hunger, food, raw: String(s) };
  };
  // walk to the slime at (-17,-2): first west (zigzag cancels y drift) until
  // x <= -16, then south with short s/d zigzags (y advances, x oscillates
  // within ~1 tile of the swamp) checking for the mob after every segment.
  const posOf = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const m = String(s).match(/player=\((-?[\d.]+),(-?[\d.]+)\)/);
    return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
  };
  let foundMob = null;
  let pos = null;
  for (let i = 0; i < 10 && !foundMob; i++) {
    pos = await posOf();
    if (pos && pos.x > -16) {
      await walk('KeyW', 87, 1500);
      await walk('KeyD', 68, 1500);
    } else {
      await walk('KeyS', 83, 800);
      foundMob = await mob();
      if (foundMob) break;
      await walk('KeyD', 68, 800);
    }
    foundMob = await mob();
  }
  foundMob
    ? ok(`slime found nearby (${foundMob.kind} at ${foundMob.tx},${foundMob.ty})`)
    : bad(`no slime found nearby (last pos ${JSON.stringify(pos)})`);

  if (foundMob) {
    const shot6 = { result: { data: await captureCanvas() } };
    if (shot6.result?.data) {
      const { width, height, pixels } = decodePng(Buffer.from(shot6.result.data, 'base64'));
      let slimePx = 0;
      for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
        const i = (y * width + x) * 4;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        if (g > r + 90 && g > 150 && b < 130) slimePx++;
      }
      (slimePx > 4)
        ? ok(`slime sprite rendered (${slimePx} sampled pixels)`)
        : bad(`slime sprite not visible: slime_px=${slimePx}`);
    } else {
      bad('no screenshot for slime check');
    }
  }

  // fight the swarm: aim at the nearest slime (the mob tile is in stats),
// swing (J) until food drops, then shoot (K) for a second kill.
  const aimAt = async (tx, ty) => {
    const p = await posOf();
    if (!p) return;
    const dx = Math.sign(tx - p.x), dy = Math.sign(ty - p.y);
    // only diagonal movement exists: pick the closest key direction
    let code;
    if (dx === -1 && dy === -1) code = 'KeyW';
    else if (dx === 1 && dy === 1) code = 'KeyS';
    else if (dx === 1 && dy === -1) code = 'KeyA';
    else if (dx === -1 && dy === 1) code = 'KeyD';
    else if (dx === -1 && dy === 0) code = 'KeyW';
    else if (dx === 1 && dy === 0) code = 'KeyS';
    else if (dx === 0 && dy === -1) code = 'KeyA';
    else code = 'KeyD';
    await walk(code, { KeyW: 87, KeyS: 83, KeyA: 65, KeyD: 68 }[code], 120);
  };
  await pressKey('KeyJ', 74);
  await sleep(400);
  await pressKey('KeyJ', 74);
  await sleep(1100);
  const stFight = await statsOf();
  (stFight.hp < 100)
    ? ok(`player took slime damage (hp ${stFight.hp.toFixed(0)})`)
    : bad(`player never got hit (hp ${stFight.hp.toFixed(0)})`);

  // melee kills: aim + swing until the first slime drops food
  for (let i = 0; i < 5; i++) {
    const m = await mob();
    if (!m) break;
    await aimAt(m.tx, m.ty);
    await pressKey('KeyJ', 74);
    if ((await statsOf()).food >= 1) break;
  }
  // arrow kills: the pack sits at its spawn tiles just outside aggro range,
  // so aim at the nearest pack member (pack= lists all positions), walk
  // toward it and shoot until a second slime drops food (2 arrows = 16 dmg
  // kill a fresh 12 hp slime).
  const packNear = async () => {
    const s = await evaluate(`window.__m.get_stats()`);
    const ppos = await posOf();
    if (!ppos) return null;
    let best = null;
    let bd = 1e9;
    for (const m of String(s).matchAll(/\((-?[\d.]+),(-?[\d.]+)\)/g)) {
      const ex = Number(m[1]), ey = Number(m[2]);
      const d = Math.max(Math.abs(ex - ppos.x), Math.abs(ey - ppos.y));
      if (d < 0.9 || d >= bd) continue;
      bd = d;
      best = [Math.floor(ex), Math.floor(ey)];
    }
    return best;
  };
  const arrowDbg = [];
  for (let i = 0; i < 6; i++) {
    const m = (await mob()) || (await packNear());
    arrowDbg.push(`${i}:${JSON.stringify(m)}:${JSON.stringify(await posOf())}`);
    if (!m) break;
    await aimAt(m.tx ?? m[0], m.ty ?? m[1]);
    await pressKey('KeyK', 75);
    await sleep(300);
    if ((await statsOf()).food >= 2) break;
  }
  const stKill = await statsOf();
  (stKill.food >= 2)
    ? ok(`slimes defeated (melee + arrow): ${stKill.food} food dropped`)
    : bad(`slimes survived: food=${stKill.food} :: ${stKill.raw} :: dbg=[${arrowDbg.join(' ')}]`);

  // story beat 3: the first slime kill clears the ruins roadblock
  (await questS()) === 3
    ? ok('quest advanced to S3 (first slime defeated)')
    : bad(`quest not S3 after kills (S${await questS()})`);

  // 21. eat the meat with C: food is consumed (2 -> 1)
  await pressKey('KeyC', 67);
  await sleep(400);
  const st2 = await statsOf();
  (st2.food === stKill.food - 1)
    ? ok(`ate the meat (food ${stKill.food} -> ${st2.food})`)
    : bad(`eating failed: food=${st2.food}`);

  // 22. the ruins POI: read the chest tile from stats and walk there
  //     (goto keeps stepping diagonally toward the target; on a blocked
  //     tile it nudges sideways once before resuming)
  const ruinsTile = async () => {
    const s = String(await evaluate(`window.__m.get_stats()`));
    const m = s.match(/ruins=\((-?\d+),(-?\d+)\)/);
    return m ? { tx: Number(m[1]), ty: Number(m[2]) } : null;
  };
  const goto = async (tx, ty, maxIters = 40) => {
    for (let i = 0; i < maxIters; i++) {
      const p = await posOf();
      if (!p) return false;
      const dx = tx + 0.5 - p.x, dy = ty + 0.5 - p.y;
      if (Math.max(Math.abs(dx), Math.abs(dy)) < 2.0) return true;
      let code;
      if (dx < -0.4 && dy < -0.4) code = 'KeyW';
      else if (dx > 0.4 && dy > 0.4) code = 'KeyS';
      else if (dx > 0.4 && dy < -0.4) code = 'KeyA';
      else if (dx < -0.4 && dy > 0.4) code = 'KeyD';
      else if (dx < -0.4) code = 'KeyW';
      else if (dx > 0.4) code = 'KeyS';
      else if (dy < -0.4) code = 'KeyA';
      else code = 'KeyD';
      const before = { x: p.x, y: p.y };
      const vk = { KeyW: 87, KeyS: 83, KeyA: 65, KeyD: 68 }[code];
      await walk(code, vk, 800);
      const after = await posOf();
      if (after && Math.abs(after.x - before.x) < 0.2 && Math.abs(after.y - before.y) < 0.2) {
        const nudge = code === 'KeyW' || code === 'KeyS' ? 'KeyA' : 'KeyW';
        await walk(nudge, { KeyW: 87, KeyS: 83, KeyA: 65, KeyD: 68 }[nudge], 500);
      }
    }
    return false;
  };
  const ruins = await ruinsTile();
  if (!ruins) {
    bad('ruins tile missing from stats');
  } else {
    const walked = await goto(ruins.tx, ruins.ty, 50);
    const stNear = await statsOf();
    const chestFlag = Number((String(stNear.raw).match(/chest=(\d)/) || [])[1] || 0);
    (walked)
      ? ok(`reached the ruins at (${ruins.tx},${ruins.ty}) (${stNear.raw.match(/quest=S\d/)?.[0]})`)
      : bad(`could not reach ruins at (${ruins.tx},${ruins.ty}): ${stNear.raw}`);

    // 23. the chest renders (gold) once in view
    const shotR = { result: { data: await captureCanvas() } };
    if (shotR.result?.data) {
      const { width, height, pixels } = decodePng(Buffer.from(shotR.result.data, 'base64'));
      let chestPx = 0;
      for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
        const i = (y * width + x) * 4;
        const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
        if (r > 170 && g > 120 && g < 210 && b < 110 && r - g > 40) chestPx++;
      }
      (chestPx > 5)
        ? ok(`chest sprite rendered (${chestPx} sampled pixels)`)
        : bad(`chest sprite not visible: chest_px=${chestPx}`);
    }

    // 24. story beat 4: proximity to the ruins flips the quest
    (await questS()) === 4
      ? ok('quest advanced to S4 (ruins found)')
      : bad(`quest not S4 near ruins (S${await questS()})`);

    // 25. open the chest with E: loot + quest S5
    const invBefore = await invOf();
    await pressE();
    await sleep(500);
    const stOpen = await statsOf();
    const chestAfter = Number((String(stOpen.raw).match(/chest=(\d)/) || [])[1] || 0);
    const invAfter = await invOf();
    const gained = invAfter && invBefore
      ? `w+${invAfter.wood - invBefore.wood} s+${invAfter.stone - invBefore.stone} f+${invAfter.food - invBefore.food}`
      : '?';
    (chestAfter === 1 && (await questS()) === 5)
      ? ok(`chest opened (chest=1, quest=S5, loot ${gained})`)
      : bad(`chest did not open: chest=${chestAfter} quest=S${await questS()} loot ${gained} :: ${stOpen.raw}`);

    // --- Chapter 2 finale: Forest Warden -> Crown Fragment -> Reforging Altar ---
    const getStat = async (re) => {
      const s = await evaluate(`window.__m.get_stats()`);
      const m = String(s).match(re);
      return m ? Number(m[1]) : -1;
    };
    const getAltarTile = async () => {
      const s = await evaluate(`window.__m.get_stats()`);
      const m = String(s).match(/altartile=\((-?\d+),(-?\d+)\)/);
      return m ? { tx: Number(m[1]), ty: Number(m[2]) } : null;
    };
    const tap = async (code, vk) => {
      await send('Input.dispatchKeyEvent', { type: 'keyDown', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
      await send('Input.dispatchKeyEvent', { type: 'keyUp', key: code[3].toLowerCase(), code, windowsVirtualKeyCode: vk });
      await sleep(70);
    };

    // 26. Forest Warden spawns once the chest is looted
    const bossUp = await waitFor(async () => (await getStat(/boss=(\d)/)) === 1, 'Warden spawned', 20000);
    bossUp ? ok('Forest Warden spawned at the ruins') : bad('Warden never spawned');

    // 27. defeat the Warden (melee + arrows) -> drops the Crown Fragment
    let frag = 0, swings = 0;
    while (swings++ < 80) {
      frag = await getStat(/frag=(\d)/);
      if (frag >= 1) break;
      await tap('KeyJ', 74);
      await sleep(90);
      await tap('KeyK', 75);
      await sleep(110);
    }
    const stKill = await statsOf();
    (frag >= 1)
      ? ok(`defeated the Warden, recovered Crown Fragment (frag=${frag}, hp ${stKill.hp.toFixed(0)})`)
      : bad(`Warden not defeated: frag=${frag} hp=${stKill.hp.toFixed(0)} :: ${stKill.raw}`);
    (await questS()) >= 6
      ? ok(`quest advanced to S${await questS()} (fragment recovered — ready to reforge)`)
      : bad(`quest not progressed after boss (S${await questS()})`);

    // 28. Reforging Altar rises at the waking place
    const altarUp = await waitFor(async () => (await getStat(/altar=(\d)/)) === 1, 'altar placed', 20000);
    altarUp ? ok('Reforging Altar rose at the waking place') : bad('altar never appeared');
    const altarTile = await getAltarTile();
    if (altarTile) {
      const reachedAltar = await goto(altarTile.tx, altarTile.ty, 120);
      const atAltar = reachedAltar && (await waitFor(async () => (await getStat(/nearaltar=(\d)/)) === 1, 'at altar', 25000));
      atAltar
        ? ok(`reached the Reforging Altar at (${altarTile.tx},${altarTile.ty})`)
        : bad(`could not reach altar (${altarTile.tx},${altarTile.ty}) :: ${(await statsOf()).raw}`);
    } else {
      bad('altartile missing from stats');
    }

    // 29. reforge the Crown -> campaign complete + New Game+
    await pressE(); // arm the reforge prompt
    await sleep(450);
    await pressKey('KeyY', 89); // choose Reign (reforge 0)
    await sleep(600);
    const stEnd = await statsOf();
    const endWord = String((String(stEnd.raw).match(/ending=(\w+)/) || [])[1] || 'none');
    const ng = await getStat(/ng=(\d)/);
    (endWord === 'reign' && ng >= 1)
      ? ok(`reforged the Crown — Reign ending, New Game+ ${ng} (quest reset to S${await questS()})`)
      : bad(`reforge failed: ending=${endWord} ng=${ng} quest=S${await questS()} :: ${stEnd.raw}`);
  }

  // 26. no JS exceptions / wasm panics
  const errors = events.filter(e =>
    e.method === 'Runtime.exceptionThrown' ||
    (e.method === 'Runtime.consoleAPICalled' && ['error', 'assert'].includes(e.params.type))
  );
  errors.length === 0 ? ok('no console errors or exceptions') : bad(`errors: ${JSON.stringify(errors[0])}`);

} catch (e) {
  bad(`CDP failure: ${e.message}`);
}

clearInterval(rssTimer);
console.log(`\nE2E assertions: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);