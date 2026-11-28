#!/usr/bin/env python3
"""Generate web/src/generated_extra_poses.rs from the optional baked pose manifests.

Two asset pipelines may drop JSON manifests into assets/models/:

  * npc_poses.json  -> NPC clip tables, GLBs named npcpose_<family>_<clip>_<NN>.glb
  * prop_poses.json -> prop tables,     GLBs named animprop_<name>_<NN>.glb

Neither is required: when a manifest is absent an empty table is emitted, so the
renderer keeps its original static GLBs and procedural motion. Accepted JSON
shapes are deliberately loose (family -> clip -> {frames, fps}, flat
"<family>_<clip>" keys, or a simple name -> frames map) so the runtime does not
break when an asset agent picks a slightly different spelling.

Run from build.sh (or directly) before wasm-pack; generated_models.rs is
regenerated separately and picks up the new GLBs automatically.
"""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NPC_MANIFEST = ROOT / "assets" / "models" / "npc_poses.json"
PROP_MANIFEST = ROOT / "assets" / "models" / "prop_poses.json"
OUT = ROOT / "web" / "src" / "generated_extra_poses.rs"

# Longest first so `scorpion_queen_attack` is not split as `scorpion` + `queen_attack`.
KNOWN_FAMILIES = sorted(
    (
        "armored_goblin", "ocean_leviathan", "scorpion_queen", "stormcaller",
        "stoneslinger", "toad_king", "civilian", "colossus", "skeleton",
        "villager", "wraith", "goblin", "guard", "humanoid", "brute", "spider",
        "raider", "golem", "wolf", "ogre", "slime", "bat", "imp", "archer",
        "boss", "npc",
    ),
    key=len,
    reverse=True,
)


def load_manifest(path: Path) -> dict | None:
    """Read a manifest; missing file means 'no baked frames yet'."""
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise SystemExit(f"[gen_extra_poses] bad {path.name}: {exc}")
    if isinstance(data, list):
        # Also accept a flat list of {name/family/clip, frames, fps} records.
        converted = {}
        for index, item in enumerate(data):
            if not isinstance(item, dict):
                continue
            key = item.get("name") or "_".join(
                str(item[k]) for k in ("family", "clip") if item.get(k)
            ) or str(index)
            converted[str(key)] = item
        data = converted
    if not isinstance(data, dict):
        raise SystemExit(f"[gen_extra_poses] {path.name}: expected a JSON object or list")
    # Tolerate a single wrapper key ({"families": {...}} / {"props": {...}}).
    for wrapper in ("families", "clips", "props", "poses"):
        if len(data) == 1 and isinstance(data.get(wrapper), dict):
            return data[wrapper]
    return data


def rs(value: object) -> str:
    """Quote a string as a Rust literal (JSON-style escapes differ for \\u)."""
    out = []
    for ch in str(value):
        if ch == "\\":
            out.append("\\\\")
        elif ch == '"':
            out.append('\\"')
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\r":
            out.append("\\r")
        elif ch == "\t":
            out.append("\\t")
        elif ord(ch) < 32 or ord(ch) > 126:
            out.append("\\u{%x}" % ord(ch))
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def frame_stems(info: object) -> list[str]:
    """Extract GLB stems from a manifest value (list, dict or single string)."""
    if isinstance(info, dict):
        info = info.get("frames") or info.get("files") or info.get("glbs") or []
    if isinstance(info, str):
        info = [info]
    if not isinstance(info, (list, tuple)):
        return []
    stems: list[str] = []
    for item in info:
        if isinstance(item, dict):
            item = (item.get("file") or item.get("glb") or item.get("name")
                    or item.get("path") or "")
        if not isinstance(item, str):
            continue
        # Manifests written on Windows may use backslashes or subdirectories.
        name = item.replace("\\", "/").split("/")[-1]
        stem = name[:-4] if name.lower().endswith(".glb") else name
        if stem and stem not in stems:
            stems.append(stem)
    return stems


def fps_of(info: object, frames: list[str]) -> float:
    """Playback rate: explicit fps/duration, else one second per clip."""
    if isinstance(info, dict):
        for key in ("fps", "frame_rate", "rate"):
            value = info.get(key)
            if isinstance(value, (int, float)) and not isinstance(value, bool) and value > 0:
                return float(value)
        duration = info.get("duration")
        if (isinstance(duration, (int, float)) and not isinstance(duration, bool)
                and duration > 0 and frames):
            return round(len(frames) / float(duration), 4)
    return float(len(frames)) if frames else 0.0


def strip_prefix(name: str, prefix: str) -> str:
    return name[len(prefix):] if name.lower().startswith(prefix) else name


def is_container(info: object) -> bool:
    """True when a value groups several clips rather than holding frames."""
    if not isinstance(info, dict) or "frames" in info or "files" in info:
        return False
    return any(isinstance(v, (dict, list)) for v in info.values())


