"""Assemble the rendered frame strips into one labelled review sheet."""

import os
from PIL import Image, ImageDraw, ImageFont

SRC = "/tmp/opencode/anim_frames"
OUT = "/home/danish1075/.local/share/opencode/worktree/36f28cfe722f7ec6731f1d8a712bebe630db953c/fight-animations/assets/animations/review"

ROWS = [
    ("idle", [0, 30, 60, 90]),
    ("walk_f", [0, 6, 12, 18]),
    ("attack_sword_slash (windup / cut / recover)", [0, 12, 16, 20, 30, 41]),
    ("hit", [0, 6, 12, 29]),
    ("death", [0, 15, 30, 42, 53]),
]

CELL = 224
LABEL = 26
COLS = 6
W = COLS * CELL
H = sum(LABEL + CELL for _, _ in ROWS) + 12
sheet = Image.new("RGB", (W, H), (28, 29, 31))
draw = ImageDraw.Draw(sheet)
try:
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
except OSError:
    font = ImageFont.load_default()

y = 6
for label, frames in ROWS:
    draw.text((8, y + 4), label, fill=(235, 235, 235), font=font)
    y += LABEL
    for i, f in enumerate(frames):
        name = "%s_%03d.png" % (label.split(" ")[0], f)
        img = Image.open(os.path.join(SRC, name)).convert("RGB").resize((CELL, CELL))
        sheet.paste(img, (i * CELL, y))
        draw.text((i * CELL + 6, y + 6), "f%d" % f, fill=(255, 255, 255), font=font)
    y += CELL

os.makedirs(OUT, exist_ok=True)
path = os.path.join(OUT, "contact_sheet.png")
sheet.save(path)
print("wrote", path, sheet.size)
