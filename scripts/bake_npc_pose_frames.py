"""Bake authored NPC/enemy clips into per-frame static pose GLBs.

    blender -b --python scripts/bake_npc_pose_frames.py

This is the family-wide companion to ``scripts/bake_pose_frames.py`` (player).
The wave2/3/4 and world animation GLBs ship a real object-empty pivot rig with
every mesh part bound to a pivot.  We read the rest joint positions from each
clip's node TRS, take the family's *authored* low-poly mesh from its base GLB,
skin each part onto its rig pivot (``M_pose @ inv(M_rest)``), evaluate the
authored frames, and export one flattened, normalised static GLB per frame:

    assets/models/npcpose_<family>_<clip>_<NN>.glb

Contract per frame (identical to the player pose bake):
  * one merged mesh, flat shaded, palette colours only, no textures;
  * base at Y = 0 and X/Z centre = (0, 0) in glTF space (Blender: min Z = 0,
    X/Y centre = 0) -- swapping frames never jumps;
  * translation-only skinning (no per-frame scaling) keeps every frame at the
    family's authored size;
  * under the 600-triangle budget (every source family is <= 416 triangles).

Outputs ``assets/models/npc_poses.json`` (same schema as ``poses.json``), the
per-frame GLBs, and a contact sheet at ``assets/previews/npc_pose_bake.png``
assembled by ``scripts/assemble_npc_pose_sheet.py`` (system Python + Pillow).

Frame policy: loops get 12 evenly spaced frames over one authored cycle;
one-shots get 16 evenly spaced frames including the final settle. Playback fps
is scaled from the source so each baked clip keeps its authored duration.
"""

import collections
import json
import math
import os
import struct
import subprocess
import sys

import bpy
from mathutils import Matrix, Vector

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SCRIPTS = os.path.join(ROOT, "scripts")
ANIM = os.path.join(ROOT, "assets", "animations")
MODELS = os.path.join(ROOT, "assets", "models")
PREVIEWS = os.path.join(ROOT, "assets", "previews")
FRAME_DIR = "/tmp/opencode/npc_pose_sheet_frames"

sys.path.insert(0, SCRIPTS)
from bake_pose_frames import box, ico, prism  # noqa: E402  (geometry helpers)

FPS = 30
LOOP_FRAMES = 12
ONESHOT_FRAMES = 16
TRI_BUDGET = 600

