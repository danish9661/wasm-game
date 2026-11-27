"""Re-open every third-wave GLB and re-measure it against its manifest.

    blender -b --python scripts/verify_wave3_animations.py

The rig derivation, re-import, slot binding, sampler and gate checks are the
same as verify_wave2_animations.py; only the manifest and label differ.
"""

import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import verify_wave2_animations as v2  # noqa: E402

v2.MANIFEST = os.path.join(ROOT, "assets", "animations", "wave3", "manifest.json")
v2.LABEL = "third-wave"

if __name__ == "__main__":
    sys.exit(v2.main())
