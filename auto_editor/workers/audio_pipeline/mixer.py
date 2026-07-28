"""Mix multiple audio tracks into final audio.

Combines voiceover, background music, and sound effects.
Uses FFmpeg for audio mixing — 0 token cost.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Optional
import subprocess
import os


@dataclass
class AudioMixConfig:
    voiceover_path: Optional[str] = None
    music_path: Optional[str] = None
    effect_paths: list[str] = None
    voiceover_volume: float = 1.0
    music_volume: float = 0.3
    effects_volume: float = 0.5
    output_path: str = "./output/audio/mixed.wav"


class AudioMixer:
    def mix(self, config: AudioMixConfig) -> Optional[str]:
        tracks = []
        filters = []

        if config.voiceover_path and os.path.exists(config.voiceover_path):
            tracks.append(config.voiceover_path)
            filters.append(f"[{len(tracks)-1}:a]volume={config.voiceover_volume}[v{len(tracks)-1}]")
        if config.music_path and os.path.exists(config.music_path):
            tracks.append(config.music_path)
            filters.append(f"[{len(tracks)-1}:a]volume={config.music_volume}[m{len(tracks)-1}]")
        if config.effect_paths:
            for i, ep in enumerate(config.effect_paths):
                if os.path.exists(ep):
                    tracks.append(ep)
                    filters.append(f"[{len(tracks)-1}:a]volume={config.effects_volume}[e{len(tracks)-1}]")

        if not tracks:
            return self._generate_silence(config.output_path)

        cmd = ["ffmpeg"]
        for track in tracks:
            cmd.extend(["-i", track])

        mix_inputs = "".join(f"[{'v' if i == 0 else 'm' if i == 1 else 'e'}{i}]" for i in range(len(tracks)))
        cmd.extend(["-filter_complex",
            f"{'; '.join(filters)};{mix_inputs}amix=inputs={len(tracks)}:duration=first:dropout_transition=2",
            "-acodec", "pcm_s16le", "-ar", "44100", config.output_path, "-y"])

        try:
            subprocess.run(cmd, capture_output=True, timeout=120, check=True)
            return config.output_path if os.path.exists(config.output_path) else None
        except subprocess.SubprocessError:
            return self._fallback_mix(tracks, config)

    def _fallback_mix(self, tracks: list[str], config: AudioMixConfig) -> Optional[str]:
        if not tracks:
            return self._generate_silence(config.output_path)
        try:
            cmd = ["ffmpeg", "-i", tracks[0], "-acodec", "copy", config.output_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=60, check=True)
            return config.output_path if os.path.exists(config.output_path) else None
        except subprocess.SubprocessError:
            return self._generate_silence(config.output_path)

    def _generate_silence(self, output_path: str, duration: float = 30.0) -> Optional[str]:
        try:
            os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
            cmd = ["ffmpeg", "-f", "lavfi", "-i", "anullsrc=r=44100:cl=mono",
                   "-t", str(duration), "-acodec", "pcm_s16le", output_path, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=30, check=True)
            return output_path if os.path.exists(output_path) else None
        except subprocess.SubprocessError:
            return None

    def normalize_volume(self, audio_path: str, target_db: float = -3.0) -> Optional[str]:
        output = audio_path.replace(".wav", "_normalized.wav")
        try:
            cmd = ["ffmpeg", "-i", audio_path, "-af", f"loudnorm=I={target_db}:LRA=11:TP=-1.5",
                   "-acodec", "pcm_s16le", output, "-y"]
            subprocess.run(cmd, capture_output=True, timeout=60, check=True)
            return output if os.path.exists(output) else None
        except subprocess.SubprocessError:
            return audio_path
