"""Load and apply layout templates to coordinate elements.

Bridge between TemplateDB and actual element positioning.
"""

from __future__ import annotations
from typing import Optional

from ...models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    TextStyle, VideoStyle, ShapeStyle, Animation, Keyframe,
)
from ...orchestrator.template_db import TemplateDB


class TemplateLoader:
    def __init__(self, templates_dir: str = "config/templates"):
        self.db = TemplateDB(templates_dir)

    def apply_template(
        self,
        template_name: str,
        variables: dict[str, str],
        duration: Optional[float] = None
    ) -> list[CoordinateElement]:
        template = self.db.get(template_name)
        if not template:
            return self._fallback_template(variables)

        elements = []
        for track in template.get("tracks", []):
            element = self._track_to_element(track, variables)
            if element:
                elements.append(element)

        if duration:
            scale = duration / max(
                (el.timeline.end for el in elements),
                default=10.0
            )
            for el in elements:
                el.timeline.end = el.timeline.end * scale
                el.timeline.start = el.timeline.start * scale

        return elements

    def _track_to_element(self, track: dict, variables: dict[str, str]) -> Optional[CoordinateElement]:
        pos = track.get("position", {})
        sz = track.get("size", {})
        tml = track.get("timeline", {})
        trf = track.get("transform", {})
        style = track.get("style", {})
        anim = track.get("animation", {})

        element_type = track.get("type", "video")

        element = CoordinateElement(
            id=track.get("id", "untitled"),
            type=element_type,
            position=Position(
                x=pos.get("x", 0.5), y=pos.get("y", 0.5), z=pos.get("z", 0)
            ),
            size=Size(
                width=sz.get("width", 0.5), height=sz.get("height", 0.5),
                unit=sz.get("unit", "normalized")
            ),
            timeline=Timeline(
                start=tml.get("start", 0.0), end=tml.get("end", 10.0)
            ),
            transform=Transform(
                rotation=trf.get("rotation", 0.0), scale=trf.get("scale", 1.0),
                opacity=trf.get("opacity", 1.0), anchor=trf.get("anchor", "center")
            ),
        )

        if anim:
            kf_list = anim.get("keyframes", [])
            if kf_list:
                keyframes = []
                for kf in kf_list:
                    if isinstance(kf, dict):
                        keyframes.append(Keyframe(
                            time=kf.get("time", 0), x=kf.get("x"),
                            y=kf.get("y"), scale=kf.get("scale"),
                            opacity=kf.get("opacity"), rotation=kf.get("rotation"),
                        ))
                element.animation = Animation(
                    keyframes=keyframes, easing=anim.get("easing", "ease_in_out")
                )

        resolved_style = self._resolve_variables(style, variables)

        if element_type == "text":
            element.text_style = TextStyle(
                text=resolved_style.get("text", ""),
                font_family=resolved_style.get("font_family", "Inter"),
                font_size=resolved_style.get("font_size", 48),
                font_weight=resolved_style.get("font_weight", 400),
                color=resolved_style.get("color", "#FFFFFF"),
                text_align=resolved_style.get("text_align", "center"),
            )
        elif element_type == "video":
            element.video_style = VideoStyle(
                fit=resolved_style.get("fit", "cover"),
            )
        elif element_type == "shape":
            element.shape_style = ShapeStyle(
                background_color=resolved_style.get("background_color", "#000000"),
                border_radius=resolved_style.get("border_radius", 0),
            )

        return element

    def _resolve_variables(self, data: dict, variables: dict[str, str]) -> dict:
        result = {}
        for key, value in data.items():
            if isinstance(value, str):
                for var_name, var_value in variables.items():
                    value = value.replace(f"{{{var_name}}}", var_value)
                result[key] = value
            elif isinstance(value, dict):
                result[key] = self._resolve_variables(value, variables)
            else:
                result[key] = value
        return result

    def _fallback_template(self, variables: dict[str, str]) -> list[CoordinateElement]:
        return [
            CoordinateElement(
                id="main", type="video",
                position=Position(0.5, 0.5, 0),
                size=Size(1.0, 1.0), timeline=Timeline(0, 30),
            ),
            CoordinateElement(
                id="title", type="text",
                position=Position(0.5, 0.1, 1),
                size=Size(0.8, 0.1), timeline=Timeline(0, 5),
                text_style=TextStyle(
                    text=variables.get("TITLE", "Video"),
                    font_size=56, color="#FFFFFF", text_align="center",
                ),
            ),
        ]

    def list_templates(self) -> list[dict]:
        return self.db.list_all()

    def find_suitable_template(self, plan) -> Optional[str]:
        query = f"{plan.style.value} {plan.target_platform.value} {plan.mood.value}"
        result = self.db.find_similar(query)
        return result.get("name") if result else None
