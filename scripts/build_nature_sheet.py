#!/usr/bin/env python3
"""Compose rendered gameplay-scale cells into labelled nature QA contact sheets.

Usage:
    python3 scripts/build_nature_sheet.py \
        --cells /tmp/opencode/nature_qa/cells_before \
        --out /tmp/opencode/nature_qa/before \
        --title "Nature / Biome props - BEFORE"

Reads model triangle counts and heights from assets/models/metrics.json so the
label under each cell carries hard numbers next to the picture.
"""
import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
CELL_W, CELL_H = 440, 380
LABEL_H = 46
HEADER_H = 92
COLS = 5
ROWS_PER_SHEET = 5
PAD = 10


def font(size, bold=False):
    candidates = (
        ["/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"] if bold else []
    ) + [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf",
    ]
    for c in candidates:
        if Path(c).is_file():
            return ImageFont.truetype(c, size)
    return ImageFont.load_default()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cells", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--family", type=Path,
                    default=ROOT / "scripts" / "blender" / "nature_family.json")
    ap.add_argument("--title", default="Nature / Biome props - visual QA")
    ap.add_argument("--subtitle", default="")
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)

    fam = json.loads(args.family.read_text(encoding="utf-8"))
    names = fam["models"]
    metrics = {}
    mp = ROOT / "assets" / "models" / "metrics.json"
    if mp.is_file():
        metrics = {m["name"]: m for m in json.loads(mp.read_text())["models"]}

    cells = [n for n in names if (args.cells / (n + ".png")).is_file()]
    per_sheet = COLS * ROWS_PER_SHEET
    sheets = math.ceil(len(cells) / per_sheet) or 1
    f_title = font(30, bold=True)
    f_sub = font(17)
    f_name = font(21, bold=True)
    f_meta = font(15)

    for s in range(sheets):
        chunk = cells[s * per_sheet:(s + 1) * per_sheet]
        rows = math.ceil(len(chunk) / COLS)
        W = COLS * (CELL_W + PAD) + PAD
        H = HEADER_H + rows * (CELL_H + LABEL_H + PAD) + PAD
        sheet = Image.new("RGB", (W, H), (24, 27, 30))
        d = ImageDraw.Draw(sheet)
        d.text((PAD + 4, 14), args.title, font=f_title, fill=(240, 242, 244))
        legend = ("1u = 1 tile  |  grey figure = 1.55u player  |  "
                  "orthographic 45 yaw / 30 pitch  |  true gameplay scale")
        d.text((PAD + 4, 54), legend, font=f_sub, fill=(160, 170, 178))
        if args.subtitle:
            d.text((PAD + 4, 74), args.subtitle, font=f_sub, fill=(200, 168, 96))
        for i, name in enumerate(chunk):
            r, c = divmod(i, COLS)
            x = PAD + c * (CELL_W + PAD)
            y = HEADER_H + PAD + r * (CELL_H + LABEL_H + PAD)
            img = Image.open(args.cells / (name + ".png")).convert("RGB")
            sheet.paste(img, (x, y))
            d.rectangle((x, y, x + CELL_W - 1, y + CELL_H - 1),
                        outline=(70, 78, 84), width=2)
            m = metrics.get(name, {})
            meta = "%s tris   %.2fu" % (m.get("triangles", "?"), m.get("height", 0))
            d.rectangle((x, y + CELL_H, x + CELL_W, y + CELL_H + LABEL_H),
                        fill=(38, 43, 47))
            d.text((x + 10, y + CELL_H + 4), name, font=f_name, fill=(238, 240, 242))
            d.text((x + 10, y + CELL_H + 28), meta, font=f_meta, fill=(150, 190, 160))
        out = args.out / ("nature_sheet_%02d.png" % (s + 1))
        sheet.save(out, optimize=True)
        print("SHEET", out, sheet.size, "cells", len(chunk))
    print("SHEETS_DONE", sheets)


if __name__ == "__main__":
    main()