def split_family_clip(name: str) -> tuple[str, str] | None:
    """Split a flat key such as `npcpose_scorpion_queen_attack`."""
    cleaned = strip_prefix(name.strip(), "npcpose_")
    low = cleaned.lower()
    for family in KNOWN_FAMILIES:
        if low == family:
            return family, "idle"
        if low.startswith(family + "_"):
            return family, low[len(family) + 1:]
    if "_" not in cleaned:
        return None
    family, clip = cleaned.split("_", 1)
    return family, clip


def parse_npc(data: dict | None) -> list[tuple[str, str, list[str], float]]:
    rows: list[tuple[str, str, list[str], float]] = []
    if not data:
        return rows
    for key, info in data.items():
        if is_container(info):
            family = strip_prefix(str(key).strip(), "npcpose_")
            for clip, clip_info in info.items():
                frames = frame_stems(clip_info)
                if not frames:
                    continue
                fam = str(clip_info.get("family") or family) if isinstance(clip_info, dict) else family
                name = str(clip_info.get("clip") or clip_info.get("state") or clip) \
                    if isinstance(clip_info, dict) else str(clip)
                rows.append((strip_prefix(fam, "npcpose_"), name, frames, fps_of(clip_info, frames)))
            continue
        frames = frame_stems(info)
        if not frames:
            continue
        split = split_family_clip(str(key))
        if split is None:
            continue
        family, clip = split
        if isinstance(info, dict):
            family = str(info.get("family") or family)
            clip = str(info.get("clip") or info.get("state") or clip)
        rows.append((strip_prefix(family, "npcpose_"), clip, frames, fps_of(info, frames)))
    return dedup(rows)


def parse_props(data: dict | None) -> list[tuple[str, str, list[str], float]]:
    rows: list[tuple[str, str, list[str], float]] = []
    if not data:
        return rows
    for key, info in data.items():
        name = strip_prefix(str(key).strip(), "animprop_")
        if is_container(info):
            for clip, clip_info in info.items():
                frames = frame_stems(clip_info)
                if frames:
                    rows.append((name, str(clip), frames, fps_of(clip_info, frames)))
            continue
        frames = frame_stems(info)
        if not frames:
            continue
        clip = ""
        if isinstance(info, dict):
            name = str(info.get("name") or info.get("prop") or name)
            clip = str(info.get("clip") or "")
        rows.append((strip_prefix(name, "animprop_"), clip, frames, fps_of(info, frames)))
    return dedup(rows)


def dedup(rows: list[tuple[str, str, list[str], float]]) -> list[tuple[str, str, list[str], float]]:
    """One row per (group, clip), sorted for reproducible generated files."""
    unique: dict[tuple[str, str], tuple[str, str, list[str], float]] = {}
    for group, clip, frames, fps in rows:
        unique.setdefault((group, clip), (group, clip, frames, fps))
    return [unique[k] for k in sorted(unique)]


def emit_table(name: str, rows: list[tuple[str, str, list[str], float]]) -> list[str]:
    lines = [f"pub const {name}: &[ExtraPoseClip] = &["]
    for group, clip, frames, fps in rows:
        lines.append("    ExtraPoseClip {")
        lines.append(f"        group: {rs(group)},")
        lines.append(f"        clip: {rs(clip)},")
        lines.append("        frames: &[")
        lines += [f"            {rs(frame)}," for frame in frames]
        lines.append("        ],")
        lines.append(f"        fps: {repr(float(fps))},")
        lines.append("    },")
    lines.append("];")
    return lines


def main() -> None:
    npc_rows = parse_npc(load_manifest(NPC_MANIFEST))
    prop_rows = parse_props(load_manifest(PROP_MANIFEST))
    lines = [
        "// GENERATED by scripts/gen_extra_poses.py — do not edit by hand.",
        "// Optional baked NPC/prop pose clips. Empty tables keep the original",
        "// static GLBs and procedural motion when the manifests are absent.",
        "pub struct ExtraPoseClip {",
        "    pub group: &'static str,",
        "    pub clip: &'static str,",
        "    pub frames: &'static [&'static str],",
        "    pub fps: f32,",
        "}",
        "impl game::anim::BakedClipRow for ExtraPoseClip {",
        "    fn group(&self) -> &str { self.group }",
        "    fn clip(&self) -> &str { self.clip }",
        "    fn frames(&self) -> &[&str] { self.frames }",
        "    fn fps(&self) -> f32 { self.fps }",
        "}",
    ]
    lines += emit_table("NPC_POSE_CLIPS", npc_rows)
    lines += emit_table("PROP_POSE_CLIPS", prop_rows)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("[gen_extra_poses] %d NPC clips, %d prop clips -> %s"
          % (len(npc_rows), len(prop_rows), OUT.relative_to(ROOT)))


if __name__ == "__main__":
    main()
