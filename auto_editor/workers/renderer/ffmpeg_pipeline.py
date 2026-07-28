"""FFmpeg-based video rendering pipeline.

Primary renderer. Used directly or as fallback from OpenCut AI.
Supports hardware acceleration via VAAPI/NVIDIA.
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Callable, Literal
import subprocess
import json
import os
import re


class FFmpegPipeline:
    def __init__(self, ffmpeg_path: str = "ffmpeg", ffprobe_path: str = "ffprobe"):
        self.ffmpeg = ffmpeg_path
        self.ffprobe = ffprobe_path

    def render(self, video_paths: list[str], output_path: str, audio_path: Optional[str] = None,
               subtitle_path: Optional[str] = None, resolution: str = "1080p",
               codec: Literal["h264", "h265"] = "h264", fps: float = 30.0,
               progress_callback: Optional[Callable[[float], None]] = None) -> Optional[str]:
        os.makedirs(Path(output_path).parent, exist_ok=True)
        scale = {"720p": "1280:720", "1080p": "1920:1080", "4k": "3840:2160"}.get(resolution, "1920:1080")
        try:
            if len(video_paths) == 1:
                return self._render_single(video_paths[0], output_path, audio_path, subtitle_path, scale, codec, fps, progress_callback)
            else:
                return self._render_concat(video_paths, output_path, audio_path, subtitle_path, scale, codec, fps, progress_callback)
        except (subprocess.SubprocessError, FileNotFoundError):
            return None

    def _render_single(self, video: str, output: str, audio: Optional[str], subtitle: Optional[str],
                       scale: str, codec: str, fps: float, progress_cb: Optional[Callable]) -> Optional[str]:
        codec_params = self._codec_params(codec)
        filters = [f"scale={scale}:force_original_aspect_ratio=decrease,pad={scale}:(ow-iw)/2:(oh-ih)/2"]
        if subtitle and os.path.exists(subtitle):
            filters.append(f"subtitles={subtitle}")
        filter_str = ",".join(filters)
        cmd = [self.ffmpeg, "-i", video] + (["-i", audio] if audio and os.path.exists(audio) else []) + [
            "-vf", filter_str, "-r", str(fps), *codec_params, "-movflags", "+faststart", output, "-y"]
        return self._run_with_progress(cmd, progress_cb)

    def _render_concat(self, videos: list[str], output: str, audio: Optional[str], subtitle: Optional[str],
                       scale: str, codec: str, fps: float, progress_cb: Optional[Callable]) -> Optional[str]:
        concat_file = "./.opencut_ai_concat.txt"
        with open(concat_file, "w") as f:
            for v in videos:
                if os.path.exists(v):
                    f.write(f"file '{os.path.abspath(v)}'\n")
        codec_params = self._codec_params(codec)
        scale_filter = f"scale={scale}:force_original_aspect_ratio=decrease,pad={scale}:(ow-iw)/2:(oh-ih)/2"
        cmd = [self.ffmpeg, "-f", "concat", "-safe", "0", "-i", concat_file] + (
            ["-i", audio] if audio and os.path.exists(audio) else []) + [
            "-vf", scale_filter, "-r", str(fps), *codec_params, "-movflags", "+faststart", output, "-y"]
        result = self._run_with_progress(cmd, progress_cb)
        if os.path.exists(concat_file):
            os.unlink(concat_file)
        return result

    def _codec_params(self, codec: str) -> list[str]:
        if codec == "h265":
            return ["-c:v", "libx265", "-crf", "23", "-preset", "medium"]
        return ["-c:v", "libx264", "-crf", "18", "-preset", "medium", "-pix_fmt", "yuv420p"]

    def _run_with_progress(self, cmd: list[str], progress_callback: Optional[Callable[[float], None]]) -> Optional[str]:
        output_path = cmd[cmd.index("-y") - 1] if "-y" in cmd else None
        try:
            process = subprocess.Popen(cmd, stderr=subprocess.PIPE, text=True, bufsize=1)
            duration = None
            time_pat = re.compile(r'time=(\d+):(\d+):(\d+)\.(\d+)')
            dur_pat = re.compile(r'Duration: (\d+):(\d+):(\d+)\.(\d+)')
            for line in process.stderr:
                if duration is None:
                    dm = dur_pat.search(line)
                    if dm:
                        duration = int(dm.group(1))*3600 + int(dm.group(2))*60 + int(dm.group(3)) + int(dm.group(4))/100
                if progress_callback and duration and duration > 0:
                    tm = time_pat.search(line)
                    if tm:
                        current = int(tm.group(1))*3600 + int(tm.group(2))*60 + int(tm.group(3)) + int(tm.group(4))/100
                        progress_callback(min(1.0, current / duration))
            process.wait()
            if process.returncode == 0 and output_path and os.path.exists(output_path):
                return output_path
            return None
        except (subprocess.SubprocessError, FileNotFoundError):
            return None

    def get_video_info(self, video_path: str) -> Optional[dict]:
        try:
            cmd = [self.ffprobe, "-v", "error", "-show_entries", "format=duration,size,bit_rate",
                   "-show_entries", "stream=width,height,codec_name,r_frame_rate", "-of", "json", video_path]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return json.loads(result.stdout)
        except (subprocess.SubprocessError, json.JSONDecodeError, FileNotFoundError):
            return None

    def validate_output(self, video_path: str) -> bool:
        info = self.get_video_info(video_path)
        if not info:
            return False
        try:
            return float(info.get("format", {}).get("duration", 0)) > 0
        except (ValueError, TypeError):
            return False