# (output clip key, source GLB relative to assets/animations).  The key is
# <family>_<clip> in npc_poses.json, e.g. "goblin_walk".  Fallbacks are recorded
# in docs/NPC_Pose_Bake.md (some families have no authored idle/walk of their
# own, exactly as documented in game/src/anim.rs).
FAMILIES = collections.OrderedDict([
    ("goblin", ("goblin", False, [
        ("goblin_idle", "wave3/goblin_idle.glb"),
        ("goblin_walk", "wave4/goblin_walk_f.glb"),
        ("goblin_attack", "wave2/goblin_attack.glb"),
    ])),
    ("skeleton", ("skeleton", False, [
        ("skeleton_idle", "wave3/skeleton_idle.glb"),
        ("skeleton_walk", "wave4/skeleton_walk_f.glb"),
        ("skeleton_attack", "wave2/skeleton_attack.glb"),
    ])),
    ("wolf", ("npc_dire_wolf", False, [
        ("wolf_idle", "wave3/wolf_idle.glb"),
        ("wolf_walk", "wave4/wolf_walk_f.glb"),
        ("wolf_attack", "wave2/wolf_attack.glb"),
    ])),
    ("archer", ("archer", False, [
        ("archer_idle", "wave3/archer_idle.glb"),
        ("archer_walk", "wave4/archer_walk_f.glb"),
        ("archer_attack", "wave2/archer_shoot.glb"),
    ])),
    ("raider", ("raider", False, [
        ("raider_idle", "raider_sneak.glb"),
        ("raider_walk", "raider_sneak.glb"),
        ("raider_attack", "raider_slash.glb"),
    ])),
    ("guard", ("guard", False, [
        ("guard_idle", "wave3/guard_watch.glb"),
        ("guard_walk", "wave2/guard_patrol.glb"),
        ("guard_attack", "guard_shield_bash.glb"),
    ])),
    ("golem", ("golem", False, [
        ("golem_idle", "wave2/golem_standby.glb"),
        ("golem_walk", "wave3/golem_work.glb"),
        ("golem_attack", "wave3/golem_work.glb"),
    ])),
    ("brute", ("brute", False, [
        ("brute_idle", "wave3/brute_idle.glb"),
        ("brute_walk", "wave3/brute_walk.glb"),
        ("brute_attack", "wave2/brute_smash.glb"),
    ])),
    ("slime", ("slime", False, [
        ("slime_idle", "wave3/slime_idle.glb"),
        ("slime_walk", "wave2/slime_hop.glb"),
        ("slime_attack", "wave2/slime_hop.glb"),
    ])),
    ("spider", ("spider", False, [
        ("spider_idle", "wave3/spider_idle.glb"),
        ("spider_walk", "wave2/spider_scuttle.glb"),
        ("spider_attack", "wave2/spider_attack.glb"),
    ])),
    ("imp", ("imp", False, [
        ("imp_idle", "imp_hover.glb"),
        ("imp_walk", "imp_mischief.glb"),
        ("imp_attack", "imp_mischief.glb"),
    ])),
    # wraith.glb is a single merged mesh with no authored clip. anim.rs maps
    # wraith onto the skeleton clips, so we skin a wraith-styled palette proxy
    # onto that real skeleton rig (same approach as the player bake).
    ("wraith", ("wraith", True, [
        ("wraith_idle", "wave3/skeleton_idle.glb"),
        ("wraith_walk", "wave4/skeleton_walk_f.glb"),
        ("wraith_attack", "wave2/skeleton_attack.glb"),
    ])),
    ("scorpion_queen", ("scorpion_queen", False, [
        ("scorpion_queen_idle", "wave2/scorpion_queen_telegraph.glb"),
        ("scorpion_queen_walk", "wave2/scorpion_queen_telegraph.glb"),
        ("scorpion_queen_attack", "wave2/scorpion_queen_attack.glb"),
    ])),
    ("toad_king", ("toad_king", False, [
        ("toad_king_idle", "wave2/toad_king_telegraph.glb"),
        ("toad_king_walk", "wave2/toad_king_telegraph.glb"),
        ("toad_king_attack", "wave2/toad_king_attack.glb"),
    ])),
    ("stormcaller", ("stormcaller", False, [
        ("stormcaller_idle", "wave2/stormcaller_telegraph.glb"),
        ("stormcaller_walk", "wave2/stormcaller_telegraph.glb"),
        ("stormcaller_attack", "wave2/stormcaller_attack.glb"),
    ])),
    ("colossus", ("colossus", False, [
        ("colossus_idle", "wave2/colossus_telegraph.glb"),
        ("colossus_walk", "wave2/colossus_telegraph.glb"),
        ("colossus_attack", "wave2/colossus_attack.glb"),
    ])),
    ("villager", ("npc_villager_child", False, [
        ("villager_idle", "wave2/villager_idle.glb"),
        ("villager_walk", "wave3/villager_gather.glb"),
        ("villager_wave", "wave2/villager_wave.glb"),
    ])),
    ("merchant", ("npc_merchant", False, [
        ("merchant_idle", "wave3/merchant_count.glb"),
        ("merchant_walk", "wave4/merchant_greet_loop.glb"),
        ("merchant_greet", "wave2/merchant_greet.glb"),
    ])),
])

