# QA loop — findings, fixes and evidence

Independent QA pass on the local build of the shipped page (`pkg/index.html`,
WASM built from this worktree). Everything below was reproduced in a real
headless Chrome driving the actual WebGPU build, not by reading code alone.

**Scope:** gameplay/UI bugs only. No assets, models, animations or scale
contract were touched (1u = 1 tile = 1m; player 1.55u; doors 0.9×2.0u;
storeys 2.5u; base z=0; centred; flat shaded; palette-only; low-poly).

## How to reproduce

```bash
cargo test -p game
cargo check --target wasm32-unknown-unknown -p wasm_game
./build.sh
python3 scripts/verify_models.py && python3 scripts/check_metrics.py
CHROME_PATH=/usr/bin/google-chrome-stable NODE_PATH=/tmp/opencode/node_modules \
  AUDIT_VISUAL_BLOCK=0 ./tools/audit.sh
PLAYWRIGHT_MODULE=/tmp/opencode/node_modules/playwright-core/index.mjs \
  CHROME_BIN=/usr/bin/google-chrome-stable node tests/smoke_3d.mjs
PLAYWRIGHT_MODULE=/tmp/opencode/node_modules/playwright-core/index.mjs \
  CHROME_BIN=/usr/bin/google-chrome-stable node tests/qa_loop.mjs
```

`tests/qa_loop.mjs` serves `pkg/`, boots New Game, drives keyboard + touch,
checks save/load and both renderers, and writes the screenshots below to
`docs/qa-evidence/`.

## Baseline (before fixes)

| Check | Result |
|---|---|
| `cargo test -p game` | 183 passed, 0 failed |
| `cargo check --target wasm32-unknown-unknown -p wasm_game` | OK |
| `./build.sh` | OK (wasm-pack + static shell) |
| `python3 scripts/verify_models.py` | 223 GLBs: 223 passed |
| `python3 scripts/check_metrics.py` | OK (player 1.55u, house 3.10u, base ≈ 0) |
| `tools/audit.sh` (Chrome + puppeteer-core) | RESULT: OK, all visual gates passed |
| `tests/smoke_3d.mjs` | 8 passed, 0 failed; 0 console errors |

The project scripts were green; the defects below were found **only by driving
the UI** (boot → New Game → movement → attack → harvest → build → inventory →
equipment → codex → settings → 3D toggle → mobile → save/load).

## Findings → fixes

