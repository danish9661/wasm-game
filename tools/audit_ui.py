#!/usr/bin/env python3
"""Comprehensive Playwright UI audit: screenshot every game state."""
import subprocess, time, os, json
from playwright.sync_api import sync_playwright

os.chdir("/home/danish1075/Documents/wasm game")
server = subprocess.Popen(
    ["python3", "-m", "http.server", "8000", "-d", "pkg"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
)
time.sleep(2)

OUT = "tools/audit"

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
        page.keyboard.press("i")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_03_inventory.png")
        print("03 - inventory panel")
        page.keyboard.press("i")  # close
        time.sleep(0.5)

        # 4. Build mode (Q key)
        page.keyboard.press("q")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_04_build_mode.png")
        print("04 - build mode")
        page.keyboard.press("q")  # exit
        time.sleep(0.5)

        # 5. Help panel (? button)
        page.click("#help")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_05_help.png")
        print("05 - help panel")
        page.click("#btn-help-close")
        time.sleep(0.5)

        # 6. Settings (Esc)
        page.keyboard.press("Escape")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_06_settings.png")
        print("06 - settings")
        page.keyboard.press("Escape")
        time.sleep(0.5)

        # 7. Weapon cycle (P key)
        page.keyboard.press("p")
        time.sleep(0.5)
        page.screenshot(path=f"{OUT}_07_weapon_cycle.png")
        print("07 - weapon cycle")
        page.keyboard.press("p")
        time.sleep(0.3)
        page.keyboard.press("p")
        time.sleep(0.3)
        page.keyboard.press("p")
        time.sleep(0.3)
        page.keyboard.press("p")
        time.sleep(0.3)

        # 8. Move around, chop a tree (WASD + E near resource)
        for _ in range(20):
            page.keyboard.press("w")
            time.sleep(0.05)
        time.sleep(0.5)
        page.keyboard.press("e")
        time.sleep(0.5)
        page.keyboard.press("e")
        time.sleep(0.5)
        page.screenshot(path=f"{OUT}_08_moving.png")
        print("08 - moving/chopping")

        # 9. Codex (L key)
        page.keyboard.press("l")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_09_codex.png")
        print("09 - codex")
        page.keyboard.press("l")
        time.sleep(0.5)

        # 10. Build anvil (N key)
        page.keyboard.press("n")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_10_build_anvil.png")
        print("10 - build anvil prompt")
        page.keyboard.press("Escape")
        time.sleep(0.5)

        # 11. Forge weapon (M key near anvil if nearby)
        page.keyboard.press("m")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_11_forge_weapon.png")
        print("11 - forge weapon")

        # 12. Eat food (C key)
        page.keyboard.press("c")
        time.sleep(1)
        page.screenshot(path=f"{OUT}_12_eat.png")
        print("12 - eat food")

        # 13. Check console errors
        print(f"\n--- JS errors: {len(errors)} ---")
        for e in errors[:10]:
            print(f"  ERROR: {e}")

        # 14. Check if any UI elements are invisible or misplaced
        ui_check = page.evaluate("""() => {
            const checks = [];
            // Check HUD visibility
            const hud = document.getElementById('hud');
            checks.push({id: 'hud', visible: hud && getComputedStyle(hud).display !== 'none'});
            // Check canvas exists
            const game = document.getElementById('game');
            const blit = document.getElementById('blit');
            checks.push({id: 'game', visible: game && getComputedStyle(game).display !== 'none'});
            checks.push({id: 'blit', visible: blit && getComputedStyle(blit).display !== 'none'});
            // Check minimap
            const mm = document.getElementById('minimap');
            checks.push({id: 'minimap', visible: mm && getComputedStyle(mm).display !== 'none'});
            // Check quest banner
            const quest = document.getElementById('quest');
            checks.push({id: 'quest', visible: quest && getComputedStyle(quest).display !== 'none'});
            // Check overlays are all hidden
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
