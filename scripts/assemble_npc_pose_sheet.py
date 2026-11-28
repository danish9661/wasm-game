#!/usr/bin/env python3
"""Assemble baked NPC pose-frame renders into a labelled contact sheet.

    python3 scripts/assemble_npc_pose_sheet.py FRAME_DIR OUT.png

FRAME_DIR holds npcpose_<clip>_<NN>.png renders from
scripts/bake_npc_pose_frames.py.  Standard library + Pillow (system Python).
"""

from __future__ import annotations

import re
import sys
from collections import OrderedDict
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

CELL = 256
LABEL = 32
COLS = 6
PATTERN = re.compile(r"^(npcpose_.+)_(\d+)\.png$")


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    frames = sorted(Path(argv[0]).glob("npcpose_*.png"))
    if not frames:
        print("FAIL: no npcpose_*.png frames in %s" % argv[0])
        return 1
    out = Path(argv[1])

    groups: OrderedDict[str, list[Path]] = OrderedDict()
    for path in frames:
        match = PATTERN.match(path.name)
        if not match:
            continue
        groups.setdefault(match.group(1), []).append(path)

    rows = 0
    for paths in groups.values():
        rows += (len(paths) + COLS - 1) // COLS
    sheet = Image.new("RGB", (COLS * CELL, rows * (CELL + LABEL)), (28, 31, 36))
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 15)
    except OSError:
        font = ImageFont.load_default()

    row = 0
    for key, paths in groups.items():
        for i, path in enumerate(paths):
            col = i % COLS
            line = row + i // COLS
            x, y = col * CELL, line * (CELL + LABEL)
            with Image.open(path) as im:
                sheet.paste(im.convert("RGB").resize((CELL, CELL)), (x, y + LABEL))
            draw.rectangle((x, y, x + CELL, y + LABEL), fill=(43, 47, 54))
            draw.text((x + 8, y + 6), "%s  f%02d" % (key, i),
                      fill=(236, 238, 240), font=font)
            draw.rectangle((x, y, x + CELL - 1, y + CELL + LABEL - 1),
                           outline=(90, 96, 104), width=1)
        row += (len(paths) + COLS - 1) // COLS

    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print("[npc-sheet] %d clips, %d frames -> %s (%dx%d)"
          % (len(groups), len(frames), out, *sheet.size))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
