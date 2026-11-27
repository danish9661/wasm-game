#!/usr/bin/env python3
"""Compose labelled contact sheets from render_gameplay_sheet.py cells.

Usage: python3 scripts/assemble_gameplay_sheet.py --cells assets/previews/qa_props/cells \
       --out assets/previews/qa_props

Each cell shows the model at gameplay scale next to a 1.55 u human, plus an
inset rendered at the size it would have when the human is 60 px tall.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
FONT_BOLD = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"


def font(size: int, bold: bool = False):
    try:
        return ImageFont.truetype(FONT_BOLD if bold else FONT, size)
    except OSError:
        return ImageFont.load_default()


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cells", default="assets/previews/qa_props/cells")
    ap.add_argument("--out", default="assets/previews/qa_props")
    ap.add_argument("--cols", type=int, default=4)
    args = ap.parse_args()

    cells_dir = (ROOT / args.cells).resolve()
    out_dir = (ROOT / args.out).resolve()
    manifests = sorted(cells_dir.glob("cells_*.json"))
    if not manifests:
        raise SystemExit("no cells_*.json manifests in %s" % cells_dir)
    records: list[dict] = []
    res = 512
    for manifest_path in manifests:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        res = manifest["res"]
        records.extend(manifest["models"])
    metrics = json.loads((ROOT / "assets/models/metrics.json").read_text(encoding="utf-8"))
    by_name = {m["name"]: m for m in metrics["models"]}

    groups: dict[str, list[dict]] = {}
    for rec in records:
        groups.setdefault(rec["group"], []).append(rec)

    title_h, label_h, pad = 92, 58, 14
    made = []
    for gname, records in groups.items():
        cols = min(args.cols, len(records))
        rows = math.ceil(len(records) / cols)
        w = pad + cols * (res + pad)
        h = title_h + rows * (res + label_h + pad) + pad
        sheet = Image.new("RGB", (w, h), (24, 26, 30))
        d = ImageDraw.Draw(sheet)
        d.text((pad, 14), "STARFALL - VISUAL QA / %s" % gname.upper(),
               font=font(30, True), fill=(255, 255, 255))
        d.text((pad, 55),
               "Gameplay scale: 1 unit = 1 tile | human silhouette = 1.55 u | "
               "camera yaw 45 deg, pitch 30 deg | inset: model when human = 60 px",
               font=font(17), fill=(178, 186, 198))
        for i, rec in enumerate(records):
            col, row = i % cols, i // cols
            x = pad + col * (res + pad)
            y = title_h + row * (res + label_h + pad)
            cell = Image.open(cells_dir / rec["cell"]).convert("RGB")
            sheet.paste(cell, (x, y))
            d.rectangle((x, y, x + res - 1, y + res - 1), outline=(90, 96, 106), width=1)

            # 60 px hero inset (human occupies 60 px); size derived from ortho scale.
            thumb_size = int(round(60.0 * rec["ortho"] / 1.55))
            thumb = cell.resize((thumb_size, thumb_size), Image.Resampling.LANCZOS)
            tx, ty = x + 8, y + res - thumb_size - 8
            sheet.paste(thumb, (tx, ty))
            d.rectangle((tx - 2, ty - 2, tx + thumb_size + 1, ty + thumb_size + 1),
                        outline=(255, 255, 255), width=2)
            d.text((tx + 4, ty - 20), "60 px hero", font=font(13, True), fill=(255, 255, 255))

            m = by_name.get(rec["name"], {})
            label = "%s  |  %s tris  |  %.2f u" % (
                rec["name"], m.get("triangles", "?"), m.get("height", rec["height"]))
            d.text((x + 6, y + res + 12), label, font=font(19, True), fill=(240, 242, 245))
        path = out_dir / ("sheet_%s.png" % gname)
        sheet.save(path, optimize=True)
        made.append(path)
        print("SHEET", path, sheet.size, len(records), "models")

    # A quick-scan overview of the 60 px insets alone, grouped.
    overview_cols = 8
    thumb_w = 170
    row_h = thumb_w + 34
    overview_rows = sum(math.ceil(len(rs) / overview_cols) for rs in groups.values()) + len(groups)
    ov = Image.new("RGB", (pad + overview_cols * (thumb_w + pad), title_h + overview_rows * (row_h + pad)),
                   (24, 26, 30))
    d = ImageDraw.Draw(ov)
    d.text((pad, 14), "STARFALL - 60 PX READABILITY OVERVIEW", font=font(30, True), fill="white")
    d.text((pad, 55), "Every family model rendered with the 1.55 u human at 60 px",
           font=font(17), fill=(178, 186, 198))
    yy = title_h
    for gname, records in groups.items():
        d.text((pad, yy), gname.upper(), font=font(20, True), fill=(255, 196, 90))
        yy += 28
        for i, rec in enumerate(records):
            col, row = i % overview_cols, i // overview_cols
            x = pad + col * (thumb_w + pad)
            y = yy + row * (row_h + pad)
            cell = Image.open(cells_dir / rec["cell"]).convert("RGB")
            thumb_size = int(round(60.0 * rec["ortho"] / 1.55))
            thumb = cell.resize((thumb_size, thumb_size), Image.Resampling.LANCZOS)
            ov.paste(thumb, (x, y))
            d.text((x, y + thumb_w + 2), rec["name"], font=font(13), fill=(226, 230, 236))
        yy += math.ceil(len(records) / overview_cols) * (row_h + pad) + 10
    ov_path = out_dir / "sheet_overview_60px.png"
    ov.save(ov_path, optimize=True)
    print("SHEET", ov_path, ov.size)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
