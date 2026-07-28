"""Classify detected shots by visual characteristics.

Pure rule-based classification using FFmpeg signalstats.
0 token cost — no AI involved.
"""

from __future__ import annotations
from typing import Optional
import subprocess
import re

from .detector import Shot
from ...models import SceneType


class ShotClassifier:
    def __init__(self, ffmpeg_path: str = "ffmpeg"):
        self.ffmpeg_path = ffmpeg_path

    def classify(self, video_path: str, shot: Shot) -> SceneType:
        try:
            mid_time = (shot.start_time + shot.end_time) / 2
            stats = self._get_frame_stats(video_path, mid_time)
            if not stats:
                return SceneType.B_ROLL
            brightness = stats.get("average_brightness", 128)
            contrast = stats.get("contrast", 0)
            motion = self._estimate_motion(video_path, shot)
            if brightness > 180 and contrast < 40:
                return SceneType.ESTABLISHING
            if contrast > 80:
                return SceneType.CLOSEUP
            if brightness < 60:
                return SceneType.DETAIL
            if motion > 0.3:
                return SceneType.MONTAGE
            if contrast < 50:
                return SceneType.WIDE
            return SceneType.B_ROLL
        except (subprocess.SubprocessError, FileNotFoundError):
            return SceneType.B_ROLL

    def classify_batch(self, video_path: str, shots: list[Shot]) -> list[SceneType]:
        return [self.classify(video_path, shot) for shot in shots]

    def _get_frame_stats(self, video_path: str, time: float) -> Optional[dict]:
        try:
            cmd = [self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                   "-vframes", "1", "-vf", "signalstats", "-f", "null", "-"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            stats = {}
            for line in result.stderr.split("\n"):
                if "signalstats" in line:
                    for key, pat in [("y_min", r'YMIN=(\d+)'), ("y_low", r'YLOW=(\d+)'),
                                      ("average_brightness", r'YAVG=(\d+)'), ("y_high", r'YHIGH=(\d+)'),
                                      ("y_max", r'YMAX=(\d+)')]:
                        m = re.search(pat, line)
                        if m: stats[key] = int(m.group(1))
            if "average_brightness" in stats:
                stats["contrast"] = stats.get("y_max", 255) - stats.get("y_min", 0)
                return stats
            return None
        except (subprocess.SubprocessError, FileNotFoundError):
            return None

    def _estimate_motion(self, video_path: str, shot: Shot, samples: int = 5) -> float:
        if shot.duration < 0.5:
            return 0.0
        try:
            step = shot.duration / (samples + 1)
            prev_hist = None
            total_diff = 0.0
            count = 0
            for i in range(1, samples + 1):
                t = shot.start_time + step * i
                hist = self._get_frame_histogram(video_path, t)
                if hist is not None and prev_hist is not None:
                    total_diff += sum(abs(a - b) for a, b in zip(hist, prev_hist))
                    count += 1
                prev_hist = hist
            if count == 0:
                return 0.0
            return min(1.0, (total_diff / count) / 500.0)
        except (subprocess.SubprocessError, FileNotFoundError):
            return 0.0

    def _get_frame_histogram(self, video_path: str, time: float) -> Optional[list[int]]:
        try:
            cmd = [self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                   "-vframes", "1", "-vf", "histogram", "-f", "null", "-"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            for line in result.stderr.split("\n"):
                if "Parsed_histogram" in line:
                    numbers = re.findall(r'\d+', line)
                    return [int(n) for n in numbers[:256]]
            return None
        except (subprocess.SubprocessError, FileNotFoundError):
            return None

    def estimate_tokens(self) -> int:
        return 0
