"""Tests for data models — dataclass construction and validation."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    EditingPlan, Scene, SceneType, AspectRatio, EditingIntent,
    TokenUsage, WorkflowResult, EditError,
)


def test_coordinate_element_defaults():
    el = CoordinateElement(id="test", type="video")
    assert el.position.x == 0.5
    assert el.position.y == 0.5
    assert el.transform.opacity == 1.0
    assert el.timeline.start == 0.0
    assert el.timeline.end == 10.0


def test_timeline_duration():
    t = Timeline(start=5, end=15)
    assert t.duration == 10


def test_editing_plan_defaults():
    plan = EditingPlan()
    assert plan.duration == 30
    assert plan.intent == EditingIntent.AUTO_EDIT
    assert plan.scenes == []


def test_token_usage():
    usage = TokenUsage()
    usage.add_local(500)
    usage.add_api(300)
    assert usage.local_llm == 500
    assert usage.api_llm == 300
    assert usage.total == 800


def test_workflow_result():
    result = WorkflowResult(success=True)
    assert result.quality_score == 1.0
    assert result.processing_time == 0.0


def test_scene_creation():
    scene = Scene(id=1, scene_type=SceneType.ESTABLISHING, duration=8.0)
    assert scene.id == 1
    assert scene.duration == 8.0
