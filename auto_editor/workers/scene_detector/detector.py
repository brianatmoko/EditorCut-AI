"""Scene detection using PySceneDetect (primary) + FFmpeg (fallback).

PySceneDetect memberikan deteksi yang jauh lebih akurat dan cepat
dibanding FFmpeg scene filter mentah — terutama untuk fade/dissolve.

Backend:
  Primary  : PySceneDetect (scenedetect library)
  Secondary: FFmpeg select filter (original implementation)
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Optional
import subprocess
import json
import re
import tempfile
import os
import logging

logger = logging.getLogger(__name__)


@dataclass
class Shot:
    index: int
    start_time: float
    end_time: float
    duration: float
    confidence: float = 1.0


class SceneDetector:
    """Detect scene cuts in video.

    Gunakan PySceneDetect bila tersedia (lebih akurat, support fade/dissolve).
    Fallback ke FFmpeg scene filter bila tidak terinstall.
    """

    def __init__(self, threshold: float = 0.3, ffmpeg_path: str = "ffmpeg"):
        self.threshold = threshold
        self.ffmpeg_path = ffmpeg_path
        self._pyscenedetect_available = self._check_pyscenedetect()

    def _check_pyscenedetect(self) -> bool:
        try:
            import scenedetect  # noqa: F401
            logger.info("[SceneDetector] PySceneDetect backend: ACTIVE")
            return True
        except ImportError:
            logger.info("[SceneDetector] PySceneDetect not found, using FFmpeg fallback")
            return False

    def detect(self, video_path: str) -> list[Shot]:
        if not os.path.exists(video_path):
            return []

        if self._pyscenedetect_available:
            shots = self._detect_pyscenedetect(video_path)
            if shots:
                return shots
            logger.warning("[SceneDetector] PySceneDetect returned empty, falling back to FFmpeg")

        return self._detect_ffmpeg(video_path)

    # ── PySceneDetect Backend ─────────────────────────────────────────────────

    def _detect_pyscenedetect(self, video_path: str) -> list[Shot]:
        """Detect scenes using PySceneDetect with ContentDetector + AdaptiveDetector."""
        try:
            from scenedetect import open_video, SceneManager
            from scenedetect.detectors import ContentDetector, AdaptiveDetector

            video = open_video(video_path)
            scene_manager = SceneManager()

            # ContentDetector: hard cuts (fast)
            scene_manager.add_detector(
                ContentDetector(threshold=self.threshold * 100)  # pyscenedetect uses 0–100 scale
            )

            # AdaptiveDetector: fades and dissolves
            scene_manager.add_detector(
                AdaptiveDetector(adaptive_threshold=3.0)
            )

            scene_manager.detect_scenes(video, show_progress=False)
            scene_list = scene_manager.get_scene_list()

            shots = []
            for i, (start, end) in enumerate(scene_list):
                start_sec = start.get_seconds()
                end_sec = end.get_seconds()
                shots.append(Shot(
                    index=i,
                    start_time=round(start_sec, 3),
                    end_time=round(end_sec, 3),
                    duration=round(end_sec - start_sec, 3),
                    confidence=0.95,
                ))

            logger.info("[SceneDetector] PySceneDetect found %d scenes in '%s'",
                        len(shots), Path(video_path).name)
            return shots

        except Exception as e:
            logger.warning("[SceneDetector] PySceneDetect error: %s", e)
            return []

    def split_video(self, video_path: str, output_dir: Optional[str] = None) -> list[str]:
        """Split video berdasarkan scene boundaries. Menggunakan stream copy (no re-encode)."""
        if not self._pyscenedetect_available:
            logger.warning("[SceneDetector] split_video requires PySceneDetect")
            return []

        try:
            from scenedetect import open_video, SceneManager, split_video_ffmpeg
            from scenedetect.detectors import ContentDetector

            out_dir = Path(output_dir) if output_dir else Path(video_path).parent / "scenes"
            out_dir.mkdir(parents=True, exist_ok=True)

            video = open_video(video_path)
            scene_manager = SceneManager()
            scene_manager.add_detector(ContentDetector(threshold=self.threshold * 100))
            scene_manager.detect_scenes(video, show_progress=False)
            scene_list = scene_manager.get_scene_list()

            if not scene_list:
                return [video_path]

            split_video_ffmpeg(
                video_path,
                scene_list,
                output_dir=str(out_dir),
                show_progress=False,
            )

            output_files = sorted(out_dir.glob(f"{Path(video_path).stem}-Scene-*.mp4"))
            logger.info("[SceneDetector] Split into %d scene files", len(output_files))
            return [str(f) for f in output_files]

        except Exception as e:
            logger.error("[SceneDetector] split_video failed: %s", e)
            return [video_path]

    # ── FFmpeg Fallback Backend ───────────────────────────────────────────────

    def _detect_ffmpeg(self, video_path: str) -> list[Shot]:
        """Original FFmpeg-based scene detection (fallback)."""
        try:
            duration = self._get_duration(video_path)
            if duration is None or duration == 0:
                return []
            scene_times = self._run_ffmpeg_scene(video_path)
            shots = []
            prev_time = 0.0
            for i, scene_time in enumerate(scene_times):
                shots.append(Shot(
                    index=i,
                    start_time=prev_time,
                    end_time=scene_time,
                    duration=scene_time - prev_time,
                    confidence=self.threshold,
                ))
                prev_time = scene_time
            shots.append(Shot(
                index=len(shots),
                start_time=prev_time,
                end_time=duration,
                duration=duration - prev_time,
                confidence=self.threshold,
            ))
            return shots
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_detect(video_path)

    def _get_duration(self, video_path: str) -> Optional[float]:
        try:
            cmd = ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "json", video_path]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return float(json.loads(result.stdout)["format"]["duration"])
        except (subprocess.SubprocessError, json.JSONDecodeError, KeyError, ValueError):
            return None

    def _run_ffmpeg_scene(self, video_path: str) -> list[float]:
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            log_path = f.name
        try:
            cmd = [self.ffmpeg_path, "-i", video_path,
                   "-filter:v", f"select='gt(scene,{self.threshold})',showinfo", "-f", "null", "-"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            scene_times = []
            for line in result.stderr.split("\n"):
                match = re.search(r'pts_time:(\d+\.?\d*)', line)
                if match:
                    scene_times.append(float(match.group(1)))
            return scene_times
        except subprocess.SubprocessError:
            return []
        finally:
            if os.path.exists(log_path):
                os.unlink(log_path)

    def _fallback_detect(self, video_path: str) -> list[Shot]:
        duration = self._get_duration(video_path)
        if duration is None:
            duration = 30.0
        return [Shot(index=0, start_time=0.0, end_time=duration, duration=duration, confidence=0.5)]

    # ── Utilities ─────────────────────────────────────────────────────────────

    def detect_with_thumbnails(self, video_path: str) -> list[dict]:
        shots = self.detect(video_path)
        result = []
        for shot in shots:
            thumb = self._extract_thumbnail(video_path, shot.start_time)
            result.append({"shot": shot, "thumbnail_path": thumb})
        return result

    def _extract_thumbnail(self, video_path: str, time: float) -> Optional[str]:
        thumb_dir = Path(video_path).parent / ".thumbnails"
        thumb_dir.mkdir(exist_ok=True)
        thumb_path = str(thumb_dir / f"thumb_{Path(video_path).stem}_{int(time)}.jpg")
        try:
            cmd = [self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                   "-vframes", "1", "-q:v", "2", thumb_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=30, check=True)
            return thumb_path if os.path.exists(thumb_path) else None
        except subprocess.SubprocessError:
            return None

    def estimate_tokens(self, video_path: str) -> int:
        return 0
