"""Shared helpers for the Starfall asset-expansion generators.

The long tail of props uses the lightweight ``common.py`` primitives. Hero
pieces additionally go through the game-asset-modeling finishing steps here:
bmesh-friendly primitives, a 1-segment bevel, weighted normals, flat shading
and an explicit UV unwrap (no textures are used - the project is palette-only).

Every asset is re-based so its lowest point is exactly z = 0 and its X/Y
footprint centre is exactly (0, 0) *before* export. Measured dimensions and
triangle counts are written to ``assets/models/expansion_<family>.json`` so the
metrics documents are generated from exported data, not from intentions.
"""
import bpy
import json
import math
import os

from common import C, OUT, ROOT, collection, mat, box, cyl, cone, ico, export_asset

PLAYER_HEIGHT = 1.55  # docs/ART_DIRECTION.md / assets/spec.json


def palette():
    """Flat-colour Principled materials; colours come only from the palette."""
    P = {
        "wood":       mat("exp_wood", C["wood"]),
        "wood_dark":  mat("exp_wood_dark", C["leather"]),
        "wood_light": mat("exp_wood_light", "#7C5633"),
        "stone":      mat("exp_stone", C["stone"]),
        "stone_dark": mat("exp_stone_dark", "#5E646B"),
        "stone_light": mat("exp_stone_light", "#9BA3AB"),
        "sand":       mat("exp_sand", C["sand"]),
        "sand_dark":  mat("exp_sand_dark", "#B8944E"),
        "snow":       mat("exp_snow", C["snow"]),
        "ice":        mat("exp_ice", "#BFD9EE", rough=0.45),
        "swamp":      mat("exp_swamp", C["swamp"]),
        "swamp_dark": mat("exp_swamp_dark", "#354D32"),
        "grass":      mat("exp_grass", C["grass"]),
        "leaf":       mat("exp_leaf", C["leaf"]),
        "leaf_dark":  mat("exp_leaf_dark", "#2C6327"),
        "leaf_light": mat("exp_leaf_light", "#55953F"),
        "water":      mat("exp_water", C["water"], rough=0.35),
        "skin":       mat("exp_skin", C["skin"]),
        "hair":       mat("exp_hair", "#4A2E1A"),
        "cloth":      mat("exp_cloth", C["cloth"]),
        "player":     mat("exp_player", C["player"]),
        "bone":       mat("exp_bone", C["bone"]),
        "gold":       mat("exp_gold", C["gold"], metallic=0.45, rough=0.45),
        "iron":       mat("exp_iron", C["iron"], metallic=0.25),
        "steel":      mat("exp_steel", C["steel"], metallic=0.6, rough=0.5),
        "arcane":     mat("exp_arcane", C["arcane"], emission=C["arcane"], strength=0.5),
        "arcane_light": mat("exp_arcane_light", C["arcane"],
                            emission=C["arcane"], strength=0.2),
        "fire":       mat("exp_fire", C["fire"], emission=C["fire"], strength=0.6),
        "enemy":      mat("exp_enemy", C["enemy"]),
        "coral":      mat("exp_coral", "#D96B5B"),
    }
    return P


P = palette()


def bounds(col):
    """World-space AABB of a collection, including rotated/scaled primitives."""
    lo = [float("inf")] * 3
    hi = [float("-inf")] * 3
    for o in col.objects:
        if o.type != "MESH":
            continue
        for corner in o.bound_box:
            p = o.matrix_world @ __import__("mathutils").Vector(corner)
            for k in range(3):
                lo[k] = min(lo[k], p[k])
                hi[k] = max(hi[k], p[k])
    if lo[0] == float("inf"):
        return [0.0] * 3, [0.0] * 3
    return lo, hi


def rebase(col):
    """Put the footprint centre at (0, 0) and the lowest vertex at z = 0.

    Blender does not refresh ``matrix_world`` the moment a location is
    assigned, so flush the depsgraph before *reading* bounds as well as after
    moving objects. Otherwise a rotated/scaled primitive can report stale
    bounds and leave the exported asset floating above the grid.
    """
    bpy.context.view_layer.update()
    lo, hi = bounds(col)
    dx = -(lo[0] + hi[0]) / 2.0
    dy = -(lo[1] + hi[1]) / 2.0
    dz = -lo[2]
    for o in col.objects:
        o.location.x += dx
        o.location.y += dy
        o.location.z += dz
    bpy.context.view_layer.update()
    lo, hi = bounds(col)
    return lo, hi


