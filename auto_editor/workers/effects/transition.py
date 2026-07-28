"""Smart scene transitions — FFmpeg xfade filter (primary) + Movis (advanced).

Menyediakan transisi cinematic berkualitas tinggi antara scene.

Transisi yang didukung:
  FFmpeg xfade: fade, dissolve, wipeleft, wiperight, slideup, slidedown,
                circleopen, radial, pixelize, squeezev, diagtl, hlwind
  Movis        : Keyframe-based custom animations (jika terinstall)

Semua transisi tersedia TANPA API key — pure FFmpeg.
"""

from __future__ import annotations

from typing import Optional, Literal
import subprocess
import os
import logging

from ...models import TransitionType, Scene

logger = logging.getLogger(__name__)

# FFmpeg xfade transition presets — tersedia sejak FFmpeg 4.3
XFADE_PRESETS = {
    TransitionType.CROSSFADE:     "fade",
    TransitionType.DIP_TO_BLACK:  "fadeblack",
    TransitionType.FADE_IN:       "fade",
    TransitionType.FADE_OUT:      "fade",
    TransitionType.SLIDE:         "slideleft",
    # Extended types (nilai baru dari enum akan ditangani di suggest_transition)
    "dissolve":   "dissolve",
    "wipeleft":   "wipeleft",
    "wiperight":  "wiperight",
    "slideup":    "slideup",
    "slidedown":  "slidedown",
    "circleopen": "circleopen",
    "radial":     "radial",
    "pixelize":   "pixelize",
    "diagtl":     "diagtl",
}


