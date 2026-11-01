#!/usr/bin/env python3
"""Draw NGLB placeholder tray icons (replaces upstream's hand artwork).
Usage: make-tray-icons.py src-tauri/resources   (needs Pillow)"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

S, K = 64, 4  # final size, supersampling
WHITE, BLACK, COLOR = (255, 255, 255, 255), (0, 0, 0, 255), (60, 120, 220, 255)


def mic(state, fg):
    im = Image.new("RGBA", (S * K, S * K), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    u = lambda v: int(v * K)
    if state == "idle":
        d.rounded_rectangle((u(23), u(6), u(41), u(36)), radius=u(9), outline=fg, width=u(4))
    else:
        d.rounded_rectangle((u(23), u(6), u(41), u(36)), radius=u(9), fill=fg)
    d.arc((u(15), u(18), u(49), u(50)), 0, 180, fill=fg, width=u(4))
    d.line((u(32), u(50), u(32), u(58)), fill=fg, width=u(4))
    d.line((u(24), u(58), u(40), u(58)), fill=fg, width=u(4))
    if state == "transcribing":
        for x in (46, 52, 58):
            d.ellipse((u(x - 2), u(30), u(x + 2), u(34)), fill=fg)
    if state == "warning":
        d.rounded_rectangle((u(50), u(34), u(54), u(48)), radius=u(2), fill=fg)
        d.ellipse((u(49), u(51), u(55), u(57)), fill=fg)
    return im.resize((S, S), Image.LANCZOS)


out = Path(sys.argv[1])
for name, state in [("idle", "idle"), ("idle_warning", "warning"), ("recording", "recording"), ("transcribing", "transcribing")]:
    mic(state, WHITE).save(out / f"tray_{name}.png")
    mic(state, BLACK).save(out / f"tray_{name}_dark.png")
mic("idle", COLOR).save(out / "handy.png")
mic("warning", COLOR).save(out / "handy_warning.png")
mic("recording", COLOR).save(out / "recording.png")
mic("transcribing", COLOR).save(out / "transcribing.png")