# Contact-sheet rows: (clip key, number of baked frames to show).
SHEET_CLIPS = [
    ("goblin_walk", 12),
    ("goblin_attack", 12),
    ("skeleton_walk", 12),
    ("wolf_walk", 12),
    ("spider_walk", 12),
    ("scorpion_queen_attack", 12),
]


# ---------------------------------------------------------------------------
# GLB / glTF helpers
# ---------------------------------------------------------------------------
def gltf_json(path):
    data = open(path, "rb").read()
    magic, version, length = struct.unpack_from("<4sII", data, 0)
    if magic != b"glTF" or version != 2:
        raise ValueError("not a GLB: %s" % path)
    clen, ctype = struct.unpack_from("<I4s", data, 12)
    if ctype != b"JSON":
        raise ValueError("no JSON chunk: %s" % path)
    return json.loads(data[20:20 + clen].decode("utf-8").rstrip("\x00 \r\n\t"))


def node_parents(gltf):
    parent = {}
    for i, node in enumerate(gltf.get("nodes", [])):
        for child in node.get("children", []):
            parent[child] = i
    return parent


def rest_joints(gltf):
    """Node TRS -> world rest joints in Blender Z-up, from the authored clip."""
    nodes = gltf.get("nodes", [])
    parent = node_parents(gltf)

    def local(i):
        t = nodes[i].get("translation", [0.0, 0.0, 0.0])
        return Vector((float(t[0]), -float(t[2]), float(t[1])))

    out = {}
    for i, node in enumerate(nodes):
        name = node.get("name")
        if not name:
            continue
        p = Vector((0.0, 0.0, 0.0))
        chain = []
        cur = i
        while cur in parent:
            chain.append(cur)
            cur = parent[cur]
        p += local(cur)
        for n in reversed(chain):
            p += local(n)
        out[name] = p
    return out


def mesh_bindings(gltf):
    """Mesh node name -> pivot (parent empty) name, from the authored rig."""
    nodes = gltf.get("nodes", [])
    parent = node_parents(gltf)
    out = {}
    for i, node in enumerate(nodes):
        if "mesh" not in node:
            continue
        # A few world clips leave an cosmetic part (e.g. guard's belt) at the
        # scene root; it still follows root motion, so bind it to "root".
        p = nodes[parent[i]] if i in parent else next(
            n for n in nodes if n.get("name") == "root")
        out[node.get("name")] = p.get("name")
    return out


def clip_meta():
    """clip name -> (frames, loop) from the authored wave/world manifests."""
    meta = {}
    # World clips first: wave manifests are authoritative where a clip name
    # appears in both (e.g. goblin_attack / archer_shoot).
    for c in json.load(open(os.path.join(ANIM, "world_manifest.json")))["clips"]:
        if "frames" in c:
            meta[c["clip"]] = (int(c["frames"]), bool(c.get("loop")))
    for wave in ("wave2", "wave3", "wave4"):
        path = os.path.join(ANIM, wave, "manifest.json")
        for c in json.load(open(path))["clips"]:
            meta[c["clip"]] = (int(c["frames"]), bool(c["loop"]))
    return meta


# ---------------------------------------------------------------------------
# Low-poly parts: authored family mesh, or a palette proxy for wraith
# ---------------------------------------------------------------------------
def bsdf_color(mat):
    try:
        node = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        return tuple(float(c) for c in node.inputs["Base Color"].default_value[:3])
    except (StopIteration, AttributeError):
        return tuple(float(c) for c in mat.diffuse_color[:3])


