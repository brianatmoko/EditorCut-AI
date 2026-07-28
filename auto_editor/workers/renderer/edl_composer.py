"""EDLComposer — Render Edit Decision List to MP4 using FFmpeg.

Cara kerja:
  1. Terima EDL (Edit Decision List) dari DirectorLLM
  2. Resolve aset (download dari Pexels/Pixabay atau buat procedural)
  3. Build FFmpeg filtergraph dari scene definitions
  4. Render ke MP4 dengan color grading, teks overlay, dan transisi
  5. Mix voiceover + background music

Output: file MP4 siap play di browser.
"""

from __future__ import annotations

import logging
import os
import subprocess
import re
import time
import urllib.request
import hashlib
from pathlib import Path
from typing import Optional, Callable

logger = logging.getLogger(__name__)

# ── Resolution mapping ────────────────────────────────────────────────────────
ASPECT_RESOLUTION = {
    "16:9": (1920, 1080),
    "9:16": (1080, 1920),
    "1:1": (1080, 1080),
    "4:3": (1440, 1080),
    "21:9": (2560, 1080),
}

# ── Color grading filtergraph presets ─────────────────────────────────────────
COLOR_GRADE_FILTERS = {
    "warm":     "colorbalance=rs=0.1:gs=0.05:bs=-0.05:rm=0.05:gm=0.02:bm=-0.05:rh=0.05:gh=0.02:bh=-0.05",
    "cool":     "colorbalance=rs=-0.05:gs=0:bs=0.1:rm=-0.05:gm=0.02:bm=0.1:rh=-0.05:gh=0.02:bh=0.1",
    "dramatic": "curves=r='0/0 0.5/0.4 1/1':g='0/0 0.5/0.45 1/1':b='0/0 0.5/0.35 1/1',vignette",
    "vintage":  "curves=r='0/0 0.5/0.5 1/0.95':g='0/0.05 0.5/0.5 1/0.9':b='0/0.1 0.5/0.45 1/0.85'",
    "neutral":  "",  # no adjustment
    "none":     "",
}

# ── Effect filtergraph snippets ───────────────────────────────────────────────
EFFECT_FILTERS = {
    "vignette":        "vignette=PI/4",
    "film_grain":      "noise=alls=5:allf=t",
    "blur_bg":         "boxblur=2:1",
    "brightness_boost": "eq=brightness=0.05:saturation=1.2",
    "zoom_slow":       "zoompan=z='min(zoom+0.0005,1.1)':d=125:x='iw/2-(iw/zoom/2)':y='ih/2-(ih/zoom/2)'",
    "pan_left":        "crop=iw-100:ih:100*t/5:0",
    "pan_right":       "crop=iw-100:ih:0:0",
}


