"""Demo: End-to-end auto-editing workflow.

This script demonstrates the complete pipeline without requiring
actual video files or GPU. Uses mock data for demonstration.

Usage:
    python -m auto_editor.workflow_demo
"""

from __future__ import annotations
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.intent_router import IntentRouter
from auto_editor.orchestrator.workflow_engine import WorkflowEngine, WorkflowNode
from auto_editor.orchestrator.template_db import TemplateDB
from auto_editor.orchestrator.mandor_llm import MandorLLM
from auto_editor.workers.layout_engine.coordinate import CoordinateEngine
from auto_editor.config.settings_loader import load_config
from auto_editor.models import *


def demo():
    """Run auto-editing workflow demo."""
    print("=" * 60)
    print("OpenCut AI Auto-Editor — Pipeline Demo")
    print("=" * 60)

    config = load_config()
    print(f"\n[1] Config loaded: mode={config.mode.value}")

    router = IntentRouter()
    plan = router.create_plan("buat video cinematic 30 detik tentang kopi")
    print(f"[2] Intent: {plan.intent.value}")
    print(f"    Duration: {plan.duration}s")
    print(f"    Style: {plan.style.value}")
    print(f"    Aspect Ratio: {plan.aspect_ratio.value}")

    llm = MandorLLM()
    analysis = llm.analyze_brief("buat video cinematic 30 detik tentang kopi")
    print(f"[3] LLM analysis: {analysis.confidence*100:.0f}% confidence")
    print(f"    Scenes planned: {len(analysis.content.get('scenes', []))}")

    db = TemplateDB()
    template = db.find_similar("cinematic")
    template_name = template.get("name", "cinematic") if template else "cinematic"
    print(f"[4] Template selected: '{template_name}'")

    elements = db.apply(template_name, {"TITLE": "KOPI NUSANTARA"})
    print(f"[5] Layout elements: {len(elements)}")
    for el in elements:
        print(f"    - {el.id} ({el.type}) at z={el.position.z}")

    coord = CoordinateEngine(1920, 1080)
    for el in elements:
        bounds = coord.get_bounds(el)
        print(f"[6] {el.id}: {bounds['width']:.0f}x{bounds['height']:.0f}px "
              f"@ ({bounds['left']:.0f},{bounds['top']:.0f})")

    engine = WorkflowEngine()
    track: list[WorkflowNode] = [
        WorkflowNode(id="analyze", handler=lambda **_: {"status": "done"}),
        WorkflowNode(id="find_assets", handler=lambda **_: {"assets": []}, deps=["analyze"]),
        WorkflowNode(id="layout", handler=lambda **_: {"elements": elements}, deps=["find_assets"]),
        WorkflowNode(id="review", handler=lambda **_: {"passed": True}, deps=["layout"]),
    ]
    engine.register("demo_workflow", track)

    result = engine.run("demo_workflow", {"plan": plan})
    print(f"[8] Workflow result: {'SUCCESS' if result.success else 'FAILED'}")
    print(f"    Quality score: {result.quality_score:.2f}")
    print(f"    Processing time: {result.processing_time:.2f}s")
    print(f"    Token usage: {result.token_usage.total}")

    total_tokens = result.token_usage.total
    print(f"\n{'=' * 60}")
    print(f"TOKEN EFFICIENCY REPORT")
    print(f"{'=' * 60}")
    print(f"  Total tokens: {total_tokens}")
    print(f"  Mode: {config.mode.value}")
    print(f"  Estimated API cost: ${total_tokens * 0.00015:.4f}")

    savings = 5000 - total_tokens
    if savings > 0:
        print(f"  Token saved vs baseline (5K): {savings} ({savings/5000*100:.0f}%)")
    print(f"{'=' * 60}")

    return 0


if __name__ == "__main__":
    sys.exit(demo())
