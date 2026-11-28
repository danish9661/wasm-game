"""Bake the authored player rig clips into per-frame static pose GLBs.

    blender -b --python scripts/bake_pose_frames.py

Why: assets/models/player.glb is a single merged mesh, so the renderer can only
bob the whole body.  The animation GLBs in assets/animations/player/ carry a
real object-empty rig (PARTS / CHAIN from generate_player_final.py) but are
also shipped with that rigid merged mesh.  This script throws the merged mesh
away, skins a low-poly palette-coloured articulated body onto the *same*
empties, evaluates each authored frame, and exports one flattened, normalised
mesh per frame to assets/models/pose_<clip>_<NN>.glb.

The rest joint positions are the ones authored by generate_player_final.py:
they are read back from each clip GLB's node TRS (glTF Y-up -> Blender Z-up)
using PARTS / CHAIN / MESH_TO_JOINT from that generator.  Reading them from
the clip keeps the skin aligned with the rig actually stored in the file.

Contract per frame:
  * one merged mesh, flat shaded, palette colours only, no textures;
  * base at Y = 0 and X/Z centre = (0, 0) in glTF space (blender: min Z = 0,
    X/Y centre = 0) -- swapping frames never jumps;
  * no per-frame scaling, so the ~1.55 u silhouette keeps its authored size;
  * under the 500-triangle asset budget (the body is 316 triangles).

Outputs assets/models/poses.json, assets/models/pose_*.glb and, by default,
raw Workbench renders in /tmp/opencode/pose_sheet_frames plus the contact sheet
assets/previews/pose_bake.png (assembled with scripts/assemble_pose_sheet.py).
"""

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
ANIM = os.path.join(ROOT, "assets", "animations", "player")
MODELS = os.path.join(ROOT, "assets", "models")
PREVIEWS = os.path.join(ROOT, "assets", "previews")
FRAME_DIR = "/tmp/opencode/pose_sheet_frames"

sys.path.insert(0, SCRIPTS)
from generate_player_final import (  # noqa: E402  (generator constants)
    CHAIN, FPS, MESH_TO_JOINT, PARTS,
)

# Clips to bake.  Loops get 12 evenly spaced frames over one authored cycle;
# one-shots get every authored frame (including the generator's settle tail).
CLIPS = ("walk", "idle", "attack_1", "dodge_roll", "hit_light")
LOOP_FRAMES = 12
CYCLE_OF = {"walk": 24, "idle": 120}  # authored cycles in generate_player_final.py

# ART_DIRECTION.md palette, sRGB -> linear-ish values for the viewport material.
PALETTE = {
    "tunic": (0xFF / 255, 0x8C / 255, 0x26 / 255),
    "skin": (0xEA / 255, 0xB8 / 255, 0x8C / 255),
    "leather": (0x6B / 255, 0x44 / 255, 0x23 / 255),
    "dark_leather": (0x59 / 255, 0x2F / 255, 0x1A / 255),
}


# ---------------------------------------------------------------------------
# Rest joints authored by generate_player_final.py
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


def rest_joints(path):
    """Node TRS -> world rest joints, Blender Z-up, from the authored clip."""
    nodes = gltf_json(path).get("nodes", [])
    by_name = {n.get("name"): n for n in nodes if n.get("name")}
    missing = [n for n in PARTS if n not in by_name]
    if missing:
        raise ValueError("%s missing rig empties: %s" % (path, missing))

    def tr(name):
        t = by_name[name].get("translation", [0.0, 0.0, 0.0])
        # glTF is Y-up and faces +Z; Blender is Z-up and faces -Y.
        return Vector((float(t[0]), -float(t[2]), float(t[1])))

    joints = {"root": tr("root")}
    for name in PARTS:
        if name == "root":
            continue
        if name not in CHAIN:  # char_player_lod0 / unrelated tops
            continue
        chain = []
        cur = name
        while cur != "root":
            chain.append(cur)
            cur = CHAIN[cur]
        p = joints["root"].copy()
        for n in reversed(chain):
            p += tr(n)
        joints[name] = p
    return joints


# ---------------------------------------------------------------------------
# Low-poly articulated body
# ---------------------------------------------------------------------------
def box(center, size):
    hx, hy, hz = (s / 2.0 for s in size)
    c = Vector(center)
    verts = [c + Vector((x * hx, y * hy, z * hz))
             for x, y, z in ((-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1),
                             (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1))]
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4),
             (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
    return verts, faces


def prism(p0, p1, radius, sides=6):
    a, b = Vector(p0), Vector(p1)
    axis = (b - a).normalized()
    ref = Vector((0, 0, 1)) if abs(axis.z) < 0.9 else Vector((0, 1, 0))
    u = axis.cross(ref).normalized()
    v = axis.cross(u).normalized()
    ring = [u * (radius * math.cos(2 * math.pi * i / sides))
            + v * (radius * math.sin(2 * math.pi * i / sides))
            for i in range(sides)]
    verts = [a + r for r in ring] + [b + r for r in ring]
    faces = [(i, (i + 1) % sides, sides + (i + 1) % sides, sides + i)
             for i in range(sides)]
    faces += [tuple(reversed(range(sides))), tuple(range(sides, 2 * sides))]
    return verts, faces


def ico(center, radius):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=0, radius=radius,
                                          location=center)
    obj = bpy.context.object
    verts = [obj.matrix_world @ v.co for v in obj.data.vertices]
    faces = [tuple(p.vertices) for p in obj.data.polygons]
    bpy.data.objects.remove(obj, do_unlink=True)
    return verts, faces


