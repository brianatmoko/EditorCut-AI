"""Mandor AI — LLM decisions via FREE API gateways.

Menggunakan MOKOClient yang memanggil free API gateways
(opencode, omniroute, ninerouter) dengan format OpenAI-compatible.
TIDAK menggunakan local LLM (GGUF).

Fallback ke hardcoded responses jika semua gateway offline.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Callable
import json
import time
import logging

logger = logging.getLogger(__name__)

from ..models import (
    EditingPlan, EditingIntent, Scene, SceneType, TransitionType,
    VoiceoverConfig, VoiceoverSegment,
    TokenUsage, EditError, WorkflowResult,
    Mode, ConfidenceSource,
)

from moko_bridge.moko_client import MOKOClient


@dataclass
class Decision:
    """Single decision from the LLM with confidence scoring."""
    content: dict
    confidence: float
    source: ConfidenceSource
    token_cost: int = 0
    reasoning: Optional[str] = None

    def is_reliable(self) -> bool:
        thresholds = {
            ConfidenceSource.RULE_ENGINE: 0.0,
            ConfidenceSource.LOCAL_LLM: 0.7,
            ConfidenceSource.API_LLM: 0.9,
        }
        return self.confidence >= thresholds.get(self.source, 0.7)


def _try_extract_json(content: str) -> Optional[dict]:
    """Try to parse JSON from LLM response, handling markdown fences."""
    cleaned = content.strip()
    if cleaned.startswith("```"):
        cleaned = cleaned.split("\n", 1)[-1].rsplit("\n", 1)[0]
        if cleaned.endswith("```"):
            cleaned = cleaned[:-3]
    try:
        return json.loads(cleaned)
    except json.JSONDecodeError:
        return None


_MOCK_PLAN = {
    "intent": "auto_edit", "duration": 30, "style": "cinematic",
    "mood": "professional", "target_platform": "youtube",
    "voiceover": {"language": "id", "voice": "default", "speed": 1.0, "pitch": 1.0, "style": "narasi_tenang"},
    "scenes": [
        {"id": 1, "scene_type": "establishing", "duration": 8.0,
         "source": "auto_find", "source_keywords": ["establishing", "wide"],
         "transition_in": "hard_cut", "transition_out": "crossfade"},
        {"id": 2, "scene_type": "product", "duration": 12.0,
         "source": "auto_find", "source_keywords": ["product", "detail"],
         "transition_in": "crossfade", "transition_out": "crossfade"},
        {"id": 3, "scene_type": "b_roll", "duration": 10.0,
         "source": "auto_find", "source_keywords": ["action"],
         "transition_in": "crossfade", "transition_out": "fade_out"},
    ]
}


class MandorLLM:
    """Interface to local LLM for editing decisions via MOKO OS.

    Uses MOKOClient to call MOKO-AI-4B (port 11435) with OpenAI-compatible
    chat format. Falls back to hardcoded responses when MOKO is offline.
    """

    def __init__(self, mode: Mode = Mode.HYBRID):
        self.mode = mode
        self.token_usage = TokenUsage()
        self._confidence = 0.75
        self._moko = MOKOClient()

    def _available(self) -> bool:
        health = self._moko.check_health()
        return health.get("available", False)

    def analyze_brief(self, query: str, context: Optional[dict] = None) -> Decision:
        if not self._available():
            logger.info("MOKO offline — using fallback for analyze_brief")
            ctx = context or {}
            cost = 500
            self.token_usage.add_local(cost)
            return Decision(
                content=_MOCK_PLAN, confidence=self._confidence,
                source=ConfidenceSource.LOCAL_LLM, token_cost=cost,
                reasoning="Extracted editing parameters from user query (fallback)"
            )

        system = (
            "You are a professional video editor. Given a user's request, "
            "return a JSON editing plan with keys: intent, duration, style, "
            "mood, target_platform, voiceover (object with language, voice, speed, pitch, style), "
            "scenes (array of objects with id, scene_type, duration, source, source_keywords, "
            "transition_in, transition_out). "
            "Respond with ONLY valid JSON, no explanation."
        )
        ctx = context or {}
        prompt = json.dumps({"query": query, "context": ctx})
        result = self._moko.llm_generate(prompt, system_prompt=system, max_tokens=800, temperature=0.2)

        cost = result.get("tokens_used", 500) if result else 500
        self.token_usage.add_local(cost)

        if result and result.get("content"):
            parsed = _try_extract_json(result["content"])
            if parsed:
                return Decision(
                    content=parsed, confidence=0.85,
                    source=ConfidenceSource.LOCAL_LLM, token_cost=cost,
                    reasoning="Extracted editing parameters from user query via MOKO"
                )

        return Decision(
            content=_MOCK_PLAN, confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM, token_cost=cost,
            reasoning="Extracted editing parameters from user query (MOKO parse failed)"
        )

    def generate_script(self, plan: EditingPlan, topic: str = "") -> Decision:
        if not self._available():
            logger.info("MOKO offline — using fallback for generate_script")
            segments = []
            for scene in plan.scenes:
                segments.append({
                    "scene_id": scene.id,
                    "text": f"[Script for scene {scene.id}: {topic or 'content'}]",
                    "start": sum(s.duration for s in plan.scenes[:scene.id - 1]),
                    "end": sum(s.duration for s in plan.scenes[:scene.id]),
                })
            full_script = " ".join(s["text"] for s in segments)
            cost = 800
            self.token_usage.add_local(cost)
            return Decision(
                content={"full_script": full_script, "segments": segments,
                         "language": plan.voiceover.language if plan.voiceover else "id"},
                confidence=self._confidence, source=ConfidenceSource.LOCAL_LLM, token_cost=cost
            )

        system = (
            f"Write a {plan.duration}-second voiceover script in {plan.style.value} style. "
            f"Split into {len(plan.scenes)} scene-sized paragraphs. "
            "Return JSON with keys: full_script (string), segments (array of scene_id, text, start, end), "
            "language (string). Respond with ONLY valid JSON."
        )
        scene_desc = "\n".join(
            f"Scene {s.id}: {s.scene_type.value}, {s.duration}s" for s in plan.scenes
        )
        prompt = f"Topic: {topic or 'content'}\n\nScenes:\n{scene_desc}"
        result = self._moko.llm_generate(prompt, system_prompt=system, max_tokens=1000, temperature=0.5)

        cost = result.get("tokens_used", 800) if result else 800
        self.token_usage.add_local(cost)

        if result and result.get("content"):
            parsed = _try_extract_json(result["content"])
            if parsed:
                return Decision(
                    content=parsed, confidence=0.85,
                    source=ConfidenceSource.LOCAL_LLM, token_cost=cost
                )

        # fallback
        segments = []
        for scene in plan.scenes:
            segments.append({
                "scene_id": scene.id,
                "text": f"[Script for scene {scene.id}: {topic or 'content'}]",
                "start": sum(s.duration for s in plan.scenes[:scene.id - 1]),
                "end": sum(s.duration for s in plan.scenes[:scene.id]),
            })
        return Decision(
            content={"full_script": " ".join(s["text"] for s in segments),
                     "segments": segments,
                     "language": plan.voiceover.language if plan.voiceover else "id"},
            confidence=self._confidence, source=ConfidenceSource.LOCAL_LLM, token_cost=cost
        )

    def storyboard(self, plan: EditingPlan, assets: list[dict]) -> Decision:
        scenes = []
        for i, scene in enumerate(plan.scenes):
            matching_assets = [
                a for a in assets
                if any(kw in a.get("keywords", []) for kw in scene.source_keywords)
            ]
            best_asset = matching_assets[0] if matching_assets else None
            scenes.append({
                "scene_id": scene.id,
                "asset_url": best_asset.get("url") if best_asset else None,
                "asset_confidence": 0.85 if best_asset else 0.0,
                "suggested_duration": scene.duration,
                "notes": f"Scene {i+1}: {scene.scene_type.value}"
            })
        cost = 600
        self.token_usage.add_local(cost)
        return Decision(
            content={"scenes": scenes}, confidence=self._confidence,
            source=ConfidenceSource.LOCAL_LLM, token_cost=cost
        )

    def review(self, result: WorkflowResult) -> Decision:
        issues = []
        for error in result.errors:
            issues.append({
                "severity": "error" if not error.recoverable else "warning",
                "node_id": error.node_id, "description": error.message,
                "fix_suggestion": error.recovery_action or "manual review needed"
            })
        if not issues:
            issues.append({"severity": "info", "description": "No issues detected", "fix_suggestion": None})
        cost = 300
        self.token_usage.add_local(cost)
        passed = len([i for i in issues if i["severity"] == "error"]) == 0
        return Decision(
            content={"passed": passed, "quality_score": result.quality_score,
                     "issues": issues, "summary": f"Review {'passed' if passed else 'failed'}"},
            confidence=self._confidence, source=ConfidenceSource.LOCAL_LLM, token_cost=cost
        )

    def refine(self, issues: list[dict]) -> Decision:
        fixes = []
        for issue in issues:
            if issue.get("fix_suggestion"):
                fixes.append({
                    "target": issue.get("node_id", "unknown"),
                    "action": issue["fix_suggestion"],
                    "priority": "high" if issue.get("severity") == "error" else "medium"
                })
        cost = 400
        self.token_usage.add_local(cost)
        return Decision(
            content={"fixes": fixes, "requires_re_review": len(fixes) > 0},
            confidence=self._confidence, source=ConfidenceSource.LOCAL_LLM, token_cost=cost
        )

    def reset_token_usage(self) -> None:
        self.token_usage = TokenUsage()

    def get_token_usage(self) -> TokenUsage:
        return self.token_usage
