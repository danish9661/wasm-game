"""Assemble the final player frame strips into a labelled review sheet."""

import os
from PIL import Image, ImageDraw, ImageFont

SRC = "/tmp/opencode/player_frames"
OUT = "/home/danish1075/.local/share/opencode/worktree/36f28cfe722f7ec6731f1d8a712bebe630db953c/fight-animations/assets/animations/player/review"

ROWS = [
    ("idle", [0, 30, 60, 90]),
    ("walk (1.05 m/s)", [0, 6, 12, 18]),
    ("run (1.60 m/s)", [0, 5, 10, 14]),
    ("attack_1 windup/cut/recover", [0, 8, 14, 20, 35]),
    ("attack_2 windup/cut/recover", [0, 8, 14, 20, 35]),
    ("attack_3 windup/cut/recover", [0, 10, 18, 26, 40]),
    ("heavy_attack windup/cut/recover", [0, 14, 26, 34, 50]),
    ("block", [0, 10, 24, 45]),
    ("dodge_roll", [0, 8, 18, 28, 40]),
    ("hit_light / hit_heavy", [0, 4, 8, 20]),
    ("death", [0, 15, 30, 45, 65]),
    ("interact", [0, 10, 20, 35]),
]

CELL, LABEL, COLS = 224, 26, 6
try:
    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
except OSError:
    font = ImageFont.load_default()

W = COLS * CELL
H = sum(LABEL + CELL for _, _ in ROWS) + 12
sheet = Image.new("RGB", (W, H), (28, 29, 31))
draw = ImageDraw.Draw(sheet)
y = 6
for label, frames in ROWS:
    draw.text((8, y + 4), label, fill=(235, 235, 235), font=font)
    y += LABEL
    for i, f in enumerate(frames):
        if label.startswith("hit_light"):
            name = "hit_light" if i < 2 else "hit_heavy"
            ff = [0, 4, 8, 20][i]
            f = ff if name == "hit_light" else [0, 5, 12, 28][i]
        else:
            name = label.split(" ")[0].split(" (")[0]
        img = Image.open(os.path.join(SRC, "%s_%03d.png" % (name, f))).convert("RGB")
        sheet.paste(img.resize((CELL, CELL)), (i * CELL, y))
        draw.text((i * CELL + 6, y + 6), "f%d" % f, fill=(255, 255, 255), font=font)
    y += CELL
os.makedirs(OUT, exist_ok=True)
path = os.path.join(OUT, "contact_sheet.png")
sheet.save(path)
print("wrote", path, sheet.size)
