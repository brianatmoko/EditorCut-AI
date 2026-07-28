#!/usr/bin/env python3
"""OpenCut AI — Desktop launcher shim.

The main launcher has moved to apps/desktop/main.py.
This file forwards all arguments to it.

Usage:
    python3 main.py [args...]
    python3 apps/desktop/main.py [args...]  ← canonical path
"""

from __future__ import annotations
import subprocess
import sys
from pathlib import Path

_DESKTOP_LAUNCHER = Path(__file__).parent / "apps" / "desktop" / "main.py"

if __name__ == "__main__":
    print("[Shim] Redirecting to apps/desktop/main.py ...")
    result = subprocess.run(
        [sys.executable, str(_DESKTOP_LAUNCHER)] + sys.argv[1:],
    )
    sys.exit(result.returncode)
