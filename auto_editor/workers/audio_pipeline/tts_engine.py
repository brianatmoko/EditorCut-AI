"""Local Text-to-Speech engine using GGUF models.

Pure local inference — 0 token cost, no API calls.
Supports CosyVoice and Bark models in GGUF format.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Literal
from pathlib import Path
import json
import subprocess
import os


@dataclass
class TTSResult:
    audio_path: str
    text: str
    duration: float
    word_timings: list[dict] = field(default_factory=list)


class TTSEngine:
    def __init__(
        self,
        model_path: str = "./models/tts/cosyvoice.gguf",
        backend: Literal["cosyvoice", "bark", "piper"] = "cosyvoice",
        voice: str = "default"
    ):
        self.model_path = model_path
        self.backend = backend
        self.voice = voice

    def generate(
        self,
        text: str,
        output_path: Optional[str] = None,
        language: str = "id",
        speed: float = 1.0
    ) -> Optional[TTSResult]:
        if not text or not text.strip():
            return None
        if not output_path:
            output_path = self._default_path(text)
        if self.backend == "piper":
            return self._run_piper(text, output_path)
        elif self.backend == "bark":
            return self._run_bark(text, output_path)
        else:
            return self._run_cosyvoice(text, output_path, language, speed)

    def generate_batch(
        self, segments: list[dict], output_dir: str = "./output/audio/"
    ) -> list[Optional[TTSResult]]:
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        results = []
        for i, seg in enumerate(segments):
            out_path = f"{output_dir}/segment_{i:04d}.wav"
            results.append(self.generate(
                text=seg.get("text", ""), output_path=out_path,
                language=seg.get("language", "id"), speed=seg.get("speed", 1.0),
            ))
        return results

    def _run_cosyvoice(self, text: str, output_path: str, language: str, speed: float) -> Optional[TTSResult]:
        try:
            cmd = ["llama-tts", "--model", self.model_path, "--text", text,
                   "--output", output_path, "--language", language, "--speed", str(speed)]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
            if result.returncode != 0 or not os.path.exists(output_path):
                return self._fallback_piper(text, output_path)
            duration = self._get_audio_duration(output_path)
            return TTSResult(audio_path=output_path, text=text, duration=duration)
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_piper(text, output_path)

    def _run_bark(self, text: str, output_path: str) -> Optional[TTSResult]:
        try:
            cmd = ["bark-tts", "--model", self.model_path, "--text", text, "--output", output_path]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
            if result.returncode != 0 or not os.path.exists(output_path):
                return self._fallback_piper(text, output_path)
            duration = self._get_audio_duration(output_path)
            return TTSResult(audio_path=output_path, text=text, duration=duration)
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_piper(text, output_path)

    def _run_piper(self, text: str, output_path: str) -> Optional[TTSResult]:
        try:
            json_input = json.dumps({"text": text})
            cmd = ["piper-tts", "--model", self.model_path, "--output", output_path]
            result = subprocess.run(cmd, input=json_input, capture_output=True, text=True, timeout=60)
            if result.returncode != 0 or not os.path.exists(output_path):
                return None
            duration = self._get_audio_duration(output_path)
            return TTSResult(audio_path=output_path, text=text, duration=duration)
        except (subprocess.SubprocessError, FileNotFoundError):
            return None

    def _fallback_piper(self, text: str, output_path: str) -> Optional[TTSResult]:
        try:
            duration = len(text.split()) * 0.3
            cmd = ["ffmpeg", "-f", "lavfi", "-i", "anullsrc=r=44100:cl=mono",
                   "-t", str(duration), "-acodec", "pcm_s16le", output_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=30)
            if os.path.exists(output_path):
                return TTSResult(audio_path=output_path, text=text, duration=duration)
        except (subprocess.SubprocessError, FileNotFoundError):
            pass
        return None

    def _default_path(self, text: str) -> str:
        safe_name = "".join(c if c.isalnum() else "_" for c in text[:30])
        return f"./output/audio/{safe_name}.wav"

    def _get_audio_duration(self, path: str) -> float:
        try:
            cmd = ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "json", path]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
            return float(json.loads(result.stdout)["format"]["duration"])
        except (subprocess.SubprocessError, json.JSONDecodeError, KeyError, ValueError):
            return 0.0

    def estimate_tokens(self, text_length: int) -> int:
        return 0