def child_of(joints, name):
    """Child joint in CHAIN; segment endpoints come from the rig hierarchy."""
    for child, parent in CHAIN.items():
        if parent == name:
            return child
    raise KeyError(name)


def build_body(joints):
    """List of (joint, verts in rest world space, faces, material key)."""
    J = joints
    parts = []

    def add(joint, verts, faces, mat):
        parts.append((joint, verts, faces, mat))

    # Torso: spine pivot, spanning the hips-to-chest volume.  MESH_TO_JOINT
    # binds the original tunic / belt / head / limb meshes to these joints.
    add("spine", *prism(J["hips"] + Vector((0, 0, 0.02)), J["chest"], 0.135),
        "tunic")
    add(MESH_TO_JOINT["belt"], *box(J["hips"], (0.23, 0.17, 0.15)),
        "dark_leather")
    add("chest", *prism(J["chest"], J["neck"], 0.052), "skin")
    add(MESH_TO_JOINT["head"], *ico(J["head"] + Vector((0, 0, 0.085)), 0.12),
        "skin")
    add("head", *ico(J["head"] + Vector((0, 0, 0.158)), 0.082),
        "dark_leather")  # hair cap

    for side in ("l", "r"):
        upper = "upper_arm_" + side
        lower = "lower_arm_" + side
        hand = "hand_" + side
        add(upper, *prism(J[upper], J[child_of(J, upper)], 0.056), "tunic")
        add(MESH_TO_JOINT["arm_" + side],
            *prism(J[lower], J[child_of(J, lower)], 0.045), "skin")
        add(MESH_TO_JOINT["hand_" + side], *ico(J[hand], 0.055), "skin")

        upper = "upper_leg_" + side
        lower = "lower_leg_" + side
        foot = "foot_" + side
        add(MESH_TO_JOINT["leg_" + side],
            *prism(J[upper], J[child_of(J, upper)], 0.072), "dark_leather")
        add(lower, *prism(J[lower], J[child_of(J, lower)], 0.055),
            "leather")
        add(MESH_TO_JOINT["boot_" + side],
            *box(J[foot] + Vector((0, -0.035, -0.025)), (0.14, 0.24, 0.10)),
            "leather")
    return parts


def make_materials():
    mats = {}
    for key, color in PALETTE.items():
        mat = bpy.data.materials.new("pose_%s" % key)
        mat.use_nodes = True
        mat.diffuse_color = (*color, 1.0)  # Workbench MATERIAL colour
        bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        bsdf.inputs["Base Color"].default_value = (*color, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.85
        bsdf.inputs["Metallic"].default_value = 0.0
        mats[key] = mat
    return mats


def posed_frame(rig, joints, parts, mats, mat_order):
    """Evaluate the current frame into one normalised, flat-shaded mesh object."""
    verts, faces, mat_ids = [], [], []
    for joint, rest_verts, part_faces, mat_key in parts:
        # The empties' rest world matrix is a pure translation at the authored
        # joint; M @ inv(rest) is the pose delta to apply to rest-space verts.
        delta = rig[joint].matrix_world @ Matrix.Translation(-joints[joint])
        base = len(verts)
        verts.extend(delta @ v for v in rest_verts)
        faces.extend(tuple(base + i for i in f) for f in part_faces)
        mat_ids.extend([mat_order.index(mat_key)] * len(part_faces))

    # Normalise: min Y = 0 and X/Z centre = (0, 0) in glTF space.
    xs = [v.x for v in verts]
    ys = [v.y for v in verts]
    zs = [v.z for v in verts]
    shift = Vector((-(min(xs) + max(xs)) / 2.0,
                    -(min(ys) + max(ys)) / 2.0,
                    -min(zs)))
    verts = [v + shift for v in verts]

    mesh = bpy.data.meshes.new("pose_flat")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    for mat in mats.values():
        mesh.materials.append(mat)
    for poly, mi in zip(mesh.polygons, mat_ids):
        poly.material_index = mi
        poly.use_smooth = False
    obj = bpy.data.objects.new("pose", mesh)
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


# ---------------------------------------------------------------------------
# Contact sheet raw frames (Workbench, 45 deg yaw / 30 deg pitch)
# ---------------------------------------------------------------------------
def render_sheet_frames(paths):
    os.makedirs(FRAME_DIR, exist_ok=True)
    for path in paths:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=path)
        scene = bpy.context.scene
        scene.render.engine = "BLENDER_WORKBENCH"
        scene.render.resolution_x = 320
        scene.render.resolution_y = 320
        scene.render.image_settings.file_format = "PNG"
        scene.display.shading.light = "STUDIO"
        scene.display.shading.color_type = "MATERIAL"
        scene.display.shading.show_shadows = True
        scene.display.shading.background_type = "VIEWPORT"
        scene.display.shading.background_color = (0.32, 0.34, 0.36)
        cam_data = bpy.data.cameras.new("cam")
        cam_data.type = "ORTHO"
        cam_data.ortho_scale = 2.0
        cam = bpy.data.objects.new("cam", cam_data)
        scene.collection.objects.link(cam)
        cam.location = (2.4, -2.4, 2.4)
        target = Vector((0, -0.1, 0.78))
        cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.camera = cam
        scene.render.filepath = os.path.join(
            FRAME_DIR, os.path.basename(path).replace(".glb", ".png"))
        bpy.ops.render.render(write_still=True)


