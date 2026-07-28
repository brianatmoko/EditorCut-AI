"""Align voiceover segments to video timeline.

Uses word-level timestamps to sync audio with visual elements.
Pure computation — 0 token cost.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Optional

from ...models import VoiceoverSegment


@dataclass
class AlignedVoiceover:
    segments: list[VoiceoverSegment]
    total_duration: float


class VoiceoverAligner:
    def align_to_scenes(
        self, audio_segments: list[VoiceoverSegment], scene_durations: list[float]
    ) -> AlignedVoiceover:
        aligned = []
        scene_cursor = 0.0

        for seg, scene_dur in zip(audio_segments, scene_durations):
            seg_dur = (seg.end - seg.start) if seg.end > seg.start else 0.0
            if seg_dur <= scene_dur:
                aligned.append(VoiceoverSegment(text=seg.text, start=scene_cursor,
                    end=scene_cursor + seg_dur, audio_path=seg.audio_path))
            else:
                ratio = scene_dur / seg_dur
                aligned.append(VoiceoverSegment(text=seg.text, start=scene_cursor,
                    end=scene_cursor + seg_dur * ratio, audio_path=seg.audio_path))
            scene_cursor += scene_dur

        total = scene_cursor if scene_cursor > 0 else (1.0 if len(scene_durations) > 0 else 0.0)
        return AlignedVoiceover(segments=aligned, total_duration=total)

    def adjust_speed_for_timeline(self, voiceover: AlignedVoiceover, target_duration: float) -> AlignedVoiceover:
        if voiceover.total_duration <= 0 or target_duration <= 0:
            return voiceover
        ratio = voiceover.total_duration / target_duration
        adjusted = []
        for seg in voiceover.segments:
            dur = (seg.end - seg.start) / ratio
            adjusted.append(VoiceoverSegment(text=seg.text, start=seg.start / ratio,
                end=seg.start / ratio + dur, audio_path=seg.audio_path))
        return AlignedVoiceover(segments=adjusted, total_duration=target_duration)
