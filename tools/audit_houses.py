#!/usr/bin/env python3
"""Walk TO the village and enter a house."""
import subprocess, time, os, json, re
from playwright.sync_api import sync_playwright

os.chdir("/home/danish1075/Documents/wasm game")
server = subprocess.Popen(
    ["python3", "-m", "http.server", "8000", "-d", "pkg"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
)
time.sleep(2)

OUT = "tools/house"
logs = []
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=False,
            args=["--enable-unsafe-webgpu", "--enable-features=Vulkan", "--ignore-gpu-blocklist"]
        )
        page = browser.new_page(viewport={"width": 1280, "height": 720})
        page.on("console", lambda msg: logs.append(msg.text))
        page.goto("http://localhost:8000/index.html", wait_until="networkidle", timeout=30000)
        time.sleep(4)
        page.click("#btn-new", timeout=5000)
        time.sleep(8)

        def get_frame():
            raw = page.evaluate("() => { try { return get_frame_dump(); } catch(e) { return '{}'; } }")
            return raw if isinstance(raw, dict) else json.loads(raw)

        def press(code, t=0.04):
            page.keyboard.down(code)
            time.sleep(t)
            page.keyboard.up(code)
            time.sleep(0.01)

        frame = get_frame()
        px, py = frame['player']['x'], frame['player']['y']
        print(f"Start: ({px:.1f},{py:.1f})")

        # Find nearest house from stats
        log = page.evaluate("() => document.getElementById('log') ? document.getElementById('log').textContent : ''")
        structs = re.findall(r'(Hh|Cb|Hu|Inn|Barn)1@\((-?\d+),(-?\d+)\)', log)
        print(f"Found {len(structs)} enterable structures")

        if not structs:
            print("No houses found!")
            browser.close()
            exit()

        # Find nearest
        nearest = min(structs, key=lambda s: (float(s[1])+0.5-px)**2 + (float(s[2])+0.5-py)**2)
        htx, hty = float(nearest[1]), float(nearest[2])
        print(f"Nearest: {nearest[0]}@({htx:.0f},{hty:.0f})")

        # Walk toward it
        target_x, target_y = htx + 0.5, hty + 0.5
        for step in range(300):
            frame = get_frame()
            px, py = frame['player']['x'], frame['player']['y']
            dx, dy = target_x - px, target_y - py
            d2 = dx*dx + dy*dy
            if d2 < 2.0:
                print(f"  Close at step {step}: ({px:.1f},{py:.1f}) d2={d2:.1f}")
                break
            if step % 50 == 0:
                print(f"  Step {step}: ({px:.1f},{py:.1f}) d2={d2:.1f}")
            if dx > 0.3: press('d')
            elif dx < -0.3: press('a')
            if dy > 0.3: press('s')
            elif dy < -0.3: press('w')

        frame = get_frame()
        px, py = frame['player']['x'], frame['player']['y']
        d2_nearest = (target_x - px)**2 + (target_y - py)**2
        print(f"\nFinal position: ({px:.1f},{py:.1f}) d2={d2_nearest:.1f}")
        page.screenshot(path=f"{OUT}_at_house.png")

        # Enter
        press("Enter", 0.2)
        time.sleep(2)
        
        # Check interior via ui_data
        ui = page.evaluate("() => { try { return JSON.parse(get_ui_data()); } catch(e) { return {}; } }")
        interior = ui.get('interior', False) if isinstance(ui, dict) else False
        print(f"Interior from ui_data: {interior}")

        # Also check frame dump
        frame2 = get_frame()
        interior2 = frame2.get('interior', False) if isinstance(frame2, dict) else False
        print(f"Interior from frame_dump: {interior2}")

        if interior or interior2:
            page.screenshot(path=f"{OUT}_inside_1.png")
            print("SUCCESS - INSIDE!")
            # Walk around inside
            for k in ['a','s','d','w','a','s']:
                press(k, 0.06)
            page.screenshot(path=f"{OUT}_inside_walk.png")
            press("Enter", 0.2)
            time.sleep(1)
            page.screenshot(path=f"{OUT}_exited.png")
            print("Exited")
        else:
            page.screenshot(path=f"{OUT}_failed.png")
            # Debug: check key_dbg
            dbg = page.evaluate("() => { try { const fd = get_frame_dump(); return JSON.stringify(fd, null, 2).substring(0, 500); } catch(e) { return e.toString(); } }")
            print(f"Frame dump sample: {dbg[:300]}")

        browser.close()
finally:
    server.terminate()
    server.wait(timeout=5)

print("\nDone!")
