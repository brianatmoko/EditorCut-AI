"""Auto color grading based on scene histogram analysis.

Pure rule-based from FFmpeg signalstats.
0 token cost — no AI involved.
"""

from __future__ import annotations
from typing import Optional
import subprocess
import re


class ColorGradingEngine:
    def __init__(self, ffmpeg_path: str = "ffmpeg"):
        self.ffmpeg_path = ffmpeg_path

    def analyze_scene(self, video_path: str, start_time: float, end_time: float) -> dict:
        mid = (start_time + end_time) / 2
        try:
            cmd = [self.ffmpeg_path, "-ss", str(mid), "-i", video_path,
                   "-vframes", "1", "-vf", "signalstats", "-f", "null", "-"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            stats = {"brightness": 128, "contrast": 128, "saturation": 1.0}
            for line in result.stderr.split("\n"):
                if "signalstats" not in line:
                    continue
                yavg = re.search(r'YAVG=(\d+)', line)
                if yavg: stats["brightness"] = int(yavg.group(1))
                sat = re.search(r'SAT=(\d+)', line)
                if sat: stats["saturation"] = int(sat.group(1)) / 100
            return stats
        except subprocess.SubprocessError:
            return {"brightness": 128, "contrast": 128, "saturation": 1.0}

    def generate_filter(self, preset: str, stats: Optional[dict] = None) -> str:
        presets = {
            "cinematic": "eq=contrast=1.2:brightness=0.05:saturation=0.8,curves=green='0/0 0.5/0.4 1/1':blue='0/0 0.5/0.6 1/1'",
            "vintage": "colorchannelmixer=rr=0.8:rg=0.1:rb=0.1,curves=red='0/0.1 0.5/0.5 1/0.9'",
            "vivid": "eq=saturation=1.5:contrast=1.1:brightness=0.02",
            "monochrome": "hue=s=0,eq=contrast=1.3:brightness=0.05",
            "warm": "colorbalance=rs=0.1:gs=-0.05:bs=-0.1",
            "cool": "colorbalance=rs=-0.1:gs=0.05:bs=0.15",
        }
        if stats and preset == "cinematic":
            if stats.get("brightness", 128) < 60:
                return presets["cinematic"].replace("brightness=0.05", "brightness=0.15")
            elif stats.get("brightness", 128) > 200:
                return presets["cinematic"].replace("brightness=0.05", "brightness=-0.05")
        return presets.get(preset, "")

    def apply_to_scene(self, video_path: str, output_path: str, preset: str = "cinematic",
                       start_time: float = 0.0, end_time: Optional[float] = None) -> Optional[str]:
        stats = self.analyze_scene(video_path, start_time, end_time or start_time + 5)
        filter_str = self.generate_filter(preset, stats)
        if not filter_str:
            return video_path
        try:
            seek = f"-ss {start_time}" if start_time > 0 else ""
            duration = f"-t {end_time - start_time}" if end_time else ""
            cmd = [self.ffmpeg_path] + seek.split() + ["-i", video_path] + duration.split() + ["-vf", filter_str,
                "-c:v", "libx264", "-preset", "fast", "-c:a", "copy", output_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=300, check=True)
            return output_path if __import__('os').path.exists(output_path) else None
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
