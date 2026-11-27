#!/usr/bin/env python3
"""Compose headless-Blender cell renders into labeled QA contact sheets.

Each cell was rendered at a fixed gameplay camera (1u = 1 tile) with a 1.55 u
human silhouette. This script only adds labels and a 60 px legibility inset.

Usage:
    python3 scripts/make_family_sheet.py --cells /tmp/opencode/qa/cells \
        --stage before --suffix ""
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
CELL_W, CELL_H = 420, 320
LABEL_H = 58
COLS, ROWS = 6, 5
PER_SHEET = COLS * ROWS
HEADER_H = 74

VIEW = 5.6


def font(size: int):
    for p in ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
              "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"):
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            pass
    return ImageFont.load_default()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cells", required=True)
    ap.add_argument("--stage", default="before")
    ap.add_argument("--suffix", default="")
    ap.add_argument("--out-dir", default=str(ROOT / "assets" / "previews"))
    args = ap.parse_args()

    cells = Path(args.cells)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    names = json.loads((cells / ("family_names%s.json" % args.suffix)).read_text())
    metrics = {m["name"]: m
               for m in json.loads((ROOT / "assets/models/metrics.json").read_text())["models"]}

    sheets = [names[i:i + PER_SHEET] for i in range(0, len(names), PER_SHEET)]
    for si, group in enumerate(sheets, 1):
        sheet_w = COLS * CELL_W
        sheet_h = HEADER_H + ROWS * (CELL_H + LABEL_H)
        sheet = Image.new("RGB", (sheet_w, sheet_h), (26, 30, 35))
        d = ImageDraw.Draw(sheet)
        d.text((18, 12), "STARFALL - BUILDINGS / STRUCTURES / INTERIORS / KIT - %s" % args.stage.upper(),
               fill=(240, 243, 248), font=font(25))
        d.text((18, 45), "Gameplay scale: 1 tile = 1u; human silhouette = 1.55u; camera yaw 45 / pitch 30; "
               "inset shows the cell at 60 px. Tall models zoomed out (factor in label).",
               fill=(170, 180, 194), font=font(15))
        for idx, name in enumerate(group):
            img_path = cells / ("%s%s.png" % (name, args.suffix))
            if not img_path.is_file():
                continue
            col, row = idx % COLS, idx // COLS
            x = col * CELL_W
            y = HEADER_H + row * (CELL_H + LABEL_H)
            img = Image.open(img_path).convert("RGB").resize((CELL_W, CELL_H), Image.LANCZOS)
            sheet.paste(img, (x, y))
            m = metrics.get(name, {})
            h = m.get("height", 0.0)
            tris = m.get("triangles", 0)
            zoom = max(1.0, (h + 0.8) / VIEW)
            label = "%s  h=%.2fu  %dt" % (name, h, tris)
            if zoom > 1.02:
                label += "  zoom 1:%.1f" % zoom
            d.rectangle((x, y + CELL_H, x + CELL_W, y + CELL_H + LABEL_H),
                        fill=(38, 44, 52))
            d.rectangle((x, y, x + CELL_W, y + CELL_H), outline=(90, 98, 110), width=1)
            d.text((x + 10, y + CELL_H + 8), label, fill=(238, 242, 248), font=font(18))
            d.text((x + 10, y + CELL_H + 32), "human 1.55u  |  tile 1u",
                   fill=(150, 164, 182), font=font(13))
            # 60 px legibility inset, taken from the same gameplay render.
            thumb = img.resize((60, 46), Image.LANCZOS)
            tx, ty = x + CELL_W - 72, y + 6
            d.rectangle((tx - 3, ty - 3, tx + 63, ty + 49), fill=(20, 24, 29))
            sheet.paste(thumb, (tx, ty))
            d.text((tx, ty + 49), "60px", fill=(190, 200, 215), font=font(11))
        out = out_dir / ("family_gameplay_%s_%02d.png" % (args.stage, si))
        sheet.save(out)
        print("SHEET", out, sheet.size, "models", len(group))


if __name__ == "__main__":
    main()
