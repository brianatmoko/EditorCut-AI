"""Custom inference presets for auto-editing tasks.

Wraps MOKOClient with auto-editor specific prompt templates
and response parsers.
"""

from __future__ import annotations
from typing import Optional
import json

from .moko_client import MOKOClient
from auto_editor.models import (
    EditingPlan, EditingIntent, Scene, SceneType,
    Mode, TokenUsage,
)


class MOKOInference:
    """Specialized inference methods for auto-editing.

    Wraps MOKOClient with:
    - Prompt templates for each editing phase
    - Response parsers for structured output
    - Confidence scoring
    - Token tracking
    """

    def __init__(self, client: Optional[MOKOClient] = None):
        self.client = client or MOKOClient()
        self.token_usage = TokenUsage()

    def analyze_brief(self, query: str) -> tuple[EditingPlan, float]:
        """Analyze editing brief and return structured plan.

        Args:
            query: User's editing request in natural language.

        Returns:
            Tuple of (EditingPlan, confidence_score).
        """
        response = self.client.analyze_brief(query)

        self.token_usage.add_local(response.get("tokens_used", 0))

        plan = EditingPlan(
            intent=EditingIntent(response.get("intent", "auto_edit")),
            duration=response.get("duration", 30),
            style=self._parse_style(response.get("style", "cinematic")),
            mood=self._parse_mood(response.get("mood", "professional")),
            target_platform=self._parse_platform(response.get("target_platform", "youtube")),
        )

        scene_count = response.get("scene_count", 3)
        for i in range(scene_count):
            plan.scenes.append(Scene(
                id=i + 1,
                scene_type=SceneType.B_ROLL,
                duration=plan.duration / scene_count,
            ))

        confidence = response.get("confidence", 0.5)
        return plan, confidence

    def generate_script(self, plan: EditingPlan, topic: str) -> tuple[str, float]:
        """Generate voiceover script from plan.

        Returns:
            Tuple of (script_text, confidence).
        """
        script = self.client.generate_script(topic, plan.duration, plan.style.value)
        self.token_usage.add_local(len(script.split()))
        return script, 0.7 if script else 0.0

    def review_project(self, result) -> tuple[bool, list[str], float]:
        """Review project quality.

        Args:
            result: WorkflowResult from pipeline.

        Returns:
            Tuple of (passed, issues_list, quality_score).
        """
        summary = {
            "duration": result.processing_time,
            "errors": [e.message for e in result.errors],
            "quality_score": result.quality_score,
            "token_usage": result.token_usage.total,
        }

        review = self.client.quality_review(summary)
        self.token_usage.add_local(review.get("tokens_used", 0))

        issues = [i.get("description", str(i)) for i in review.get("issues", [])]
        return review.get("passed", True), issues, review.get("score", 1.0)

    def auto_complete_layout(self, plan: EditingPlan) -> list[dict]:
        """AI-suggested layout improvements."""
        system = (
            "Suggest layout improvements for this video project. "
            "Return JSON array of {element_id: str, suggestion: str, reason: str}. "
            "Keep suggestions minimal and practical."
        )
        prompt = json.dumps({
            "duration": plan.duration,
            "style": plan.style.value,
            "scenes": len(plan.scenes),
        })

        result = self.client.llm_generate(prompt, system_prompt=system, max_tokens=300, temperature=0.5)
        self.token_usage.add_local(result.get("tokens_used", 0) if result else 0)

        if result and result.get("content"):
            try:
                return json.loads(result["content"])
            except json.JSONDecodeError:
                pass
        return []

    def _parse_style(self, value: str):
        from auto_editor.models import EditingStyle
        mapping = {
            "cinematic": EditingStyle.CINEMATIC,
            "vlog": EditingStyle.VLOG,
            "tutorial": EditingStyle.TUTORIAL,
            "product": EditingStyle.PRODUCT,
        }
        return mapping.get(value, EditingStyle.CUSTOM)

    def _parse_mood(self, value: str):
        from auto_editor.models import Mood
        mapping = {
            "professional": Mood.PROFESSIONAL,
            "fun": Mood.FUN,
            "serious": Mood.SERIOUS,
        }
        return mapping.get(value, Mood.PROFESSIONAL)

    def _parse_platform(self, value: str):
        from auto_editor.models import Platform
        mapping = {
            "tiktok": Platform.TIKTOK,
            "youtube": Platform.YOUTUBE,
            "instagram": Platform.INSTAGRAM,
        }
        return mapping.get(value, Platform.CUSTOM)

    def get_token_usage(self) -> TokenUsage:
        """Get accumulated token usage."""
        return self.token_usage

    def reset_token_usage(self) -> None:
        """Reset token counter."""
        self.token_usage = TokenUsage()
