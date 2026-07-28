"""Automatic Speech Recognition — faster-whisper backend (primary) + whisper.cpp (fallback).

faster-whisper menggunakan CTranslate2 quantized model:
  - 5x lebih cepat dari Whisper original
  - 4x lebih hemat RAM
  - Output word-level timestamps
  - Support 99 bahasa (termasuk Indonesia)
  - Bisa berjalan di CPU saja

Model otomatis diunduh saat pertama kali digunakan (~145MB untuk 'base').
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional, Literal
from pathlib import Path
import subprocess
import json
import os
import logging

logger = logging.getLogger(__name__)

ModelSize = Literal["tiny", "base", "small", "medium", "large-v2", "large-v3"]


@dataclass
class TranscriptionSegment:
    id: int
    start: float
    end: float
    text: str
    confidence: float = 1.0
    words: list[dict] = field(default_factory=list)


@dataclass
class TranscriptionResult:
    text: str
    language: str
    segments: list[dict] = field(default_factory=list)
    duration: float = 0.0


class ASREngine:
    """ASR Engine dengan faster-whisper backend.

    Otomatis download model saat pertama kali digunakan.
    Tidak perlu setup manual — langsung pakai.

    Args:
        model_size: Ukuran model ("tiny", "base", "small", "medium", "large-v3")
        model_path: Path ke model whisper.cpp GGUF (untuk fallback lama)
        device: "cpu" atau "cuda" (otomatis deteksi)
        compute_type: "int8" (CPU hemat), "float16" (GPU cepat), "float32" (CPU presisi)
    """

    def __init__(
        self,
        model_path: str = "./models/asr/whisper-small.gguf",
        model_type: str = "small",
        model_size: ModelSize = "base",
        device: str = "auto",
        compute_type: str = "int8",
    ):
        self.model_path = model_path
        self.model_type = model_type
        self.model_size = model_size
        self.compute_type = compute_type
        self._device = self._resolve_device(device)
        self._faster_whisper_available = self._check_faster_whisper()
        self._fw_model = None  # Lazy load

    def _resolve_device(self, device: str) -> str:
        if device != "auto":
            return device
        try:
            import torch
            return "cuda" if torch.cuda.is_available() else "cpu"
        except ImportError:
            return "cpu"

    def _check_faster_whisper(self) -> bool:
        try:
            import faster_whisper  # noqa: F401
            logger.info("[ASR] faster-whisper backend: ACTIVE (device=%s, compute=%s)",
                        self._device, self.compute_type)
            return True
        except ImportError:
            logger.info("[ASR] faster-whisper not found, using whisper.cpp fallback")
            return False

    def _load_model(self):
        """Lazy load model on first use (auto-downloads if needed)."""
        if self._fw_model is not None:
            return
        try:
            from faster_whisper import WhisperModel
            logger.info("[ASR] Loading faster-whisper model '%s' on %s...",
                        self.model_size, self._device)
            self._fw_model = WhisperModel(
                self.model_size,
                device=self._device,
                compute_type=self.compute_type,
                # Model cache: ~/.cache/huggingface/hub/
            )
            logger.info("[ASR] Model loaded ✓")
        except Exception as e:
            logger.error("[ASR] Failed to load faster-whisper model: %s", e)
            self._fw_model = None

    # ── Public API ────────────────────────────────────────────────────────────

    def transcribe(
        self,
        audio_path: str,
        language: str = "id",
        output_format: str = "json",
    ) -> Optional[TranscriptionResult]:
        """Transkripsi audio ke teks dengan timestamp per segmen."""
        if not os.path.exists(audio_path):
            logger.warning("[ASR] File not found: %s", audio_path)
            return None

        if self._faster_whisper_available:
            result = self._transcribe_faster_whisper(audio_path, language)
            if result:
                return result
            logger.warning("[ASR] faster-whisper failed, falling back to whisper.cpp")

        return self._transcribe_whisper_cpp(audio_path, language)

    def transcribe_to_srt(self, audio_path: str, language: str = "id") -> Optional[str]:
        """Transkripsi dan konversi ke format SRT (SubRip)."""
        result = self.transcribe(audio_path, language)
        if not result or not result.segments:
            return None
        srt_lines = []
        for i, seg in enumerate(result.segments, 1):
            start = self._fmt_srt(seg["start"])
            end = self._fmt_srt(seg["end"])
            text = seg["text"].strip()
            srt_lines.append(f"{i}\n{start} --> {end}\n{text}\n")
        return "\n".join(srt_lines)

    def transcribe_to_vtt(self, audio_path: str, language: str = "id") -> Optional[str]:
        """Transkripsi dan konversi ke format WebVTT."""
        result = self.transcribe(audio_path, language)
        if not result or not result.segments:
            return None
        lines = ["WEBVTT\n"]
        for i, seg in enumerate(result.segments, 1):
            start = self._fmt_vtt(seg["start"])
            end = self._fmt_vtt(seg["end"])
            lines.append(f"{i}\n{start} --> {end}\n{seg['text'].strip()}\n")
        return "\n".join(lines)

    def transcribe_video(
        self,
        video_path: str,
        language: str = "id",
        output_format: Literal["srt", "vtt", "json"] = "srt",
        output_path: Optional[str] = None,
    ) -> Optional[str]:
        """Transkripsi langsung dari video (ekstrak audio otomatis)."""
        audio_path = self._extract_audio(video_path)
        if not audio_path:
            return None

        try:
            result = self.transcribe(audio_path, language)
            if not result:
                return None

            if output_format == "srt":
                content = self.transcribe_to_srt(audio_path, language)
            elif output_format == "vtt":
                content = self.transcribe_to_vtt(audio_path, language)
            else:
                segs = [{"id": s.get("id",0), "start": s.get("start",0.0),
                          "end": s.get("end",0.0), "text": s.get("text","")}
                         for s in result.segments]
                content = json.dumps({"language": result.language, "segments": segs}, ensure_ascii=False, indent=2)

            if content and output_path:
                Path(output_path).parent.mkdir(parents=True, exist_ok=True)
                with open(output_path, "w", encoding="utf-8") as f:
                    f.write(content)
                logger.info("[ASR] Subtitle saved: %s", output_path)
                return output_path

            return content
        finally:
            # Cleanup extracted audio
            if audio_path and os.path.exists(audio_path) and audio_path.endswith("_asr_tmp.wav"):
                os.unlink(audio_path)

    def burn_subtitles(
        self,
        video_path: str,
        srt_path: str,
        output_path: str,
        font_size: int = 24,
        font_color: str = "white",
        outline_color: str = "black",
        position: str = "bottom",
    ) -> Optional[str]:
        """Bakar subtitle SRT ke dalam video menggunakan FFmpeg subtitles filter."""
        if not os.path.exists(srt_path):
            return None

        margin_v = 30 if position == "bottom" else 900
        style = (
            f"FontSize={font_size},PrimaryColour=&H00ffffff,"
            f"OutlineColour=&H00000000,BorderStyle=1,Outline=2,"
            f"MarginV={margin_v}"
        )
        try:
            Path(output_path).parent.mkdir(parents=True, exist_ok=True)
            cmd = [
                "ffmpeg", "-i", video_path,
                "-vf", f"subtitles={srt_path}:force_style='{style}'",
                "-c:a", "copy",
                "-c:v", "libx264", "-crf", "18",
                output_path, "-y"
            ]
            result = subprocess.run(cmd, capture_output=True, timeout=300)
            if result.returncode == 0 and os.path.exists(output_path):
                logger.info("[ASR] Subtitles burned into: %s", output_path)
                return output_path
            logger.error("[ASR] burn_subtitles FFmpeg error: %s", result.stderr.decode()[-200:])
            return None
        except subprocess.SubprocessError as e:
            logger.error("[ASR] burn_subtitles failed: %s", e)
            return None

    # ── faster-whisper backend ────────────────────────────────────────────────

    def _transcribe_faster_whisper(
        self, audio_path: str, language: str
    ) -> Optional[TranscriptionResult]:
        try:
            self._load_model()
            if self._fw_model is None:
                return None

            # faster-whisper uses ISO 639-1 codes — "id" = Indonesian
            lang = language if language != "auto" else None

            segments_iter, info = self._fw_model.transcribe(
                audio_path,
                language=lang,
                beam_size=5,
                word_timestamps=True,
                vad_filter=True,           # Voice Activity Detection — skip silence
                vad_parameters=dict(
                    min_silence_duration_ms=500,
                ),
            )

            full_text = []
            segments = []
            for i, seg in enumerate(segments_iter):
                text = seg.text.strip()
                full_text.append(text)
                words = []
                if seg.words:
                    words = [{"word": w.word, "start": w.start, "end": w.end,
                              "probability": w.probability} for w in seg.words]
                segments.append({
                    "id": i,
                    "start": round(seg.start, 3),
                    "end": round(seg.end, 3),
                    "text": text,
                    "confidence": seg.avg_logprob,
                    "words": words,
                })

            detected_lang = info.language if lang is None else language
            result = TranscriptionResult(
                text=" ".join(full_text),
                language=detected_lang,
                segments=segments,
                duration=info.duration,
            )
            logger.info("[ASR] faster-whisper: %.1fs audio → %d segments (lang=%s)",
                        info.duration, len(segments), detected_lang)
            return result

        except Exception as e:
            logger.warning("[ASR] faster-whisper transcription failed: %s", e)
            return None

    # ── whisper.cpp fallback ──────────────────────────────────────────────────

    def _transcribe_whisper_cpp(self, audio_path: str, language: str) -> Optional[TranscriptionResult]:
        """Original whisper.cpp binary fallback."""
        try:
            cmd = ["whisper.cpp", "--model", self.model_path, "--file", audio_path,
                   "--language", language, "--output-format", "json"]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            if result.returncode != 0:
                return self._fallback_transcribe(audio_path)
            data = json.loads(result.stdout)
            segments = [{"id": s.get("id", 0), "start": s.get("start", 0.0),
                         "end": s.get("end", 0.0), "text": s.get("text", "").strip(),
                         "confidence": s.get("confidence", 1.0)} for s in data.get("segments", [])]
            return TranscriptionResult(
                text=data.get("text", "").strip(),
                language=language,
                segments=segments,
                duration=data.get("duration", 0.0),
            )
        except (subprocess.SubprocessError, FileNotFoundError, json.JSONDecodeError):
            return self._fallback_transcribe(audio_path)

    # ── Utilities ─────────────────────────────────────────────────────────────

    def _extract_audio(self, video_path: str) -> Optional[str]:
        """Ekstrak audio dari video ke WAV temporary file."""
        audio_path = str(Path(video_path).with_suffix("")) + "_asr_tmp.wav"
        try:
            cmd = ["ffmpeg", "-i", video_path, "-vn",
                   "-acodec", "pcm_s16le", "-ar", "16000", "-ac", "1",
                   audio_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=120, check=True)
            return audio_path if os.path.exists(audio_path) else None
        except subprocess.SubprocessError as e:
            logger.error("[ASR] Audio extraction failed: %s", e)
            return None

    def _fmt_srt(self, seconds: float) -> str:
        h = int(seconds // 3600)
        m = int((seconds % 3600) // 60)
        s = int(seconds % 60)
        ms = int((seconds % 1) * 1000)
        return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"

    def _fmt_vtt(self, seconds: float) -> str:
        h = int(seconds // 3600)
        m = int((seconds % 3600) // 60)
        s = int(seconds % 60)
        ms = int((seconds % 1) * 1000)
        return f"{h:02d}:{m:02d}:{s:02d}.{ms:03d}"

    def _fallback_transcribe(self, audio_path: str) -> Optional[TranscriptionResult]:
        return TranscriptionResult(
            text="[Transcription unavailable]",
            language="id",
            segments=[{"id": 0, "start": 0.0, "end": 0.0,
                       "text": "[Transcription unavailable]", "confidence": 0.0}],
            duration=0.0,
        )

    def estimate_tokens(self, audio_duration: float) -> int:
        return 0
