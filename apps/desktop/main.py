#!/usr/bin/env python3
"""OpenCut AI Desktop Launcher — runs the native GPUI desktop app.

Usage:
    python3 apps/desktop/main.py                # Run desktop app (cargo run)
    python3 apps/desktop/main.py --release      # Run optimized release build
    python3 apps/desktop/main.py --api          # Run desktop + auto-editor API
    python3 apps/desktop/main.py --api-only     # Run auto-editor API only
    python3 apps/desktop/main.py --help         # Show this help
"""

from __future__ import annotations
import argparse
import subprocess
import sys
import os
import signal
import time
import socket
from pathlib import Path


# Root of the monorepo (two levels up from apps/desktop/)
ROOT_DIR = Path(__file__).parent.parent.parent

# Prepend ~/.cargo/bin and local project bin to PATH and LIBRARY_PATH
_CARGO_BIN = Path.home() / ".cargo" / "bin"
_LOCAL_BIN = ROOT_DIR / "bin"

path_parts = []
if _CARGO_BIN.exists():
    path_parts.append(str(_CARGO_BIN))
if _LOCAL_BIN.exists():
    path_parts.append(str(_LOCAL_BIN))
if path_parts:
    os.environ["PATH"] = os.pathsep.join(path_parts) + os.pathsep + os.environ.get("PATH", "")

if _LOCAL_BIN.exists():
    lib_path = str(_LOCAL_BIN)
    os.environ["LIBRARY_PATH"] = lib_path + os.pathsep + os.environ.get("LIBRARY_PATH", "")
    os.environ["LD_LIBRARY_PATH"] = (
        lib_path + os.pathsep +
        "/usr/lib/x86_64-linux-gnu" + os.pathsep +
        os.environ.get("LD_LIBRARY_PATH", "")
    )


# ── Helpers ───────────────────────────────────────────────────────────────────

def check_cargo() -> bool:
    """Check if cargo (Rust toolchain) is installed."""
    try:
        subprocess.run(["cargo", "--version"], capture_output=True, check=True)
        return True
    except (subprocess.SubprocessError, FileNotFoundError):
        return False


