#!/usr/bin/env python3
"""Assemble baked walk-frame renders into a labelled contact sheet.

    python3 scripts/assemble_pose_sheet.py FRAME_DIR OUT.png [FPS]

FRAME_DIR holds pose_walk_NN.png renders from scripts/bake_pose_frames.py.
Only the standard library + Pillow are used (system Python, not Blender's).
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

CELL = 320
LABEL = 28
COLS = 4


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    frames = sorted(Path(argv[0]).glob("pose_walk_*.png"))
    if not frames:
        print("FAIL: no pose_walk_*.png frames in %s" % argv[0])
        return 1
    out = Path(argv[1])
    fps = float(argv[2]) if len(argv) > 2 else 0.0

    rows = (len(frames) + COLS - 1) // COLS
    sheet = Image.new("RGB", (COLS * CELL, rows * (CELL + LABEL)), (28, 31, 36))
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 17)
    except OSError:
        font = ImageFont.load_default()

    for i, path in enumerate(frames):
        col, row = i % COLS, i // COLS
        x, y = col * CELL, row * (CELL + LABEL)
        with Image.open(path) as im:
            sheet.paste(im.convert("RGB").resize((CELL, CELL)), (x, y + LABEL))
        t = i / fps if fps else float(i)
        text = "f%02d  %s" % (i, ("%.2fs" % t) if fps else "")
        draw.rectangle((x, y, x + CELL, y + LABEL), fill=(43, 47, 54))
        draw.text((x + 10, y + 5), text, fill=(236, 238, 240), font=font)
        draw.rectangle((x, y, x + CELL - 1, y + CELL + LABEL - 1),
                       outline=(90, 96, 104), width=1)

    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print("[sheet] %d frames -> %s (%dx%d)" % (len(frames), out, *sheet.size))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
