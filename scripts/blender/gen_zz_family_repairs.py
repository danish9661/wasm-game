"""Export the visual-QA rebuilds for the building/structure family.

Run:
    ~/.local/blender/blender -b --python scripts/blender/gen_zz_family_repairs.py

These six assets replace their older flat versions in assets/models/ with
bmesh-built, bevelled, UV-packed, weighted-normal meshes. Base stays at z=0,
the pivot stays bottom-centre, and filenames are unchanged.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from common import new_scene, collection, export_asset  # noqa: E402
import family_repairs as R  # noqa: E402
from qa_mesh import tri_count  # noqa: E402

new_scene()
M = R.materials()

BUILDERS = [
    ("statue", R.build_statue),
    ("spike", R.build_spike),
    ("interior_floor", R.build_interior_floor),
    ("interior_wall", R.build_interior_wall),
    ("kit_wall_stone", R.build_kit_wall_stone),
    ("kit_wall_wood", R.build_kit_wall_wood),
]

for name, build in BUILDERS:
    col = collection(name)
    build(col, M)
    export_asset(name, col)
    print("QA_REPAIR", name, tri_count(col), "tris")
print("QA_REPAIRS_DONE", len(BUILDERS))
