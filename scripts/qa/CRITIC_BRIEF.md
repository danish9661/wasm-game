# Visual QA critic brief (fresh eyes)

You are judging shipped low-poly game assets as a player sees them in-game.

- Art direction: `docs/ART_DIRECTION.md` (Low-Poly Diorama, flat faceted shading, palette-only, 1 tile = 1 u, player = 1.55 u).
- Evidence: the labelled gameplay-scale contact sheets in `assets/previews/qa_props/` (`sheet_*.png` and `sheet_overview_60px.png`). Each model is rendered beside a 1.55 u human silhouette, orthographic yaw 45 deg / pitch 30 deg, with a 60 px inset.
- Judge ONLY the images and the art direction. Do not read generator scripts, model source or metrics.
- Held weapons must read as weapons in hand; props must have functional detail; nothing may be a featureless box or blob.
- Rank the worst 10 models by: flat/boxy silhouette, missing detail, wrong proportions, colour mismatch, unreadable at 60 px. Give each a severity (CRITICAL / MAJOR / MINOR), the evidence you see, and a one-line fix.
- Call out any model that reads as a plain cube, a featureless blob or has clearly wrong scale versus the human.