def rebase_vertices(col):
    """Final data-level re-base so exported nodes keep an identity transform."""
    bpy.context.view_layer.update()
    lo, hi = bounds(col)
    from mathutils import Vector
    shift = Vector((-(lo[0] + hi[0]) / 2.0,
                    -(lo[1] + hi[1]) / 2.0,
                    -lo[2]))
    for o in col.objects:
        if o.type == "MESH":
            for v in o.data.vertices:
                v.co += shift
    bpy.context.view_layer.update()
    return bounds(col)


def tris(col):
    n = 0
    for o in col.objects:
        if o.type == "MESH":
            o.data.calc_loop_triangles()
            n += len(o.data.loop_triangles)
    return n


def hero_finish(col, bevel_width=0.008, bevel_parts=None):
    """Join -> bevel -> weighted normals -> flat shading -> smart UV pack.

    Used for hero pieces only. Bevel is a single 80-degree-angle segment, so
    silhouettes stay chunky and the triangle budget is not blown. Passing
    ``bevel_parts`` limits the bevel to named silhouette parts (e.g. a hero's
    torso and staff) instead of every small detail. The UVs are written even
    though the renderer uses flat palette colours: this keeps hero meshes ready
    for an optional textured pass.
    """
    meshes = [o for o in col.objects if o.type == "MESH"]
    if not meshes:
        return
    def bevel_object(o):
        bevel = o.modifiers.new("hero_bevel", "BEVEL")
        bevel.width = bevel_width
        bevel.segments = 1
        bevel.limit_method = "ANGLE"
        bevel.angle_limit = math.radians(80)  # leave faceted ico/cone sides alone
        bpy.context.view_layer.objects.active = o
        bpy.ops.object.modifier_apply(modifier=bevel.name)

    if bevel_parts:
        for o in meshes:
            if any(tag in o.name for tag in bevel_parts):
                bevel_object(o)

    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active

    # bmesh cleanup: consistent outward winding after joining rotated parts.
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(obj.data)
    bm.free()

    if not bevel_parts:
        bevel_object(obj)

    wn = obj.modifiers.new("hero_weighted_normals", "WEIGHTED_NORMAL")
    wn.weight = 50
    wn.keep_sharp = True
    bpy.ops.object.modifier_apply(modifier=wn.name)

    for p in obj.data.polygons:
        p.use_smooth = False

    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(island_margin=0.015)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.context.view_layer.update()


def finalize(name, col, family, target, notes="", hero=False, bevel_width=0.008,
             bevel_parts=None, records=None):
    """Finish, export and measure one asset. ``target`` is (width, depth, height)."""
    col.name = name
    if hero:
        hero_finish(col, bevel_width, bevel_parts)
    rebase(col)  # flush transforms written by hero_finish, then re-centre exactly
    rebase(col)
    # Bake object transforms into the mesh data. A joined hero room inherits the
    # active part's scale (e.g. 5 x 4 x 0.1 for a floor slab); leaving that on
    # the glTF root would give a non-uniformly scaled node and awkward normals.
    bpy.ops.object.select_all(action="DESELECT")
    for o in col.objects:
        if o.type == "MESH":
            o.select_set(True)
            bpy.context.view_layer.objects.active = o
            bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.context.view_layer.update()
    rebase_vertices(col)  # exact base/centre with identity node transforms
    export_asset(name, col)
    lo, hi = bounds(col)
    measured = {
        "width": round(hi[0] - lo[0], 4),
        "depth": round(hi[1] - lo[1], 4),
        "height": round(hi[2] - lo[2], 4),
        "base_z": round(lo[2], 4),
        "centre_x": round((lo[0] + hi[0]) / 2.0, 4),
        "centre_y": round((lo[1] + hi[1]) / 2.0, 4),
    }
    rec = {
        "name": name,
        "family": family,
        "target": {"width": target[0], "depth": target[1], "height": target[2]},
        "measured": measured,
        "ratio_to_player": round(measured["height"] / PLAYER_HEIGHT, 3),
        "triangles": tris(col),
        "materials": len({m.name for o in col.objects if o.type == "MESH"
                          for m in o.data.materials if m}),
        "hero": hero,
        "notes": notes,
    }
    if records is not None:
        records.append(rec)
    print("MEASURED %-24s %0.3f x %0.3f x %0.3f u  %4d tris  %.2fx player"
          % (name, measured["width"], measured["depth"], measured["height"],
             rec["triangles"], rec["ratio_to_player"]))
    return rec


def write_records(family, records):
    path = os.path.join(ROOT, "assets", "models", "expansion_%s.json" % family)
    with open(path, "w", encoding="utf-8") as fh:
        json.dump({"family": family, "player_height": PLAYER_HEIGHT,
                   "assets": records}, fh, indent=2)
    print("RECORDS", path, len(records))