def is_port_in_use(port: int) -> bool:
    """Check if a TCP port is already bound (SO_REUSEADDR to avoid false negatives)."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            s.bind(("0.0.0.0", port))
            return False
        except OSError:
            return True


def free_port(port: int, max_retries: int = 5) -> bool:
    """Kill any process listening on *port* (Linux only).
    Returns True if the port was freed, False if it's still in use.
    """
    for attempt in range(max_retries):
        if not is_port_in_use(port):
            return True

        # Method 1: fuser -k (fast, works on most processes)
        killed = False
        try:
            res = subprocess.run(
                ["fuser", "-k", f"{port}/tcp"],
                capture_output=True, text=True,
            )
            if res.returncode == 0:
                killed = True
        except FileNotFoundError:
            pass

        # Method 2: lsof + kill as fallback
        if not killed:
            try:
                res = subprocess.run(
                    ["lsof", "-ti", f":{port}"],
                    capture_output=True, text=True,
                )
                pids = [p for p in res.stdout.strip().splitlines() if p]
                for pid in pids:
                    try:
                        os.kill(int(pid), signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                killed = bool(pids)
            except FileNotFoundError:
                pass

        # Method 3: sudo kill (for stubborn processes)
        if not killed or is_port_in_use(port):
            try:
                res = subprocess.run(
                    ["lsof", "-ti", f":{port}"],
                    capture_output=True, text=True,
                )
                pids = [p for p in res.stdout.strip().splitlines() if p]
                for pid in pids:
                    subprocess.run(
                        ["sudo", "kill", "-9", pid],
                        capture_output=True,
                    )
            except (FileNotFoundError, subprocess.SubprocessError):
                pass

        time.sleep(1.0)

        if attempt > 0 and not is_port_in_use(port):
            print(f"[Launcher] Freed port {port} on attempt {attempt + 1}.")
            return True

    freed = not is_port_in_use(port)
    if freed:
        print(f"[Launcher] Freed port {port}.")
    return freed


def stream_output(proc: subprocess.Popen, prefix: str) -> None:
    """Stream process stdout to our stdout."""
    try:
        for line in iter(proc.stdout.readline, ""):
            if line:
                print(f"{prefix} {line}", end="", flush=True)
    except (ValueError, OSError):
        pass


# ── Service starters ──────────────────────────────────────────────────────────

def run_desktop(release: bool = False) -> subprocess.Popen:
    """Start the OpenCut AI GPUI desktop app via cargo."""
    # Prefer custom Rust toolchain
    rust_toolchain = Path.home() / ".rustup" / "toolchains" / "stable-x86_64-unknown-linux-gnu" / "bin"
    if rust_toolchain.exists():
        os.environ.setdefault("RUSTC", str(rust_toolchain / "rustc"))
        os.environ.setdefault("CARGO", str(rust_toolchain / "cargo"))
        os.environ["PATH"] = str(rust_toolchain) + os.pathsep + os.environ.get("PATH", "")

    cmd = [os.environ.get("CARGO", "cargo"), "run", "-p", "opencut-desktop"]
    if release:
        cmd.append("--release")

    print("[Desktop] Starting OpenCut AI native desktop...")
    print(f"[Desktop]   cwd: {ROOT_DIR}")
    print(f"[Desktop]   cmd: {' '.join(cmd)}")

    proc = subprocess.Popen(
        cmd,
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    print(f"[Desktop]   PID: {proc.pid}")
    return proc


def run_api() -> subprocess.Popen:
    """Start the auto-editor Python API server."""
    venv_python = (
        ROOT_DIR / ".venv" / "bin" / "python"
        if (ROOT_DIR / ".venv" / "bin" / "python").exists()
        else "python3"
    )

    print("[API] Starting auto-editor API server...")

    proc = subprocess.Popen(
        [str(venv_python), "-m", "auto_editor.api.server"],
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    print(f"[API]   PID: {proc.pid}")
    print("[API]   URL: http://localhost:8765")
    print("[API]   Docs: http://localhost:8765/docs")
    return proc


# ── Main ──────────────────────────────────────────────────────────────────────

def main() -> int:
    parser = argparse.ArgumentParser(
        description="OpenCut AI Desktop Launcher",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 apps/desktop/main.py             Launch desktop app (debug build)
  python3 apps/desktop/main.py --release   Launch desktop app (release build)
  python3 apps/desktop/main.py --api       Launch desktop + API server
  python3 apps/desktop/main.py --api-only  Launch API server only
        """,
    )
    parser.add_argument(
        "--release", action="store_true",
        help="Build and run in release mode (slower build, faster app)",
    )
    parser.add_argument(
        "--api", action="store_true",
        help="Also start the auto-editor API server (port 8765)",
    )
    parser.add_argument(
        "--api-only", action="store_true",
        help="Start API server only — no desktop window",
    )
    args = parser.parse_args()

    # Preflight: cargo check
    if not args.api_only and not check_cargo():
        print("[Error] Rust/cargo not found. Install from https://rustup.rs", file=sys.stderr)
        print("  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh", file=sys.stderr)
        sys.exit(1)

    # Preflight: clear API port if needed
    if args.api or args.api_only:
        if is_port_in_use(8765):
            print("[Launcher] Port 8765 already in use — freeing it...")
            freed = free_port(8765)
            if not freed:
                print("[Warning] Could not free port 8765.", file=sys.stderr)
                print("[Hint] A process is stuck in D-state on port 8765. Reboot required.", file=sys.stderr)
                print("[Hint] Try: sudo ss -tlnp | grep 8765 to identify the PID.", file=sys.stderr)

    # Check MOKO OS AI gateways
    try:
        sys.path.insert(0, str(ROOT_DIR))
        from moko_bridge.moko_client import MOKOClient
        moko = MOKOClient()
        health = moko.check_health()
        if health.get("llm") or health.get("rag"):
            print("[AI] MOKO OS ✓ available")
        else:
            print("[AI] MOKO OS ⚠ offline (desktop will run in offline mode)")
    except Exception:
        print("[AI] MOKO OS ⚠ not reachable (desktop will run in offline mode)")

    # ── Start services ────────────────────────────────────────────────────────
    processes: list[tuple[str, subprocess.Popen]] = []

    if not args.api_only:
        proc = run_desktop(release=args.release)
        processes.append(("Desktop", proc))

    if args.api or args.api_only:
        proc = run_api()
        processes.append(("API", proc))

    if not processes:
        parser.print_help()
        return 0

    print(f"\n{'='*55}")
    print("OpenCut AI Desktop running. Press Ctrl+C to stop.")
    print(f"{'='*55}\n")

    # ── Graceful shutdown ─────────────────────────────────────────────────────
    def shutdown(signum, frame) -> None:
        print("\n[Launcher] Shutting down...")
        for name, proc in processes:
            if proc.poll() is None:
                print(f"[Launcher] Stopping {name} (PID: {proc.pid})...")
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    # ── Stream output ─────────────────────────────────────────────────────────
    import threading
    for name, proc in processes:
        t = threading.Thread(
            target=stream_output, args=(proc, f"[{name}]"), daemon=True,
        )
        t.start()

    # ── Wait loop ─────────────────────────────────────────────────────────────
    try:
        while True:
            dead = [(name, proc) for name, proc in processes if proc.poll() is not None]
            for name, proc in dead:
                if name == "Desktop":
                    print(f"[Launcher] Desktop exited (code: {proc.returncode})")
                    shutdown(None, None)
                    return proc.returncode or 0
                else:
                    print(f"[Launcher] {name} exited (code: {proc.returncode}) — continuing...")
                    processes.remove((name, proc))
            time.sleep(0.5)
    except KeyboardInterrupt:
        shutdown(None, None)

    return 0


if __name__ == "__main__":
    sys.exit(main())