def import_parts(model_path):
    """Import the authored family GLB; return parts bound to their rig pivots."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.context.scene.render.fps = FPS
    bpy.ops.import_scene.gltf(filepath=model_path)
    parts = []
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        mesh = obj.data
        mesh.calc_loop_triangles()
        verts = [obj.matrix_world @ v.co for v in mesh.vertices]
        tris = []
        for lt in mesh.loop_triangles:
            mi = mesh.polygons[lt.polygon_index].material_index
            color = bsdf_color(mesh.materials[mi]) if mesh.materials else (0.5, 0.5, 0.5)
            tris.append((tuple(lt.vertices), color))
        parts.append({"name": obj.name, "verts": verts, "tris": tris})
    if not parts:
        raise ValueError("%s has no mesh parts" % model_path)
    return parts


def build_wraith_body(joints):
    """Skeleton-rig proxy for the merged, unanimated wraith.glb.

    Colours come from wraith.glb's own palette (cloth / bone / arcane / eye).
    All geometry is expressed in the skeleton rig's rest-joint space, so the
    exact same skinning/baking code handles it as the authored families.
    """
    palette = {
        "cloth": (0.0409, 0.0844, 0.1946),
        "bone": (0.8070, 0.7454, 0.6038),
        "arcane": (0.1470, 0.0497, 0.3515),
        "eye": (0.0103, 0.0103, 0.0103),
        "glow": (0.65, 0.32, 0.95),
    }
    J = joints
    parts = []

    def add(joint, verts, faces, color):
        parts.append({"name": "wraith_" + joint + "_" + str(len(parts)),
                      "joint": joint, "verts": verts, "tris": faces,
                      "color": palette[color]})

    def tri_faces(faces, color):
        return [(tuple(f), palette[color]) for f in faces]

    torso, head = J["torso"], J["head"]
    verts, faces = prism(torso + Vector((0, 0, 0.02)),
                         head - Vector((0, 0, 0.06)), 0.14)
    add("torso", verts, tri_faces(faces, "cloth"), "cloth")
    verts, faces = box(torso + Vector((0, 0.02, -0.16)), (0.26, 0.20, 0.26))
    add("torso", verts, tri_faces(faces, "cloth"), "cloth")

    verts, faces = ico(head + Vector((0, -0.02, 0.045)), 0.125)
    add("head", verts, tri_faces(faces, "cloth"), "cloth")
    verts, faces = ico(head + Vector((0, -0.10, 0.045)), 0.085)
    add("head", verts, tri_faces(faces, "eye"), "eye")
    for side in (-1, 1):
        verts, faces = ico(head + Vector((side * 0.045, -0.165, 0.07)), 0.018)
        add("head", verts, tri_faces(faces, "glow"), "glow")

    for side, tag in ((-1, "l"), (1, "r")):
        upper, lower = "arm_" + tag, "lower_arm_" + tag
        verts, faces = prism(J[upper], J[lower], 0.052)
        add(upper, verts, tri_faces(faces, "cloth"), "cloth")
        tip = J[lower] + Vector((side * 0.03, -0.02, -0.17))
        verts, faces = prism(J[lower], tip, 0.040)
        add(lower, verts, tri_faces(faces, "bone"), "bone")
        upper, lower = "leg_" + tag, "lower_leg_" + tag
        verts, faces = prism(J[upper], J[lower], 0.070)
        add(upper, verts, tri_faces(faces, "cloth"), "cloth")
        tip = J[lower] + Vector((0, -0.01, -0.16))
        verts, faces = prism(J[lower], tip, 0.050)
        add(lower, verts, tri_faces(faces, "cloth"), "cloth")
    return parts


# ---------------------------------------------------------------------------
# Bake / export
# ---------------------------------------------------------------------------
def make_materials(colors):
    """Fresh flat, untextured palette materials keyed by sRGB-ish colour."""
    mats = {}
    for i, color in enumerate(sorted(set(colors))):
        mat = bpy.data.materials.new("npcpose_palette_%02d" % i)
        mat.use_nodes = True
        mat.diffuse_color = (*color, 1.0)
        bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.85
        bsdf.inputs["Metallic"].default_value = 0.0
        mats[color] = mat
    return mats


def posed_frame(parts, bindings, rig, joints, mats, mat_order):
    """One normalised, flat-shaded merged mesh at the current scene frame."""
    bpy.context.view_layer.update()
    verts, faces, mat_ids = [], [], []
    for part in parts:
        pivot = part.get("joint") or bindings[part["name"]]
        delta = rig[pivot].matrix_world @ Matrix.Translation(-joints[pivot])
        base = len(verts)
        verts.extend(delta @ v for v in part["verts"])
        for indices, color in part["tris"]:
            faces.append(tuple(base + i for i in indices))
            mat_ids.append(mat_order.index(color))
    if not verts:
        raise ValueError("empty baked frame")

    xs = [v.x for v in verts]
    ys = [v.y for v in verts]
    zs = [v.z for v in verts]
    shift = Vector((-(min(xs) + max(xs)) / 2.0,
                    -(min(ys) + max(ys)) / 2.0,
                    -min(zs)))
    verts = [v + shift for v in verts]

    mesh = bpy.data.meshes.new("npcpose_flat")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    for color in mat_order:
        mesh.materials.append(mats[color])
    for poly, mi in zip(mesh.polygons, mat_ids):
        poly.material_index = mi
        poly.use_smooth = False
    obj = bpy.data.objects.new("npcpose", mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj, max(zs) - min(zs)


def export_pose(obj, path):
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.export_scene.gltf(
        filepath=path, export_format="GLB", export_yup=True,
        export_animations=False, export_apply=False, export_extras=False,
        use_selection=True)


def frame_indices(total, loop):
    if loop:
        cycle = total - 1  # the last authored key duplicates frame 0
        return [round(i * cycle / LOOP_FRAMES) for i in range(LOOP_FRAMES)]
    return [round(i * (total - 1) / (ONESHOT_FRAMES - 1))
            for i in range(ONESHOT_FRAMES)]


def render_sheet_frames(files, heights):
    os.makedirs(FRAME_DIR, exist_ok=True)
    for path in files:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=path)
        scene = bpy.context.scene
        scene.render.engine = "BLENDER_WORKBENCH"
        scene.render.resolution_x = 256
        scene.render.resolution_y = 256
        scene.render.image_settings.file_format = "PNG"
        scene.display.shading.light = "STUDIO"
        scene.display.shading.color_type = "MATERIAL"
        scene.display.shading.show_shadows = True
        scene.display.shading.background_type = "VIEWPORT"
        scene.display.shading.background_color = (0.32, 0.34, 0.36)
        height = heights.get(os.path.basename(path), 1.55)
        cam_data = bpy.data.cameras.new("cam")
        cam_data.type = "ORTHO"
        cam_data.ortho_scale = max(0.9, height * 1.35)
        cam = bpy.data.objects.new("cam", cam_data)
        scene.collection.objects.link(cam)
        cam.location = (height * 1.6, -height * 1.6, height * 1.5)
        target = Vector((0, 0, height * 0.5))
        cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.camera = cam
        scene.render.filepath = os.path.join(
            FRAME_DIR, os.path.basename(path).replace(".glb", ".png"))
        bpy.ops.render.render(write_still=True)


def main():
    os.makedirs(MODELS, exist_ok=True)
    os.makedirs(PREVIEWS, exist_ok=True)
    meta = clip_meta()
    poses, heights = {}, {}
    all_files, proxies = [], {}

    for family, (model, use_proxy, clips) in FAMILIES.items():
        source = os.path.join(MODELS, model + ".glb")
        # The authored mesh is imported once per family and reused by all of
        # that family's clip bakes (Python data survives factory resets).
        parts = import_parts(source)
        tris = sum(len(p["tris"]) for p in parts)
        if tris > TRI_BUDGET and not use_proxy:
            raise ValueError("%s: %d triangles exceeds the %d budget"
                             % (family, tris, TRI_BUDGET))

        for key, rel in clips:
            clip_name = os.path.basename(rel).replace(".glb", "")
            total, loop = meta.get(clip_name, (48, False))
            path = os.path.join(ANIM, rel)
            gltf = gltf_json(path)
            joints = rest_joints(gltf)
            bindings = mesh_bindings(gltf)

            bpy.ops.wm.read_factory_settings(use_empty=True)
            bpy.context.scene.render.fps = FPS
            bpy.ops.import_scene.gltf(filepath=path)
            # Only the authored empties drive the skin; clip meshes are dropped
            # so no frame can accidentally render the clip's rigid source mesh.
            for obj in list(bpy.data.objects):
                if obj.type == "MESH":
                    bpy.data.objects.remove(obj, do_unlink=True)
            bpy.context.view_layer.update()
            rig = {n: bpy.data.objects[n] for n in joints if n in bpy.data.objects}
            missing = {n for n in bindings.values() if n not in rig}
            if missing:
                raise ValueError("%s missing rig pivots: %s" % (clip_name, sorted(missing)))

            if use_proxy:
                clip_parts = build_wraith_body(joints)
                colors = [p["color"] for p in clip_parts]
                for p in clip_parts:
                    p.setdefault("joint", p["joint"])
            else:
                clip_parts = parts
                colors = [color for p in parts for _, color in p["tris"]]
                for p in parts:
                    if p["name"] not in bindings:
                        raise ValueError("%s: mesh %s not bound to a pivot"
                                         % (key, p["name"]))
            mat_order = sorted(set(colors))
            mats = make_materials(mat_order)
            tris = sum(len(p["tris"]) for p in clip_parts)
            if tris > TRI_BUDGET:
                raise ValueError("%s: %d triangles exceeds the %d budget"
                                 % (key, tris, TRI_BUDGET))

            scene = bpy.context.scene
            indices = frame_indices(total, loop)
            files = []
            for out_i, f in enumerate(indices):
                scene.frame_set(f)
                obj, height = posed_frame(clip_parts, bindings, rig, joints,
                                          mats, mat_order)
                filename = "npcpose_%s_%02d.glb" % (key, out_i)
                export_pose(obj, os.path.join(MODELS, filename))
                bpy.data.objects.remove(obj, do_unlink=True)
                files.append(filename)
                heights[filename] = height
                all_files.append(filename)
            # Keep the authored duration. A loop's last key duplicates frame 0,
            # so one cycle is (total-1)/FPS seconds; a one-shot runs for
            # total/FPS seconds. 12 / 16 baked frames cover that same span.
            cycle = total - 1 if loop else total
            play_fps = round(len(files) * FPS / cycle, 6)
            poses[key] = {"fps": play_fps, "frames": files, "loop": loop}
            print("[npc-bake] %-24s %2d frames @ %-9g fps (%3d authored, %3d tris, %s)"
                  % (key, len(files), play_fps, total, tris,
                     "loop" if loop else "one-shot"))

        proxies[family] = use_proxy

    with open(os.path.join(MODELS, "npc_poses.json"), "w") as fh:
        json.dump(poses, fh, indent=2)
        fh.write("\n")

    # Render a contact sheet from the *baked* GLBs (workbench, no textures).
    sheet_files = []
    for key, count in SHEET_CLIPS:
        sheet_files.extend(poses[key]["frames"][:count])
    render_sheet_frames([os.path.join(MODELS, f) for f in sheet_files], heights)
    python = "/usr/bin/python3" if os.path.exists("/usr/bin/python3") else "python3"
    subprocess.run(
        [python, os.path.join(SCRIPTS, "assemble_npc_pose_sheet.py"),
         FRAME_DIR, os.path.join(PREVIEWS, "npc_pose_bake.png")],
        check=True)
    print("[npc-bake] npc_poses.json: %d clips, %d GLBs; height %.3f..%.3f u"
          % (len(poses), sum(len(p["frames"]) for p in poses.values()),
             min(heights.values()), max(heights.values())))
    return 0


if __name__ == "__main__":
    sys.exit(main())
