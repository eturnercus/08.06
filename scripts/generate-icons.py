#!/usr/bin/env python3
"""Generate valid Windows icon.ico from PNG sources. Run before Windows build."""
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("[FAIL] Pillow required: pip install pillow", file=sys.stderr)
    sys.exit(1)

ROOT = Path(__file__).resolve().parent.parent
ICONS = ROOT / "src-tauri" / "icons"


def main() -> int:
    for name in ("128x128@2x.png", "128x128.png", "32x32.png"):
        src = ICONS / name
        if src.exists():
            break
    else:
        print(f"[FAIL] No PNG source in {ICONS}", file=sys.stderr)
        return 1

    img = Image.open(src).convert("RGBA")
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    out = ICONS / "icon.ico"
    img.save(out, format="ICO", sizes=sizes)
    print(f"[ OK ] {out} ({out.stat().st_size} bytes, from {src.name})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
