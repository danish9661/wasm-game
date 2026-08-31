#!/usr/bin/env python3
"""Start server, launch game in HEADED Chrome, take screenshot."""
import subprocess, time, os
from playwright.sync_api import sync_playwright

os.chdir("/home/danish1075/Documents/wasm game")
server = subprocess.Popen(
    ["python3", "-m", "http.server", "8000", "-d", "pkg"],
    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
)
time.sleep(2)

logs = []
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=False,
            args=[
                "--enable-unsafe-webgpu",
                "--enable-features=Vulkan",
                "--ignore-gpu-blocklist",
            ]
        )
        page = browser.new_page(viewport={"width": 1280, "height": 720})
        page.on("console", lambda msg: logs.append(f"[{msg.type}] {msg.text}"))
        page.goto("http://localhost:8000/index.html", wait_until="networkidle", timeout=30000)
        time.sleep(4)

        # The bootmenu shows on load. Click "New Game" button (proper onclick handler).
        page.click("#btn-new", timeout=5000)
        time.sleep(10)

        page.screenshot(path="tools/screenshot_headed.png", full_page=False)
        print("Screenshot saved")

        try:
            stats = page.evaluate("() => document.getElementById('log') ? document.getElementById('log').textContent : 'no log'")
            print("Stats:", stats[:400])
        except:
            pass

        browser.close()
finally:
    server.terminate()
    server.wait(timeout=5)

print("\n--- Console logs (last 10) ---")
for log in logs[-10:]:
    print(log)