class TransitionEngine:
    """Rule-based transition selector + FFmpeg xfade renderer."""

    def suggest_transition(self, scene_a: Scene, scene_b: Scene) -> TransitionType:
        """Pilih jenis transisi berdasarkan konten scene."""
        if scene_a.scene_type == scene_b.scene_type:
            return TransitionType.HARD_CUT

        a_type = getattr(scene_a.scene_type, "value", "")
        b_type = getattr(scene_b.scene_type, "value", "")

        # Wide/establishing → fade untuk cinematic feel
        if a_type in ("establishing", "wide") or b_type in ("establishing", "wide"):
            return TransitionType.CROSSFADE

        # Montage → fast wipe
        if a_type == "montage" or b_type == "montage":
            return TransitionType.SLIDE

        # Closeup → dissolve untuk intimacy
        if a_type == "closeup":
            return TransitionType.CROSSFADE

        # Default: clean crossfade
        return TransitionType.CROSSFADE

    def suggest_transition_name(self, scene_a: Scene, scene_b: Scene) -> str:
        """Suggest transisi sebagai string xfade name."""
        a_type = getattr(scene_a.scene_type, "value", "")
        b_type = getattr(scene_b.scene_type, "value", "")

        mapping = {
            "establishing": "fade",
            "wide": "dissolve",
            "closeup": "fadeblack",
            "montage": "wipeleft",
            "detail": "radial",
        }
        return mapping.get(a_type, mapping.get(b_type, "fade"))

    def generate_filter(self, transition: TransitionType, duration: float = 0.5) -> str:
        """Generate FFmpeg filter string untuk transisi (legacy API)."""
        transitions = {
            TransitionType.CROSSFADE: f"fade=t=cross:f=128:d={duration}",
            TransitionType.DIP_TO_BLACK: f"fade=t=out:st=0:d={duration/2},fade=t=in:st={duration/2}:d={duration/2}",
            TransitionType.FADE_IN: f"fade=t=in:st=0:d={duration}",
            TransitionType.FADE_OUT: f"fade=t=out:st=0:d={duration}",
            TransitionType.SLIDE: f"slide=w>{'if(gt(t,0),1,0)'}:d={duration}",
        }
        return transitions.get(transition, "")

    def apply_transition(
        self,
        input_a: str,
        input_b: str,
        output_path: str,
        transition: TransitionType,
        duration: float = 0.5,
    ) -> Optional[str]:
        """Terapkan transisi antara dua klip menggunakan FFmpeg xfade.

        FFmpeg xfade jauh lebih akurat dibanding filter lama.
        Input kedua klip di-encode bersama dengan xfade filter.
        """
        xfade_name = XFADE_PRESETS.get(transition, "fade")
        return self.apply_xfade(input_a, input_b, output_path, xfade_name, duration)

    def apply_xfade(
        self,
        input_a: str,
        input_b: str,
        output_path: str,
        xfade_name: str = "fade",
        duration: float = 0.5,
    ) -> Optional[str]:
        """Render transisi menggunakan FFmpeg xfade filter.

        xfade adalah cara yang direkomendasikan FFmpeg untuk transisi antar klip.
        Mendukung: fade, dissolve, wipeleft, wiperight, slideup, slidedown,
                   circleopen, radial, pixelize, squeezev, diagtl, hlwind, dll.
        """
        if not os.path.exists(input_a) or not os.path.exists(input_b):
            logger.warning("[Transition] Input files missing: %s, %s", input_a, input_b)
            return input_b

        # Dapatkan durasi klip A untuk menghitung offset xfade
        dur_a = self._get_duration(input_a)
        if dur_a is None:
            return input_b

        offset = max(0.0, dur_a - duration)

        try:
            os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
            cmd = [
                "ffmpeg",
                "-i", input_a,
                "-i", input_b,
                "-filter_complex",
                (
                    f"[0:v][1:v]xfade=transition={xfade_name}:"
                    f"duration={duration}:offset={offset}[xv];"
                    f"[0:a][1:a]acrossfade=d={duration}[xa]"
                ),
                "-map", "[xv]",
                "-map", "[xa]",
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", "18",
                "-c:a", "aac",
                output_path, "-y",
            ]
            result = subprocess.run(cmd, capture_output=True, timeout=300)
            if result.returncode == 0 and os.path.exists(output_path):
                logger.info("[Transition] xfade '%s' applied: %s", xfade_name, output_path)
                return output_path
            logger.warning("[Transition] xfade failed: %s", result.stderr.decode()[-200:])
            return input_b
        except subprocess.SubprocessError as e:
            logger.warning("[Transition] apply_xfade error: %s", e)
            return input_b

    def apply_batch_transitions(
        self,
        clips: list[str],
        xfade_name: str = "fade",
        duration: float = 0.5,
        output_path: str = "./output/compiled.mp4",
    ) -> Optional[str]:
        """Gabungkan banyak klip dengan transisi menggunakan FFmpeg filter_complex.

        Lebih efisien dari apply_xfade berulang karena satu-pass rendering.
        Cocok untuk timeline dengan 3+ scene.
        """
        clips = [c for c in clips if os.path.exists(c)]
        if not clips:
            return None
        if len(clips) == 1:
            return clips[0]

        try:
            os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)

            # Build ffmpeg input args
            cmd = ["ffmpeg"]
            for clip in clips:
                cmd += ["-i", clip]

            # Build xfade filter chain
            filter_parts = []
            current = "[0:v]"
            audio_current = "[0:a]"
            total_offset = 0.0

            for i, clip in enumerate(clips[:-1]):
                dur = self._get_duration(clip) or 5.0
                total_offset += dur - duration
                next_v = f"[v{i}]"
                next_a = f"[a{i}]"
                filter_parts.append(
                    f"{current}[{i+1}:v]xfade=transition={xfade_name}:"
                    f"duration={duration}:offset={max(0,total_offset)}{next_v}"
                )
                filter_parts.append(
                    f"{audio_current}[{i+1}:a]acrossfade=d={duration}{next_a}"
                )
                current = next_v
                audio_current = next_a

            filter_complex = ";".join(filter_parts)
            cmd += [
                "-filter_complex", filter_complex,
                "-map", current,
                "-map", audio_current,
                "-c:v", "libx264", "-preset", "fast", "-crf", "18",
                "-c:a", "aac",
                output_path, "-y",
            ]

            result = subprocess.run(cmd, capture_output=True, timeout=600)
            if result.returncode == 0 and os.path.exists(output_path):
                logger.info("[Transition] Batch compiled %d clips → %s", len(clips), output_path)
                return output_path

            logger.warning("[Transition] Batch compile failed: %s", result.stderr.decode()[-300:])
            return self._simple_concat(clips, output_path)

        except subprocess.SubprocessError as e:
            logger.warning("[Transition] apply_batch_transitions error: %s", e)
            return self._simple_concat(clips, output_path)

    def _simple_concat(self, clips: list[str], output_path: str) -> Optional[str]:
        """Fallback: concat tanpa transisi menggunakan concat demuxer."""
        import tempfile
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            for clip in clips:
                f.write(f"file '{os.path.abspath(clip)}'\n")
            list_path = f.name
        try:
            cmd = ["ffmpeg", "-f", "concat", "-safe", "0", "-i", list_path,
                   "-c", "copy", output_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=300, check=True)
            return output_path if os.path.exists(output_path) else None
        except subprocess.SubprocessError:
            return clips[-1] if clips else None
        finally:
            os.unlink(list_path)

    def _get_duration(self, video_path: str) -> Optional[float]:
        try:
            import json as _json
            cmd = ["ffprobe", "-v", "error", "-show_entries", "format=duration",
                   "-of", "json", video_path]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
            return float(_json.loads(result.stdout)["format"]["duration"])
        except Exception:
            return None

    def list_available_transitions(self) -> list[str]:
        """Kembalikan daftar semua xfade transitions yang tersedia di FFmpeg."""
        return [
            "fade", "fadeblack", "fadewhite", "distance", "wipeleft", "wiperight",
            "wipeup", "wipedown", "slideleft", "slideright", "slideup", "slidedown",
            "smoothleft", "smoothright", "smoothup", "smoothdown", "circleopen",
            "circlecrop", "rectcrop", "dissolve", "pixelize", "diagtl", "diagtr",
            "diagbl", "diagbr", "hlwind", "hrwind", "vuwind", "vdwind",
            "coverleft", "coverright", "revealleft", "revealright", "zoomin",
            "squeezev", "squeezeh", "hblur", "fadegrays", "wipetl", "wipetr",
            "wipebl", "wipebr",
        ]
