#!/usr/bin/env python3
"""Assemble baked prop-frame renders into a labelled contact sheet.

    python3 scripts/assemble_prop_sheet.py FRAME_DIR OUT.png

FRAME_DIR holds animprop_<clip>_<NN>.png renders from scripts/bake_prop_poses.py.
One row per clip, one column per frame; the shared pivot/size is easy to check
by eye because the static anchor never drifts between columns.  Standard
library + Pillow (system Python, not Blender's).
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

CELL = 200
LABEL = 24
COLS = 8


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    frames = sorted(Path(argv[0]).glob("animprop_*.png"))
    if not frames:
        print("FAIL: no animprop_*.png frames in %s" % argv[0])
        return 1
    out = Path(argv[1])

    groups: dict[str, list[Path]] = defaultdict(list)
    for path in frames:
        m = re.match(r"animprop_(.+)_(\d+)\.png$", path.name)
        if m:
            groups[m.group(1)].append(path)
    clips = sorted(groups)

    sheet = Image.new("RGB", (COLS * CELL, len(clips) * (CELL + LABEL)),
                      (28, 31, 36))
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 15)
    except OSError:
        font = ImageFont.load_default()

    for row, clip in enumerate(clips):
        paths = sorted(groups[clip])
        y = row * (CELL + LABEL)
        for col, path in enumerate(paths):
            x = col * CELL
            with Image.open(path) as im:
                sheet.paste(im.convert("RGB").resize((CELL, CELL)), (x, y + LABEL))
            draw.rectangle((x, y, x + CELL, y + LABEL), fill=(43, 47, 54))
            draw.text((x + 8, y + 4), "%s  f%02d" % (clip, col),
                      fill=(236, 238, 240), font=font)
            draw.rectangle((x, y, x + CELL - 1, y + CELL + LABEL - 1),
                           outline=(90, 96, 104), width=1)
        for col in range(len(paths), COLS):
            x = col * CELL
            draw.rectangle((x, y, x + CELL, y + CELL + LABEL), fill=(24, 27, 31))

    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print("[sheet] %d clips, %d frames -> %s (%dx%d)"
          % (len(clips), len(frames), out, *sheet.size))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
