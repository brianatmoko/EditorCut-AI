"""Coordinate math engine — pure computation, 0 token cost."""

from __future__ import annotations
from typing import Optional
import math

from ...models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    Keyframe, Animation, AspectRatio
)


class CoordinateEngine:
    """Compute exact positions from normalized coordinates."""

    def __init__(self, canvas_width: int = 1920, canvas_height: int = 1080):
        self.canvas_width = canvas_width
        self.canvas_height = canvas_height
        self.aspect_ratio = canvas_width / canvas_height

    def to_pixels(self, normalized: float, dimension: int) -> int:
        return round(normalized * dimension)

    def to_normalized(self, pixels: int, dimension: int) -> float:
        if dimension == 0:
            return 0.0
        return pixels / dimension

    def get_bounds(self, element: CoordinateElement) -> dict:
        pw = self.canvas_width
        ph = self.canvas_height
        if element.size.unit == "normalized":
            w = element.size.width * pw
            h = element.size.height * ph
        elif element.size.unit == "pixel":
            w = element.size.width
            h = element.size.height
        else:
            w = (element.size.width / 100) * pw
            h = (element.size.height / 100) * ph
        w *= element.transform.scale
        h *= element.transform.scale
        anchor_offsets = {
            "center": (0.5, 0.5), "top_left": (0, 0), "top_right": (1, 0),
            "bottom_left": (0, 1), "bottom_right": (1, 1),
        }
        ox, oy = anchor_offsets.get(element.transform.anchor, (0.5, 0.5))
        x = (element.position.x * pw) - (w * ox)
        y = (element.position.y * ph) - (h * oy)
        return {
            "left": x, "top": y, "right": x + w, "bottom": y + h,
            "width": w, "height": h, "center_x": x + w / 2, "center_y": y + h / 2,
        }

    def check_overlap(self, a: CoordinateElement, b: CoordinateElement) -> bool:
        ba = self.get_bounds(a)
        bb = self.get_bounds(b)
        return not (ba["right"] <= bb["left"] or ba["left"] >= bb["right"] or
                    ba["bottom"] <= bb["top"] or ba["top"] >= bb["bottom"])

    def resolve_overlap(self, fixed: CoordinateElement, moving: CoordinateElement,
                        direction: str = "right") -> CoordinateElement:
        bf = self.get_bounds(fixed)
        bm = self.get_bounds(moving)
        import copy
        adjusted = copy.deepcopy(moving)
        if direction == "right":
            adjusted.position.x = (bf["right"] + bm["width"] / 2) / self.canvas_width
        elif direction == "left":
            adjusted.position.x = (bf["left"] - bm["width"] / 2) / self.canvas_width
        elif direction == "down":
            adjusted.position.y = (bf["bottom"] + bm["height"] / 2) / self.canvas_height
        elif direction == "up":
            adjusted.position.y = (bf["top"] - bm["height"] / 2) / self.canvas_height
        return adjusted

    def create_grid(self, start: Position, cols: int, rows: int,
                    spacing: float = 0.02, element_size: Size = Size(0.2, 0.2)) -> list[CoordinateElement]:
        elements = []
        for row in range(rows):
            for col in range(cols):
                x = start.x + col * (element_size.width + spacing)
                y = start.y + row * (element_size.height + spacing)
                elements.append(CoordinateElement(
                    id=f"grid_{row}_{col}", type="video",
                    position=Position(x=x, y=y, z=start.z),
                    size=Size(width=element_size.width, height=element_size.height)
                ))
        return elements

    def split_screen(self, count: int, layout: str = "grid") -> list[Position]:
        positions = []
        if layout == "horizontal":
            h = 1.0 / count
            for i in range(count):
                positions.append(Position(x=0.5, y=h * i + h / 2))
        elif layout == "vertical":
            w = 1.0 / count
            for i in range(count):
                positions.append(Position(x=w * i + w / 2, y=0.5))
        elif layout == "picture_in_picture":
            positions = [Position(x=0.5, y=0.5), Position(x=0.8, y=0.8)]
        else:
            sqrt_n = math.ceil(math.sqrt(count))
            w = 1.0 / sqrt_n
            h = 1.0 / sqrt_n
            for row in range(sqrt_n):
                for col in range(sqrt_n):
                    if len(positions) < count:
                        positions.append(Position(x=w * col + w / 2, y=h * row + h / 2))
        return positions

    def apply_keyframe(self, element: CoordinateElement, time: float) -> CoordinateElement:
        if not element.animation or not element.animation.keyframes:
            return element
        kfs = sorted(element.animation.keyframes, key=lambda k: k.time)
        if time <= kfs[0].time:
            return self._apply_keyframe_values(element, kfs[0])
        if time >= kfs[-1].time:
            return self._apply_keyframe_values(element, kfs[-1])
        for i in range(len(kfs) - 1):
            if kfs[i].time <= time <= kfs[i + 1].time:
                t = (time - kfs[i].time) / (kfs[i + 1].time - kfs[i].time)
                eased_t = self._ease(t, element.animation.easing)
                return self._interpolate(element, kfs[i], kfs[i + 1], eased_t)
        return element

    def _interpolate(self, element: CoordinateElement, kf_start: Keyframe, kf_end: Keyframe, t: float) -> CoordinateElement:
        import copy
        result = copy.deepcopy(element)
        if kf_start.x is not None and kf_end.x is not None:
            result.position.x = kf_start.x + (kf_end.x - kf_start.x) * t
        if kf_start.y is not None and kf_end.y is not None:
            result.position.y = kf_start.y + (kf_end.y - kf_start.y) * t
        if kf_start.scale is not None and kf_end.scale is not None:
            result.transform.scale = kf_start.scale + (kf_end.scale - kf_start.scale) * t
        if kf_start.opacity is not None and kf_end.opacity is not None:
            result.transform.opacity = kf_start.opacity + (kf_end.opacity - kf_start.opacity) * t
        if kf_start.rotation is not None and kf_end.rotation is not None:
            result.transform.rotation = kf_start.rotation + (kf_end.rotation - kf_start.rotation) * t
        return result

    def _apply_keyframe_values(self, element: CoordinateElement, kf: Keyframe) -> CoordinateElement:
        import copy
        result = copy.deepcopy(element)
        if kf.x is not None: result.position.x = kf.x
        if kf.y is not None: result.position.y = kf.y
        if kf.scale is not None: result.transform.scale = kf.scale
        if kf.opacity is not None: result.transform.opacity = kf.opacity
        if kf.rotation is not None: result.transform.rotation = kf.rotation
        return result

    def _ease(self, t: float, easing: str) -> float:
        if easing == "linear":
            return t
        elif easing == "ease_in":
            return t * t
        elif easing == "ease_out":
            return t * (2 - t)
        elif easing == "ease_in_out":
            return t * t * (3 - 2 * t) if t < 0.5 else 1 - (1 - t) * (1 - t) * (3 - 2 * (1 - t))
        elif easing == "bounce":
            if t < 0.3636:
                return 7.5625 * t * t
            elif t < 0.7273:
                t -= 0.5455
                return 7.5625 * t * t + 0.75
            elif t < 0.9091:
                t -= 0.8182
                return 7.5625 * t * t + 0.9375
            else:
                t -= 0.9545
                return 7.5625 * t * t + 0.984375
        return t

    def center_in_canvas(self, size: Size) -> Position:
        return Position(x=0.5, y=0.5)

    def align_to_edge(self, edge: str, margin: float = 0.05) -> Position:
        positions = {
            "top-left": Position(x=margin, y=margin),
            "top-right": Position(x=1 - margin, y=margin),
            "bottom-left": Position(x=margin, y=1 - margin),
            "bottom-right": Position(x=1 - margin, y=1 - margin),
            "top": Position(x=0.5, y=margin),
            "bottom": Position(x=0.5, y=1 - margin),
        }
        return positions.get(edge, Position(x=0.5, y=0.5))

    def rule_of_thirds(self, h_pos: str, v_pos: str) -> Position:
        h_map = {"left": 1/3, "center": 0.5, "right": 2/3}
        v_map = {"top": 1/3, "middle": 0.5, "bottom": 2/3}
        return Position(x=h_map.get(h_pos, 0.5), y=v_map.get(v_pos, 0.5))

    def golden_ratio_position(self, offset_x: float = 0, offset_y: float = 0) -> Position:
        phi = (1 + math.sqrt(5)) / 2
        return Position(x=1/phi + offset_x, y=1/phi + offset_y)

    def safe_zone(self, margin: float = 0.1) -> dict:
        return {
            "left": self.canvas_width * margin, "top": self.canvas_height * margin,
            "right": self.canvas_width * (1 - margin), "bottom": self.canvas_height * (1 - margin),
            "width": self.canvas_width * (1 - 2 * margin), "height": self.canvas_height * (1 - 2 * margin),
        }