# ---------------------------------------------------------------------------
def main():
    os.makedirs(MODELS, exist_ok=True)
    os.makedirs(PREVIEWS, exist_ok=True)
    manifest = {c["clip"]: c for c in json.load(
        open(os.path.join(ANIM, "manifest.json")))["clips"]}
    poses = {}
    heights = []

    for name in CLIPS:
        info = manifest[name]
        total = int(info["frames"])
        loop = bool(info["loop"])
        fps = int(info["fps"])
        if loop:
            cycle = CYCLE_OF.get(name, total - 1)
            indices = [round(i * cycle / LOOP_FRAMES)
                       for i in range(LOOP_FRAMES)]
            # A 12-frame loop covers one authored cycle in cycle/fps seconds.
            play_fps = round(LOOP_FRAMES * fps / cycle, 6)
        else:
            indices = list(range(total))
            play_fps = fps

        path = os.path.join(ANIM, name + ".glb")
        joints = rest_joints(path)
        bpy.ops.wm.read_factory_settings(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=path)
        # Drop the rigid merged player mesh: only the authored empties drive
        # the articulated body below.
        for obj in list(bpy.data.objects):
            if obj.type == "MESH" or obj.name == "char_player":
                bpy.data.objects.remove(obj, do_unlink=True)
        bpy.context.view_layer.update()
        rig = {n: bpy.data.objects[n] for n in PARTS if n in bpy.data.objects}
        missing = [n for n in PARTS if n not in rig]
        if missing:
            raise ValueError("rig empties lost: %s" % missing)
        mats = make_materials()
        mat_order = list(PALETTE)
        parts = build_body(joints)
        tris = sum(sum(len(face) - 2 for face in f) for _, _, f, _ in parts)
        if tris > 500:
            raise ValueError("%d triangles exceeds the 500 budget" % tris)

        files = []
        scene = bpy.context.scene
        for out_i, f in enumerate(indices):
            scene.frame_set(f)
            bpy.context.view_layer.update()
            obj, height = posed_frame(rig, joints, parts, mats, mat_order)
            filename = "pose_%s_%02d.glb" % (name, out_i)
            export_pose(obj, os.path.join(MODELS, filename))
            bpy.data.objects.remove(obj, do_unlink=True)
            files.append(filename)
            heights.append(height)
        poses[name] = {"fps": play_fps, "frames": files, "loop": loop}
        print("[bake] %-11s %3d frames @ %g fps  (%d authored frames, %d tris)"
              % (name, len(files), play_fps, total, tris))

    with open(os.path.join(MODELS, "poses.json"), "w") as fh:
        json.dump(poses, fh, indent=2)
        fh.write("\n")

    # Render + assemble the walk contact sheet from the *baked* GLBs.
    sheet = os.path.join(PREVIEWS, "pose_bake.png")
    render_sheet_frames([os.path.join(MODELS, f) for f in poses["walk"]["frames"]])
    # Blender puts its own Python (no Pillow) first on PATH; use the system one.
    python = "/usr/bin/python3" if os.path.exists("/usr/bin/python3") else "python3"
    subprocess.run(
        [python, os.path.join(SCRIPTS, "assemble_pose_sheet.py"),
         FRAME_DIR, sheet, str(poses["walk"]["fps"])],
        check=True)
    print("[bake] poses.json: %d clips, %d pose GLBs; height %.3f..%.3f u; sheet %s"
          % (len(poses), sum(len(p["frames"]) for p in poses.values()),
             min(heights), max(heights), sheet))
    return 0


if __name__ == "__main__":
    sys.exit(main())
