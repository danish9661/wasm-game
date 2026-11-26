# Round 1 — independent critic (fresh sub-agent)

VERDICT: REJECT. Average ≈ 6.8 (ship bar: every axis ≥ 8, average ≥ 8.5). Hard fail: lint FAIL (4 contrast runs per viewport).

Scores: task clarity 8 · information hierarchy 7 · layout & density 7 · typography & numbers 6 · colour & theme 8 · states 7 · interaction & feedback 6 · keyboard & accessibility 5 · responsiveness 6 · craft & consistency 7 · originality 7 · conversion 8.

Ranked defects:
1. Desktop + mobile HUD, all states — contrast FAIL: HP/HUNGER/THIRST + a fourth label at 4.42:1 vs AA 4.5. Fix: lighten small-label token ~10%, target ≥4.6:1.
2. Mobile HUD — vitals are numbers without bars (128×82 panel, no proportional bars). Fix: restore four ≥6px-tall, ≥100px-wide filled bars with right-aligned numbers; panel ≥130×110px.
3. Mobile HUD — playfield occlusion: right action grid x≈196–382,y≈590–838; left icon column and joystick blanket bottom third. Fix: cap persistent controls to ~120px tray; reveal extra grid on tap; minimap as 72×72 corner chip.
4. Desktop HUD, all states — shipped debug output (FPS, SPD, @ coords, MOBS, KEY). Fix: delete or gate behind a dev toggle; keep only objective timer.
5. Desktop + mobile inventory — no visible close affordance; controls visible through drawer. Fix: 44×44 X top-right, I/Esc toggle, dim underlying controls.
6. Mobile settings — "Medium (960×540)" shown on a 390×844 phone with Auto Resolution on. Fix: show "Auto · 390×844" or hide the control.
7. Desktop HUD/codex — keyboard discovery: only Codex shows a shortcut; no visible focus treatment. Fix: keycaps on top-right icons; visible focus ring.

Three highest-leverage changes: (1) raise every small-label colour above 4.5:1; (2) restore mobile vitals bars and collapse the action cluster into a 120px tray; (3) strip debug telemetry and add a universal keyboard-reachable close on the inventory drawer.
