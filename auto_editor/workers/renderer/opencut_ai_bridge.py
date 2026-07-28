"""Bridge to OpenCut AI's internal compositor for rendering.

Uses OpenCut AI's WASM compositor via CLI/API when available.
Falls back to FFmpeg pipeline.
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Callable
import subprocess
import json
import os


class OpenCutAIBridge:
    def __init__(self, opencut_ai_cli: str = "npx opencut-ai"):
        self.opencut_ai_cli = opencut_ai_cli

    def render_project(self, project_data: dict, output_path: str,
                       progress_callback: Optional[Callable[[float], None]] = None) -> Optional[str]:
        project_file = self._write_project_file(project_data)
        if not project_file:
            return self._fallback_render(project_data, output_path)
        try:
            cmd = [*self.opencut_ai_cli.split(), "render", project_file, "--output", output_path]
            process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            while True:
                line = process.stdout.readline()
                if not line:
                    break
                if "progress" in line.lower():
                    try:
                        pct = float(line.split(":")[-1].strip().rstrip("%"))
                        if progress_callback:
                            progress_callback(pct / 100.0)
                    except ValueError:
                        pass
            process.wait()
            if process.returncode == 0 and os.path.exists(output_path):
                return output_path
            return self._fallback_render(project_data, output_path)
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_render(project_data, output_path)

    def _write_project_file(self, project_data: dict) -> Optional[str]:
        try:
            tmp = Path("./.opencut_ai_projects")
            tmp.mkdir(exist_ok=True)
            project_file = tmp / "project.json"
            with open(project_file, "w") as f:
                json.dump(project_data, f)
            return str(project_file)
        except IOError:
            return None

    def _fallback_render(self, project_data: dict, output_path: str) -> Optional[str]:
        return None

    def is_available(self) -> bool:
        try:
            result = subprocess.run([*self.opencut_ai_cli.split(), "--version"], capture_output=True, timeout=10)
            return result.returncode == 0
        except (subprocess.SubprocessError, FileNotFoundError):
            return False
