"""Generate text overlays and subtitles for video.

Supports SRT, ASS subtitle formats and dynamic text overlays.
Pure rule-based from coordinate layout — 0 token cost.
"""

from __future__ import annotations
from typing import Optional
import os


class TextOverlayEngine:
    def generate_subtitle_file(self, segments: list[dict], output_path: str, format: str = "srt") -> Optional[str]:
        if format == "ass":
            return self._generate_ass(segments, output_path)
        return self._generate_srt(segments, output_path)

    def _generate_srt(self, segments: list[dict], output_path: str) -> Optional[str]:
        lines = []
        for i, seg in enumerate(segments, 1):
            text = seg.get("text", "").strip()
            if text:
                lines.append(f"{i}\n{self._fmt_time(seg.get('start', 0))} --> {self._fmt_time(seg.get('end', 0))}\n{text}\n")
        if not lines:
            return None
        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        return output_path

    def _generate_ass(self, segments: list[dict], output_path: str) -> Optional[str]:
        header = (
            "[Script Info]\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n"
            "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, "
            "SecondaryColour, OutlineColour, BackColour, Bold, Italic, "
            "Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, "
            "BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, "
            "MarginV, Encoding\n"
            "Style: Default, Arial, 48, &H00FFFFFF, &H000000FF, "
            "&H00000000, &H80000000, 0, 0, 0, 0, 100, 100, 0, 0, "
            "1, 2, 1, 2, 20, 20, 40, 1\n\n"
            "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n"
        )
        events = []
        for seg in segments:
            text = seg.get("text", "").strip()
            if text:
                events.append(f"Dialogue: 0,{self._fmt_ass(seg.get('start', 0))},{self._fmt_ass(seg.get('end', 0))},Default,,0,0,0,,{text}")
        if not events:
            return None
        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(header + "\n".join(events))
        return output_path

    def _fmt_time(self, seconds: float) -> str:
        h, m = int(seconds // 3600), int((seconds % 3600) // 60)
        s, ms = int(seconds % 60), int((seconds % 1) * 1000)
        return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"

    def _fmt_ass(self, seconds: float) -> str:
        h, m = int(seconds // 3600), int((seconds % 3600) // 60)
        s, cs = int(seconds % 60), int((seconds % 1) * 100)
        return f"{h}:{m:02d}:{s:02d}.{cs:02d}"

    def generate_drawtext_filter(self, text: str, x: int, y: int, font_size: int = 48,
                                  color: str = "white", duration: float = 5.0) -> str:
        escaped = text.replace("'", "'\\''").replace(":", "\\:")
        return (f"drawtext=text='{escaped}':x={x}:y={y}:fontsize={font_size}:"
                f"fontcolor={color}:fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf:"
                f"enable='between(t,0,{duration})'")