class AssetResolver:
    """Download and cache video/image assets for each scene."""

    def __init__(self, cache_dir: str = ".asset_cache"):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.pexels_key = self._resolve_key("pexels_api_key", "PEXELS_API_KEY")
        self.pixabay_key = self._resolve_key("pixabay_api_key", "PIXABAY_API_KEY")

    @staticmethod
    def _resolve_key(config_key: str, env_key: str) -> str:
        key = os.environ.get(env_key, "")
        if not key:
            try:
                from auto_editor.config.opencut_settings import OpenCutConfig
                cfg = OpenCutConfig.get()
                key = getattr(cfg, config_key, "")
            except Exception:
                pass
        return key or ""

    def resolve_scene_asset(
        self,
        scene_id: int,
        query: str,
        asset_type: str,
        scene_duration: float,
        width: int,
        height: int,
    ) -> Optional[str]:
        """Download an asset for a scene and return local path.

        Falls back to procedural color card if no API keys or no results.
        """
        cache_key = hashlib.md5(f"{query}:{asset_type}:{width}:{height}".encode()).hexdigest()
        cached = self._check_cache(cache_key, asset_type)
        if cached:
            logger.info("[AssetResolver] Scene %d: cache hit for '%s'", scene_id, query)
            return cached

        # Try Pexels
        if self.pexels_key and asset_type == "video":
            path = self._download_pexels_video(scene_id, query, cache_key, scene_duration)
            if path:
                return path

        # Try Pixabay
        if self.pixabay_key:
            path = self._download_pixabay(scene_id, query, asset_type, cache_key, scene_duration)
            if path:
                return path

        # Fallback: generate procedural color card using FFmpeg
        logger.info("[AssetResolver] Scene %d: no external asset, generating procedural", scene_id)
        return self._generate_procedural(scene_id, query, scene_duration, width, height, cache_key)

    def _check_cache(self, cache_key: str, asset_type: str) -> Optional[str]:
        ext = "mp4" if asset_type == "video" else "jpg"
        for path in self.cache_dir.glob(f"{cache_key}*.{ext}"):
            if path.exists() and path.stat().st_size > 1024:
                return str(path)
        return None

    def _download_pexels_video(
        self, scene_id: int, query: str, cache_key: str, duration: float
    ) -> Optional[str]:
        try:
            import json
            url = f"https://api.pexels.com/videos/search?query={query}&per_page=5&min_duration=3"
            req = urllib.request.Request(url, headers={"Authorization": self.pexels_key})
            with urllib.request.urlopen(req, timeout=10) as resp:
                data = json.loads(resp.read())

            videos = data.get("videos", [])
            if not videos:
                return None

            # Pick best quality video file
            for video in videos:
                files = sorted(
                    video.get("video_files", []),
                    key=lambda f: f.get("width", 0) * f.get("height", 0),
                    reverse=True,
                )
                for vf in files:
                    if vf.get("file_type") == "video/mp4" and vf.get("width", 0) >= 720:
                        dl_url = vf["link"]
                        out_path = self.cache_dir / f"{cache_key}_pexels.mp4"
                        urllib.request.urlretrieve(dl_url, out_path)
                        if out_path.stat().st_size > 10240:
                            logger.info("[Pexels] Scene %d: downloaded '%s'", scene_id, query)
                            return str(out_path)
        except Exception as e:
            logger.warning("[Pexels] Scene %d failed: %s", scene_id, e)
        return None

    def _download_pixabay(
        self, scene_id: int, query: str, asset_type: str, cache_key: str, duration: float
    ) -> Optional[str]:
        try:
            import json
            endpoint = "videos" if asset_type == "video" else "photos"
            url = f"https://pixabay.com/api/{endpoint}/?key={self.pixabay_key}&q={query}&per_page=5"
            with urllib.request.urlopen(url, timeout=10) as resp:
                data = json.loads(resp.read())

            hits = data.get("hits", [])
            if not hits:
                return None

            hit = hits[0]
            if asset_type == "video":
                videos = hit.get("videos", {})
                vdata = videos.get("large", videos.get("medium", videos.get("small", {})))
                dl_url = vdata.get("url")
                ext = "mp4"
            else:
                dl_url = hit.get("largeImageURL")
                ext = "jpg"

            if not dl_url:
                return None

            out_path = self.cache_dir / f"{cache_key}_pixabay.{ext}"
            urllib.request.urlretrieve(dl_url, out_path)
            if out_path.stat().st_size > 1024:
                logger.info("[Pixabay] Scene %d: downloaded '%s'", scene_id, query)
                return str(out_path)
        except Exception as e:
            logger.warning("[Pixabay] Scene %d failed: %s", scene_id, e)
        return None

    def _generate_procedural(
        self,
        scene_id: int,
        query: str,
        duration: float,
        width: int,
        height: int,
        cache_key: str,
    ) -> str:
        """Generate a procedural color gradient video using FFmpeg."""
        out_path = self.cache_dir / f"{cache_key}_proc.mp4"
        # Generate rich gradient colors based on scene ID
        colors = [
            ("0x1a1a2e", "0x16213e"),  # dark blue
            ("0x0f3460", "0x533483"),  # indigo purple
            ("0x2d6a4f", "0x1b4332"),  # forest green
            ("0x7b2d00", "0x411900"),  # burnt orange
            ("0x2c1654", "0x150e42"),  # deep purple
            ("0x1a3c34", "0x0d1f1b"),  # teal dark
        ]
        c1, c2 = colors[scene_id % len(colors)]

        # Use FFmpeg to create a gradient video with animated zoom
        cmd = [
            "ffmpeg",
            "-f", "lavfi",
            "-i", (
                f"gradients=s={width}x{height}:duration={duration}:speed=0.3:"
                f"c0={c1}:c1={c2}:type=linear:angle=45"
            ),
            "-t", str(duration),
            "-r", "30",
            "-c:v", "libx264", "-crf", "23", "-preset", "fast", "-pix_fmt", "yuv420p",
            str(out_path), "-y",
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, timeout=30)
            if result.returncode == 0 and out_path.exists():
                return str(out_path)
        except Exception:
            pass

        # Simpler fallback: color=c
        cmd2 = [
            "ffmpeg",
            "-f", "lavfi",
            "-i", f"color=c={c1.replace('0x', '#')}:size={width}x{height}:duration={duration}:rate=30",
            "-t", str(duration),
            "-c:v", "libx264", "-crf", "23", "-preset", "ultrafast", "-pix_fmt", "yuv420p",
            str(out_path), "-y",
        ]
        try:
            subprocess.run(cmd2, capture_output=True, timeout=30)
        except Exception as e:
            logger.error("[Procedural] Failed to generate: %s", e)

        return str(out_path)


