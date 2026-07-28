"""End-to-end integration tests for the complete pipeline."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.intent_router import IntentRouter
from auto_editor.orchestrator.workflow_engine import WorkflowEngine, WorkflowNode
from auto_editor.orchestrator.template_db import TemplateDB
from auto_editor.config.settings_loader import load_config
from auto_editor.workers.scene_detector.detector import SceneDetector
from auto_editor.workers.layout_engine.coordinate import CoordinateEngine
from auto_editor.models import (
    CoordinateElement, Position, Size, Timeline,
    EditingPlan, EditingIntent,
)


def test_intent_to_template():
    """Test: classify intent -> find matching template."""
    router = IntentRouter()
    plan = router.create_plan("buat video tiktok product review 60 detik")
    assert plan.intent == EditingIntent.AUTO_EDIT
    assert plan.duration == 60
    assert plan.aspect_ratio.value == "9:16"

    db = TemplateDB()
    result = db.find_similar(f"{plan.style.value} {plan.target_platform.value}")
    assert result is not None


def test_coordinate_to_render():
    """Test: coordinate element -> compositor -> render ready."""
    engine = CoordinateEngine(1920, 1080)

    elements = [
        CoordinateElement(
            id="bg", type="video",
            position=Position(0.5, 0.5, 0),
            size=Size(1.0, 1.0),
            timeline=Timeline(0, 10),
        ),
        CoordinateElement(
            id="title", type="text",
            position=Position(0.5, 0.1, 1),
            size=Size(0.8, 0.1),
            timeline=Timeline(0, 5),
        ),
    ]

    bounds = engine.get_bounds(elements[0])
    assert bounds["width"] == 1920
    assert bounds["height"] == 1080

    overlap = engine.check_overlap(elements[0], elements[1])
    assert overlap


def test_config_load():
    """Test: config loads correctly."""
    config = load_config()
    assert config.mode.value in ("offline", "hybrid", "cloud")
    assert 0 < config.behavior.confidence_threshold <= 1.0
    assert config.resources.max_vram_gb > 0


def test_scene_detector_fallback():
    """Test: scene detector graceful degradation."""
    det = SceneDetector()
    shots = det.detect("nonexistent_video.mp4")
    assert shots == []


def test_workflow_engine_dag():
    """Test: workflow engine topological execution."""
    engine = WorkflowEngine()
    order = []

    engine.register("integration_test", [
        WorkflowNode(id="a", handler=lambda **_: order.append("a") or {}),
        WorkflowNode(id="b", handler=lambda **_: order.append("b") or {}, deps=["a"]),
        WorkflowNode(id="c", handler=lambda **_: order.append("c") or {}, deps=["a"]),
    ])

    result = engine.run("integration_test", {})
    assert result.success
    assert order == ["a", "b", "c"] or order == ["a", "c", "b"]


def test_full_editing_pipeline_in_memory():
    """Test: complete in-memory editing pipeline without real files."""
    router = IntentRouter()
    plan = router.create_plan("buat video cinematic 30 detik")
    assert plan.duration == 30

    db = TemplateDB()
    template = db.find_similar("cinematic")
    assert template is not None

    elements = db.apply(template["name"], {"TITLE": "Test Video"})
    assert len(elements) >= 2

    engine = CoordinateEngine()
    bounds = engine.get_bounds(elements[0])
    assert bounds["width"] > 0
    assert bounds["height"] > 0

    for i, a in enumerate(elements):
        for b in elements[i+1:]:
            if engine.check_overlap(a, b):
                adjusted = engine.resolve_overlap(a, b)
                assert adjusted.position.x != b.position.x or adjusted.position.y != b.position.y
