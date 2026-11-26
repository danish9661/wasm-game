"""Assemble the rendered frame strips into two labelled review sheets."""

import os
from PIL import Image, ImageDraw, ImageFont

SRC = "/tmp/opencode/anim_frames"
OUT = "/home/danish1075/.local/share/opencode/worktree/36f28cfe722f7ec6731f1d8a712bebe630db953c/fight-animations/assets/animations/review"

SHEETS = {
    "contact_sheet.png": [
        ("idle", [0, 30, 60, 90]),
        ("walk_f", [0, 6, 12, 18]),
        ("run_f", [0, 5, 10, 15]),
        ("sprint_f", [0, 4, 8, 12]),
        ("attack_sword_slash (windup / cut / recover)", [0, 12, 16, 20, 30, 41]),
        ("block", [0, 8, 20, 40]),
        ("dodge", [0, 6, 12, 20]),
        ("hit", [0, 6, 12, 29]),
        ("jump", [0, 10, 18, 26, 40]),
        ("death", [0, 15, 30, 42, 53]),
        ("climb", [0, 9, 18, 27]),
    ],
    "contact_sheet_life.png": [
        ("bow", [0, 12, 24, 40]),
        ("sleep", [0, 30, 60, 90]),
        ("drink", [0, 10, 20, 30]),
        ("eat", [0, 8, 16, 24, 35]),
        ("cook", [0, 15, 30, 45]),
        ("craft", [0, 12, 24, 36]),
        ("door_open", [0, 12, 20, 30]),
        ("interact", [0, 10, 18, 30]),
    ],
}

CELL = 224
LABEL = 26
COLS = 6

try:
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
except OSError:
    font = ImageFont.load_default()

os.makedirs(OUT, exist_ok=True)
for filename, rows in SHEETS.items():
    W = COLS * CELL
    H = sum(LABEL + CELL for _, _ in rows) + 12
    sheet = Image.new("RGB", (W, H), (28, 29, 31))
    draw = ImageDraw.Draw(sheet)
    y = 6
    for label, frames in rows:
        draw.text((8, y + 4), label, fill=(235, 235, 235), font=font)
        y += LABEL
        for i, f in enumerate(frames):
            name = "%s_%03d.png" % (label.split(" ")[0], f)
            img = Image.open(os.path.join(SRC, name)).convert("RGB").resize((CELL, CELL))
            sheet.paste(img, (i * CELL, y))
            draw.text((i * CELL + 6, y + 6), "f%d" % f, fill=(255, 255, 255), font=font)
        y += CELL
    path = os.path.join(OUT, filename)
    sheet.save(path)
    print("wrote", path, sheet.size)