class EDLComposer:
    """Render an Edit Decision List to a final MP4 using FFmpeg.

    Pipeline per scene:
      1. Trim/loop source clip to exact duration
      2. Scale to target resolution
      3. Apply color grade filtergraph
      4. Apply effects (vignette, zoom, etc.)
      5. Overlay text if specified
      6. Concat all scenes
      7. Mix voiceover audio (edge-tts if available, else silent)
      8. Encode final MP4
    """

    def __init__(
        self,
        output_dir: str = "./output",
        cache_dir: str = ".asset_cache",
        ffmpeg: str = "ffmpeg",
        ffprobe: str = "ffprobe",
    ):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.ffmpeg = ffmpeg
        self.ffprobe = ffprobe
        self.resolver = AssetResolver(cache_dir)

    def compose(
        self,
        edl: "EditDecisionList",
        output_filename: Optional[str] = None,
        progress_callback: Optional[Callable[[str, float], None]] = None,
    ) -> str:
        """Compose an EditDecisionList into a final MP4.

        Args:
            edl: The edit decision list from DirectorLLM
            output_filename: Output filename (auto-generated if None)
            progress_callback: Called with (stage_name, 0.0-1.0)

        Returns:
            Path to the rendered MP4 file
        """
        from auto_editor.orchestrator.director_llm import EditDecisionList

        if output_filename is None:
            safe_title = re.sub(r'[^\w\s-]', '', edl.title).strip().replace(' ', '_')[:40]
            timestamp = int(time.time())
            output_filename = f"{safe_title}_{timestamp}.mp4"

        output_path = self.output_dir / output_filename
        width, height = ASPECT_RESOLUTION.get(edl.aspect_ratio, (1920, 1080))

        logger.info("[EDLComposer] Starting composition: %s (%dx%d, %d scenes)",
                    edl.title, width, height, len(edl.scenes))

        # Stage 1: Resolve all assets
        self._progress(progress_callback, "resolving_assets", 0.0)
        scene_clips = self._resolve_assets(edl, width, height, progress_callback)

        # Stage 2: Process each scene clip (color grade, effects, text)
        self._progress(progress_callback, "processing_scenes", 0.3)
        processed_clips = self._process_scenes(edl, scene_clips, width, height, progress_callback)

        # Stage 3: Concatenate scenes
        self._progress(progress_callback, "concatenating", 0.6)
        concat_path = self.output_dir / f"_concat_{int(time.time())}.mp4"
        self._concatenate(processed_clips, concat_path, edl.fps)

        # Stage 4: Generate voiceover audio
        self._progress(progress_callback, "generating_audio", 0.75)
        voiceover_path = self._generate_voiceover(edl, width, height)

        # Stage 5: Final mix (video + audio)
        self._progress(progress_callback, "final_mix", 0.85)
        self._final_mix(str(concat_path), voiceover_path, str(output_path), edl)

        # Cleanup temp files
        for clip in processed_clips:
            try:
                if "_scene_proc_" in str(clip) or "_scene_trim_" in str(clip):
                    Path(clip).unlink(missing_ok=True)
            except Exception:
                pass
        concat_path.unlink(missing_ok=True)

        self._progress(progress_callback, "complete", 1.0)
        logger.info("[EDLComposer] Done: %s", output_path)
        return str(output_path)

    def _progress(self, cb, stage: str, progress: float):
        if cb:
            try:
                cb(stage, progress)
            except Exception:
                pass

    # ── Asset Resolution ──────────────────────────────────────────────────────

    def _resolve_assets(
        self,
        edl: "EditDecisionList",
        width: int,
        height: int,
        progress_callback,
    ) -> list[str]:
        """Download/generate assets for each scene."""
        clips = []
        for i, scene in enumerate(edl.scenes):
            self._progress(progress_callback, "resolving_assets",
                           i / len(edl.scenes) * 0.3)
            path = self.resolver.resolve_scene_asset(
                scene_id=scene.id,
                query=scene.asset_query,
                asset_type=scene.asset_type,
                scene_duration=scene.duration,
                width=width,
                height=height,
            )
            clips.append(path or "")
            scene.asset_path = path
        return clips

    # ── Scene Processing ──────────────────────────────────────────────────────

    def _process_scenes(
        self,
        edl: "EditDecisionList",
        clips: list[str],
        width: int,
        height: int,
        progress_callback,
    ) -> list[str]:
        """Apply trim, color grade, effects, and text overlay to each scene."""
        processed = []
        total = len(edl.scenes)

        for i, (scene, clip_path) in enumerate(zip(edl.scenes, clips)):
            self._progress(progress_callback, "processing_scenes",
                           0.3 + (i / total) * 0.3)

            out_path = str(self.output_dir / f"_scene_proc_{scene.id}_{int(time.time())}.mp4")
            ok = self._process_single_scene(
                clip_path, out_path, scene, width, height, edl.fps,
            )
            processed.append(out_path if ok else clip_path)

        return processed

    def _process_single_scene(
        self,
        clip_path: str,
        out_path: str,
        scene,
        width: int,
        height: int,
        fps: int,
    ) -> bool:
        """Process one scene: trim → scale → color grade → effects → text."""
        # Build filter chain
        filters = []

        # Scale to target resolution (crop to fill)
        filters.append(
            f"scale={width}:{height}:force_original_aspect_ratio=increase,"
            f"crop={width}:{height}"
        )

        # Color grading
        grade = COLOR_GRADE_FILTERS.get(scene.color_grade, "")
        if grade:
            filters.append(grade)

        # Effects
        for effect in scene.effects:
            ef = EFFECT_FILTERS.get(effect, "")
            if ef:
                filters.append(ef)

        # Text overlay
        if scene.text_overlay:
            safe_text = scene.text_overlay.replace("'", "\\'").replace(":", "\\:")
            filters.append(
                f"drawtext=text='{safe_text}'"
                f":fontsize={int(height * 0.06)}"
                f":fontcolor=white@0.9"
                f":borderw=3:bordercolor=black@0.6"
                f":x=(w-text_w)/2:y=h*0.8"
                f":enable='between(t,0.5,{scene.duration - 0.5})'"
            )

        filter_str = ",".join(filters) if filters else "null"

        # If clip doesn't exist, generate procedural
        if not clip_path or not Path(clip_path).exists():
            clip_path = str(self.resolver._generate_procedural(
                scene.id, scene.asset_query, scene.duration, width, height,
                hashlib.md5(scene.asset_query.encode()).hexdigest()
            ))

        cmd = [
            self.ffmpeg,
            "-ss", "0",
            "-t", str(scene.duration),
            "-i", clip_path,
            "-vf", filter_str,
            "-r", str(fps),
            "-c:v", "libx264", "-crf", "20", "-preset", "fast", "-pix_fmt", "yuv420p",
            "-an",  # no audio in clip — audio added in final mix
            out_path, "-y",
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, timeout=120)
            return result.returncode == 0 and Path(out_path).exists()
        except Exception as e:
            logger.warning("[EDLComposer] Scene %d processing failed: %s", scene.id, e)
            return False

    # ── Concatenation ─────────────────────────────────────────────────────────

    def _concatenate(self, clips: list[str], output: Path, fps: int) -> None:
        """Concatenate all scene clips into one video."""
        concat_list = self.output_dir / "_concat_list.txt"
        valid_clips = [c for c in clips if c and Path(c).exists()]

        if not valid_clips:
            logger.error("[EDLComposer] No valid clips to concatenate!")
            return

        with open(concat_list, "w") as f:
            for clip in valid_clips:
                f.write(f"file '{Path(clip).resolve()}'\n")

        cmd = [
            self.ffmpeg,
            "-f", "concat", "-safe", "0",
            "-i", str(concat_list),
            "-c:v", "libx264", "-crf", "18", "-preset", "medium", "-pix_fmt", "yuv420p",
            "-r", str(fps),
            str(output), "-y",
        ]

        try:
            subprocess.run(cmd, capture_output=True, timeout=300)
        except Exception as e:
            logger.error("[EDLComposer] Concatenation failed: %s", e)
        finally:
            concat_list.unlink(missing_ok=True)

    # ── Voiceover Generation ──────────────────────────────────────────────────

    def _generate_voiceover(self, edl: "EditDecisionList", width: int, height: int) -> Optional[str]:
        """Generate voiceover audio using edge-tts (offline) or gTTS."""
        if not edl.voiceover_script.strip():
            return None

        vo_path = str(self.output_dir / f"_voiceover_{int(time.time())}.mp3")

        # Try edge-tts (offline, high quality)
        try:
            import asyncio
            import edge_tts

            async def _tts():
                voice = "id-ID-GadisNeural" if "id" in edl.source_prompt[:50].lower() else "en-US-JennyNeural"
                communicate = edge_tts.Communicate(edl.voiceover_script, voice)
                await communicate.save(vo_path)

            asyncio.run(_tts())
            if Path(vo_path).exists() and Path(vo_path).stat().st_size > 100:
                logger.info("[EDLComposer] Voiceover generated via edge-tts")
                return vo_path
        except ImportError:
            logger.debug("[EDLComposer] edge-tts not installed, trying gTTS")
        except Exception as e:
            logger.warning("[EDLComposer] edge-tts failed: %s", e)

        # Try gTTS
        try:
            from gtts import gTTS
            lang = "id"
            tts = gTTS(edl.voiceover_script, lang=lang)
            tts.save(vo_path)
            if Path(vo_path).exists() and Path(vo_path).stat().st_size > 100:
                logger.info("[EDLComposer] Voiceover generated via gTTS")
                return vo_path
        except ImportError:
            logger.debug("[EDLComposer] gTTS not installed")
        except Exception as e:
            logger.warning("[EDLComposer] gTTS failed: %s", e)

        logger.info("[EDLComposer] No TTS engine available, video will be silent")
        return None

    # ── Final Mix ─────────────────────────────────────────────────────────────

    def _final_mix(
        self,
        video_path: str,
        audio_path: Optional[str],
        output_path: str,
        edl: "EditDecisionList",
    ) -> None:
        """Mix video + voiceover audio into final output."""
        if audio_path and Path(audio_path).exists():
            cmd = [
                self.ffmpeg,
                "-i", video_path,
                "-i", audio_path,
                "-c:v", "copy",
                "-c:a", "aac", "-b:a", "128k",
                "-shortest",
                "-movflags", "+faststart",
                output_path, "-y",
            ]
        else:
            cmd = [
                self.ffmpeg,
                "-i", video_path,
                "-c:v", "copy",
                "-an",  # no audio
                "-movflags", "+faststart",
                output_path, "-y",
            ]

        try:
            result = subprocess.run(cmd, capture_output=True, timeout=300)
            if result.returncode != 0:
                logger.error("[EDLComposer] Final mix error: %s", result.stderr.decode()[-500:])
                # Fallback: just copy video
                import shutil
                shutil.copy2(video_path, output_path)
        except Exception as e:
            logger.error("[EDLComposer] Final mix failed: %s", e)
            import shutil
            shutil.copy2(video_path, output_path)

        # Cleanup voiceover
        if audio_path:
            try:
                Path(audio_path).unlink(missing_ok=True)
            except Exception:
                pass

    def check_ffmpeg(self) -> bool:
        """Check if FFmpeg is available."""
        try:
            result = subprocess.run(
                [self.ffmpeg, "-version"],
                capture_output=True, timeout=5,
            )
            return result.returncode == 0
        except Exception:
            return False


import hashlib
