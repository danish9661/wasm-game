#!/usr/bin/env python3
"""Comprehensive Playwright UI audit: screenshot every game state."""
import subprocess, time, os, json, re
from playwright.sync_api import sync_playwright

os.chdir("/home/danish1075/Documents/wasm game")
server = subprocess.Popen(
    ["python3", "-m", "http.server", "8000", "-d", "pkg"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
)
time.sleep(2)

OUT = "tools/audit"

def press(page, code, t=0.1):
    page.keyboard.down(code)
    time.sleep(t)
    page.keyboard.up(code)
    time.sleep(0.05)

logs = []
errors = []
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=False,
            args=["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--ignore-gpu-blocklist"]
        )
        page = browser.new_page(viewport={"width": 1280, "height": 720})
        page.on("console", lambda msg: (
            errors.append(msg.text) if msg.type == "error" else None,
            logs.append(f"[{msg.type}] {msg.text}")
        ))
        page.goto("http://localhost:8000/index.html", wait_until="networkidle", timeout=30000)
        time.sleep(4)

        # 1. Boot menu
        page.screenshot(path=f"{OUT}_01_bootmenu.png")
        print("01 - bootmenu")

        # 2. Click New Game
        page.click("#btn-new", timeout=5000)
        time.sleep(10)
        page.screenshot(path=f"{OUT}_02_gameplay.png")
        print("02 - gameplay (idle)")

        # 3. Open Inventory (I key)
        press(page, "KeyI")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_03_inventory.png")
        print("03 - inventory panel")
        press(page, "KeyI")
        time.sleep(0.5)

        # 4. Build mode (Q key)
        press(page, "KeyQ")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_04_build_mode.png")
        print("04 - build mode")
        press(page, "Escape")
        time.sleep(0.5)

        # 5. Help panel (? button)
        page.click("#help")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_05_help.png")
        print("05 - help panel")
        page.click("#btn-help-close")
        time.sleep(0.5)

        # 6. Settings — check settings btn
        settings_btn = page.query_selector('#settings-btn') or page.query_selector('#btn-settings')
        if settings_btn:
            settings_btn.click()
        else:
            press(page, "Escape")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_06_settings.png")
        print("06 - settings")
        press(page, "Escape")
        time.sleep(0.5)

        # 7. Weapon cycle (P key)
        press(page, "KeyP")
        time.sleep(0.5)
        page.screenshot(path=f"{OUT}_07_weapon_cycle.png")
        print("07 - weapon cycle")
        for _ in range(4):
            press(page, "KeyP")
            time.sleep(0.3)

        # 8. Move around, chop a tree (WASD + E near resource)
        for _ in range(20):
            press(page, "KeyW", 0.04)
        time.sleep(0.5)
        press(page, "KeyE")
        time.sleep(0.5)
        press(page, "KeyE")
        time.sleep(0.5)
        page.screenshot(path=f"{OUT}_08_moving.png")
        print("08 - moving/chopping")

        # 9. Codex (K key)
        press(page, "KeyK")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_09_codex.png")
        print("09 - codex")
        press(page, "KeyK")
        time.sleep(0.5)

        # 10. Build anvil (N key)
        press(page, "KeyN")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_10_build_anvil.png")
        print("10 - build anvil")
        press(page, "Escape")
        time.sleep(0.5)

        # 11. Forge weapon (M key)
        press(page, "KeyM")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_11_forge_weapon.png")
        print("11 - forge weapon")

        # 12. Eat food (C key)
        press(page, "KeyC")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_12_eat.png")
        print("12 - eat food")

        # 13. Walk to house and enter (Enter key)
        frame_raw = page.evaluate("() => { try { return JSON.parse(get_ui_data()); } catch(e) { return {}; } }")
        frame = frame_raw if isinstance(frame_raw, dict) else json.loads(frame_raw) if isinstance(frame_raw, str) else {}
        px = frame.get('px', 0)
        py = frame.get('py', 0)
        
        log = page.evaluate("() => document.getElementById('log') ? document.getElementById('log').textContent : ''")
        structs = re.findall(r'(Hh|Cb|Hu|Inn|Barn)1@\((-?\d+),(-?\d+)\)', log)
        if structs:
            nearest = min(structs, key=lambda s: (float(s[1])+0.5-px)**2 + (float(s[2])+0.5-py)**2)
            htx, hty = float(nearest[1]), float(nearest[2])
            print(f"13 - walking to {nearest[0]}@({htx:.0f},{hty:.0f}) from ({px:.1f},{py:.1f})")
            target_x, target_y = htx + 0.5, hty + 0.5
            for step in range(400):
                frame_raw = page.evaluate("() => { try { return JSON.parse(get_ui_data()); } catch(e) { return {}; } }")
                frame = frame_raw if isinstance(frame_raw, dict) else {}
                px = frame.get('px', 0)
                py = frame.get('py', 0)
                dx, dy = target_x - px, target_y - py
                d2 = dx*dx + dy*dy
                if d2 < 2.0:
                    print(f"    Close at step {step}: ({px:.1f},{py:.1f}) d2={d2:.1f}")
                    break
                if dx > 0.3: press(page, "KeyD", 0.04)
                elif dx < -0.3: press(page, "KeyA", 0.04)
                if dy > 0.3: press(page, "KeyS", 0.04)
                elif dy < -0.3: press(page, "KeyW", 0.04)
            
            page.screenshot(path=f"{OUT}_13_before_enter.png")
            press(page, "Enter", 0.2)
            time.sleep(2)
            frame_raw2 = page.evaluate("() => { try { return get_frame_dump(); } catch(e) { return '{}'; } }")
            frame2 = frame_raw2 if isinstance(frame_raw2, dict) else json.loads(frame_raw2)
            interior = frame2.get('interior', False)
            print(f"    Interior: {interior}")
            if interior:
                page.screenshot(path=f"{OUT}_13_interior.png")
                print("13 - interior ENTERED")
                # walk inside
                for k in ['KeyA','KeyS','KeyD','KeyW']:
                    press(page, k, 0.06)
                page.screenshot(path=f"{OUT}_13_interior_walk.png")
                press(page, "Enter", 0.2)
                time.sleep(1)
            else:
                print("13 - FAILED to enter house")
                page.screenshot(path=f"{OUT}_13_enter_failed.png")
        else:
            print("13 - no houses found, skipping")

        # 14. Merchant trade (O key near merchant)
        press(page, "KeyO")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_14_merchant.png")
        print("14 - merchant trade")

        # 15. Check console errors
        print(f"\n--- JS errors: {len(errors)} ---")
        for e in errors[:10]:
            print(f"  ERROR: {e}")

        # 16. UI element visibility check
        ui_check = page.evaluate("""() => {
            const checks = [];
            const hud = document.getElementById('hud');
            checks.push({id: 'hud', visible: hud && getComputedStyle(hud).display !== 'none'});
            const game = document.getElementById('game');
            const blit = document.getElementById('blit');
            checks.push({id: 'game', visible: game && getComputedStyle(game).display !== 'none'});
            checks.push({id: 'blit', visible: blit && getComputedStyle(blit).display !== 'none'});
            const mm = document.getElementById('minimap');
            checks.push({id: 'minimap', visible: mm && getComputedStyle(mm).display !== 'none'});
            const quest = document.getElementById('quest');
            checks.push({id: 'quest', visible: quest && getComputedStyle(quest).display !== 'none'});
            ['help-panel','bootmenu','mpmenu','settings','pause-overlay','codex','overlay'].forEach(id => {
                const el = document.getElementById(id);
                checks.push({id, visible: el && !el.classList.contains('hidden')});
            });
            return checks;
        }""")
        print("\n--- UI element check ---")
        for c in ui_check:
            status = "VISIBLE" if c['visible'] else "hidden"
            print(f"  {c['id']}: {status}")

        browser.close()
finally:
    server.terminate()
    server.wait(timeout=5)

print("\nDone! Screenshots in tools/audit_*.png")
