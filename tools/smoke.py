"""Headless-Chromium smoke test: loads the built app, enters the machine, plays, screenshots.
usage: smoke.py <public_dir> <out_dir>"""
import sys, threading, http.server, functools, os, time
from playwright.sync_api import sync_playwright

public, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=public)
handler.log_message = lambda *a, **k: None
srv = http.server.ThreadingHTTPServer(("127.0.0.1", 8123), handler)
threading.Thread(target=srv.serve_forever, daemon=True).start()

with sync_playwright() as p:
    b = p.chromium.launch(executable_path="/usr/bin/chromium", args=[
        "--autoplay-policy=no-user-gesture-required", "--no-sandbox", "--enable-unsafe-swiftshader"])
    pg = b.new_page(viewport={"width": 1400, "height": 1500})
    logs = []
    pg.on("console", lambda m: logs.append(f"[{m.type}] {m.text}"))
    pg.on("pageerror", lambda e: logs.append(f"[pageerror] {e}"))
    pg.goto("http://127.0.0.1:8123/")
    pg.wait_for_selector(".overlay", timeout=20000)
    pg.screenshot(path=f"{out}/1-overlay.png")
    pg.click("button.enter")
    time.sleep(0.5)
    pg.keyboard.press("z"); pg.keyboard.press("c")
    time.sleep(2.5)
    pg.screenshot(path=f"{out}/2-playing.png")
    print("overlay gone:", pg.locator(".overlay").count() == 0)
    # shift-click lock the first knob, then RND, check lock persisted & values changed
    knob = pg.locator(".knob").first
    before = pg.locator(".knob .kv").all_inner_texts()
    knob.click(modifiers=["Shift"])
    print("locked class:", "locked" in (knob.get_attribute("class") or ""))
    pg.click("button.rnd")
    time.sleep(1.2)
    after = pg.locator(".knob .kv").all_inner_texts()
    print("first knob unchanged by RND:", before[0] == after[0], "| others changed:", sum(a != b for a, b in zip(before[1:], after[1:])), "of", len(before) - 1)
    pg.screenshot(path=f"{out}/3-rnd.png")
    print("\n".join(logs[:30]))
    b.close()
