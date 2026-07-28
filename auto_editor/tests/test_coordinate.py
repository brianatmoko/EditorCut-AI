"""Tests for CoordinateEngine — 0-token math engine."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.layout_engine.coordinate import CoordinateEngine
from auto_editor.models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    Keyframe, Animation,
)


def test_pixel_conversion():
    engine = CoordinateEngine(1920, 1080)
    assert engine.to_pixels(0.5, 1920) == 960
    assert engine.to_pixels(1.0, 1080) == 1080
    assert engine.to_normalized(960, 1920) == 0.5


def test_get_bounds():
    engine = CoordinateEngine(1920, 1080)
    el = CoordinateElement(
        id="test", type="video",
        position=Position(x=0.5, y=0.5),
        size=Size(width=0.5, height=0.5)
    )
    bounds = engine.get_bounds(el)
    assert bounds["width"] == 960
    assert bounds["height"] == 540


def test_overlap_detection():
    engine = CoordinateEngine(1920, 1080)
    a = CoordinateElement("a", "video", size=Size(0.5, 0.5))
    b = CoordinateElement("b", "video", position=Position(0.5, 0.5), size=Size(0.5, 0.5))
    assert engine.check_overlap(a, b)


def test_no_overlap():
    engine = CoordinateEngine(1920, 1080)
    a = CoordinateElement("a", "video", size=Size(0.1, 0.1))
    b = CoordinateElement("b", "video", position=Position(1, 1), size=Size(0.1, 0.1))
    assert not engine.check_overlap(a, b)


def test_grid_creation():
    engine = CoordinateEngine(1920, 1080)
    grid = engine.create_grid(Position(0, 0), 2, 2)
    assert len(grid) == 4
    x_positions = set(el.position.x for el in grid)
    assert len(x_positions) == 2


def test_rule_of_thirds():
    engine = CoordinateEngine()
    pos = engine.rule_of_thirds("left", "top")
    assert round(pos.x, 4) == round(1/3, 4)
    assert round(pos.y, 4) == round(1/3, 4)


def test_split_screen():
    engine = CoordinateEngine()
    positions = engine.split_screen(4, "grid")
    assert len(positions) == 4


def test_keyframe_interpolation():
    engine = CoordinateEngine()
    elem = CoordinateElement(
        "test", "text",
        animation=Animation(keyframes=[
            Keyframe(time=0, opacity=0),
            Keyframe(time=1, opacity=1),
        ])
    )
    result = engine.apply_keyframe(elem, 0)
    assert result.transform.opacity == 0
    result = engine.apply_keyframe(elem, 1)
    assert result.transform.opacity == 1


def test_safe_zone():
    engine = CoordinateEngine(1920, 1080)
    zone = engine.safe_zone(0.1)
    assert zone["left"] == 192
    assert zone["top"] == 108
    assert zone["width"] == 1536
    assert zone["height"] == 864


def test_center_in_canvas():
    engine = CoordinateEngine()
    pos = engine.center_in_canvas(Size(0.5, 0.5))
    assert pos.x == 0.5
    assert pos.y == 0.5
