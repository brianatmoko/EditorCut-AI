"""Composite multiple CoordinateElements into render-ready frames.

Takes layout coordinates + assets → produces composited frames.
Serves as the bridge between layout logic and the renderer.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional
from pathlib import Path

from ...models import CoordinateElement
from .coordinate import CoordinateEngine


@dataclass
class CompositedFrame:
    frame_number: int
    timestamp: float
    layer_count: int
    elements: list[dict]


class Compositor:
    def __init__(self, canvas_width: int = 1920, canvas_height: int = 1080):
        self.canvas_width = canvas_width
        self.canvas_height = canvas_height
        self.coord = CoordinateEngine(canvas_width, canvas_height)

    def composite_frame(
        self,
        elements: list[CoordinateElement],
        frame_number: int,
        fps: float = 30.0
    ) -> Optional[CompositedFrame]:
        current_time = frame_number / fps

        visible = [
            el for el in elements
            if el.timeline.start <= current_time <= el.timeline.end
        ]

        if not visible:
            return None

        visible.sort(key=lambda el: el.position.z)

        frame_elements = []
        for el in visible:
            if el.animation:
                anim_time = current_time - el.timeline.start
                el = self.coord.apply_keyframe(el, anim_time)

            bounds = self.coord.get_bounds(el)

            frame_elements.append({
                "id": el.id,
                "type": el.type,
                "z": el.position.z,
                "bounds": bounds,
                "opacity": el.transform.opacity,
                "rotation": el.transform.rotation,
                "scale": el.transform.scale,
                "text": el.text_style.text if el.text_style else None,
                "style": self._serialize_style(el),
            })

        return CompositedFrame(
            frame_number=frame_number,
            timestamp=current_time,
            layer_count=len(frame_elements),
            elements=frame_elements,
        )

    def composite_range(
        self,
        elements: list[CoordinateElement],
        start_frame: int,
        end_frame: int,
        fps: float = 30.0,
        progress_callback=None
    ) -> list[CompositedFrame]:
        frames = []
        total = end_frame - start_frame + 1

        for i in range(start_frame, end_frame + 1):
            frame = self.composite_frame(elements, i, fps)
            if frame:
                frames.append(frame)
            if progress_callback:
                progress_callback(i - start_frame + 1, total)

        return frames

    def get_project_duration(self, elements: list[CoordinateElement]) -> float:
        if not elements:
            return 0.0
        return max(el.timeline.end for el in elements)

    def get_frame_count(self, elements: list[CoordinateElement], fps: float = 30.0) -> int:
        duration = self.get_project_duration(elements)
        return int(duration * fps)

    def to_filter_graph(self, elements: list[CoordinateElement], canvas_size: tuple[int, int]) -> str:
        if not elements:
            return ""

        filters = []
        input_index = 0

        for el in elements:
            bounds = self.coord.get_bounds(el)
            x = int(bounds["left"])
            y = int(bounds["top"])
            w = int(bounds["width"])
            h = int(bounds["height"])

            filters.append(
                f"[{input_index}:v]scale={w}:{h},"
                f"setpts=PTS-STARTPTS,"
                f"format=rgba,"
                f"colorchannelmixer=aa={el.transform.opacity}[v{input_index}];"
            )
            input_index += 1

        if filters:
            overlay = f"[0:v]"
            for i in range(1, input_index):
                if i == 1:
                    overlay += f"[v{i}]overlay={x}:{y}[ov{i}]"
                else:
                    overlay += f"[ov{i-1}][v{i}]overlay={x}:{y}[ov{i}]"

            filters.append(overlay)

        return "".join(filters)

    def _serialize_style(self, element: CoordinateElement) -> dict:
        if element.text_style:
            return {
                "text": element.text_style.text,
                "font_family": element.text_style.font_family,
                "font_size": element.text_style.font_size,
                "color": element.text_style.color,
                "text_align": element.text_style.text_align,
            }
        if element.video_style:
            return {"fit": element.video_style.fit}
        if element.shape_style:
            return {"bg_color": element.shape_style.background_color}
        return {}