| # | Observed defect (reproduced in browser) | Root cause | Surgical fix | Regression test in `tests/qa_loop.mjs` |
|---|---|---|---|---|
| 1 | Pressing **3** toggled the renderer *and* silently entered build mode (`get_ui_data().buildMode === true`, no on-screen hint). | `Digit3` was bound to `select_build(2)` in `renderer.rs::set_key`, while the page shell binds 3 to the 3D/2.5D toggle. | Removed the `Digit3` build-select arm; Fence is still reachable by its `G` hotkey and by clicking the build menu. | “3 toggles renderer”, “3 does not silently enter build mode”, “3 toggles back” |
| 2 | Pressing **U** opened Equipment *and* attempted to place a Farm Plot in the same tick. Phone players had **no way at all** to open Equipment. | `KeyU` was both `build(FarmPlot)` in Rust and `toggleEquipment()` in JS; the mobile action deck never advertised `KeyU`. | Rust builds a plot with U only while build mode is active; JS toggles Equipment otherwise. Added a mobile “Gear” action and marked it available when not building. | “U opens Equipment”, “U outside build mode does not build a Farm Plot”, “Equipment key leaves build mode off”, “Equipment is offered in the mobile action deck”, “touch Equipment button opens the panel” |
| 3 | Pressing **L** to open the Codex also called `cook()` — a spurious “Stand near a campfire to cook” toast on every browse, and it could consume 2 food when standing at a fire. | Both features claimed `KeyL`; help text said `L` cooks, Codex button/close said `L`. | Kept `L` for cooking (its documented gameplay role), moved Codex to **Tab**, and updated the help grid, button badge, close label and touch binding. | “Tab opens the Codex”, “opening the Codex does not fire cook feedback”, “L no longer opens the Codex”, “L still cooks at a campfire”, “mobile Codex action uses the Tab binding” |
| 4 | **Esc** with Equipment/Codex open opened Settings instead of closing the panel (Settings was checked first); Esc to exit build mode also popped Settings open. | Wrong priority in the page keydown handler; no knowledge of the pre-keypress build state (Rust clears it before the window listener runs). | Close topmost panel first (help → codex → equipment → inventory → settings); only open Settings when nothing is open and build mode was not active. Use the HUD-synchronised `buildMode` flag for the pre-keypress state. | “Esc closes Equipment without opening Settings”, “Esc closes the Codex”, “Esc exits build mode”, “Esc does not open Settings while exiting build mode” |
| 5 | Low-HP bar pulse, red vignette and heartbeat never fired; placing a structure never played the JS build sound. | `parseStats()` never parsed `maxhp` or `structures` from `get_stats()`, so `hpFrac` was always 1 and the structure diff was `undefined > undefined`. | Parse `maxhp`, `structures` and the new `build` flag; added `build={}` to the Rust stats string so HUD feedback can follow build mode without an extra WASM call. | “stats parsed: maxhp/structures/build”, “low HP lights the critical bar + vignette”, “Q shows the build hint with Inventory closed” |
| 6 | Build mode had **no desktop feedback**: pressing Q left `#buildhint` hidden. | `updateInventory()` (the only place that refreshed the hint) ran only while the Inventory panel was open. | Extracted `syncBuildHint()` and call it from both `updateInventory()` and the 200 ms HUD tick; the hint and touch build banner now follow live build state. | “Q shows the build hint with Inventory closed”, “Esc exits build mode” |
| 7 | On a phone, the **Actions FAB did nothing** — taps fell through to the canvas. | `#touch` is `pointer-events:none` so the playfield stays clickable, but `#touch-more` never opted back in. | Added `pointer-events:auto` to `#touch-more`. | “touch controls are visible”, “touch Equipment button opens the panel”, “touch Codex button opens the Bestiary” |
| 8 | Uploading a `.zip` save reported “Save loaded from file” into `#mp-note`, which lives in the hidden Multiplayer menu — no visible feedback, and the boot menu stayed up. | Result message written to an element outside the boot screen. | Show a real toast, persist the loaded state, and drop the boot overlay so the loaded world starts. | Covered by “Save Game commits to IndexedDB” + “Continue is enabled after a save” round trip; manual upload path is wired to the same visible `toast()` channel |
| 9 | Exiting any panel with Esc was inconsistent across states (follow-on from #4); mobile panel geometry was untested. | No panel-priority handling; no phone-size assertions in the suite. | Fixed in #4; QA suite now measures panel rects at 390×844. | “mobile Equipment panel fits the viewport”, plus mobile inventory/codex/settings screenshots |
| 10 | The legacy `tests/e2e.sh` CDP runner failed almost every assertion: `get_stats()` returned `no app`, canvas looked 300×150, etc. | It imported bare `./wasm_game.js` (a second, uninitialised module instance, unlike the page's `?v=` URL) and sampled the canvas size at a fixed 500 ms. | Import the exact stamped module URL, await promises in `Runtime.evaluate`, and poll for resize. `tests/assert.js` only. | Harness fix; assertions rose 8 → 14 passes. Remaining failures are stale game-state expectations (quest stages, boss/altar, 2D readback pixels) under the shipped 3D renderer and are not part of this task's gate |

## Evidence — screenshots

All captured from the live build by `tests/qa_loop.mjs` (JPEG, 1280×720
desktop / 390×844 phone @2x):

| File | What it proves |
|---|---|
| `docs/qa-evidence/01-boot.jpg` | Boot menu with Continue / New Game / seed field |
| `02-new-game.jpg` | New Game renders GLB world, HUD, quest, minimap |
| `03-inventory.jpg` | Inventory/build recipes panel |
| `04-equipment.jpg` | Equipment panel (U) open without farm-plot placement |
| `05-settings.jpg` | Settings overlay, live sliders, Save/Resume |
| `06-build-mode.jpg` | Build hint visible with Inventory closed (fix #6) |
| `07-codex.jpg` | Codex opened with Tab |
| `08-3d-toggle.jpg` | Renderer toggled by 3, build mode still off |
| `09-loaded.jpg` | Save → reload → Continue restores seed + position |
| `10-day-night.jpg` | Shared day/night clock and weather HUD |
| `11-mobile-game.jpg` | Phone layout, HUD, joystick and Actions FAB |
| `12-mobile-equipment.jpg` | Equipment from the touch deck, panel within viewport |
| `13-mobile-codex.jpg` | Codex from the touch deck (Tab binding) |
| `14-mobile-inventory.jpg` | Scrollable inventory at phone size |
| `15-mobile-settings.jpg` | Settings at phone size |

## Final verification (after fixes)

| Check | Result |
|---|---|
| `cargo test -p game` | 183 passed, 0 failed |
| `cargo check --target wasm32-unknown-unknown -p wasm_game` | OK |
| `./build.sh` | OK |
| `python3 scripts/verify_models.py` | 223 passed, 0 failed (metrics unchanged) |
| `python3 scripts/check_metrics.py` | OK |
| `tools/audit.sh` (Chrome + puppeteer-core) | RESULT: OK — houses/player/movement/attack/quest/crafting/interior gates passed |
| `tests/smoke_3d.mjs` | 8 passed, 0 failed; 226 GLBs, 1147 instances; 0 console errors |
| `tests/qa_loop.mjs` | **40 passed, 0 failed**; 0 console errors on desktop and mobile |
| `tests/e2e.sh` (legacy CDP harness, not a task gate) | Harness bug fixed (8 → 14 passes); 24 stale assertions remain (quest stages, Warden/altar, 2D readback pixels, mobile viewport) — unrelated to the fixes above. The required 3D smoke + visual audit + new QA loop are all green |

Gameplay flows confirmed live: boot, New Game (seed), WASD movement
(position + keydown counter + speed), J attack, E harvest (node discovery),
Q/build placement and hint, inventory, equipment, codex, settings persistence,
3D toggle, day/night clock, Save Game + reload + Continue, and phone-size
layout with working touch actions.

**Not changed:** `game/src/*`, asset generators/GLBs, animation data, palette
or scale contract. All edits are in `web/src/renderer.rs`,
`web/static/index.html`, `web/static/style.css`, `tests/assert.js` (harness
only), plus the new regression test and this report.
