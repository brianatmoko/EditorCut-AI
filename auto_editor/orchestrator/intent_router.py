"""Classify user editing commands using rule-based pattern matching.

Rule-based = 0 token cost. Falls back to UNKNOWN intent when no pattern matches.
"""

from __future__ import annotations
import re
from typing import Optional
from ..models import AspectRatio, EditingIntent, EditingPlan, EditingStyle, Platform


# ─── Parameter Extractors ─────────────────────────────────────────

def _extract_auto_edit_params(match: re.Match, query: str) -> dict:
    return {
        "has_duration": bool(re.search(r'\d+\s*(detik|second|menit)', query, re.IGNORECASE)),
        "has_style": bool(re.search(r'(cinematic|vlog|tutorial|product)', query, re.IGNORECASE)),
    }

def _extract_voiceover_params(match: re.Match, query: str) -> dict:
    lang = "id"
    if re.search(r'\b(english|inggris|en\b)', query, re.IGNORECASE):
        lang = "en"
    return {"language": lang}

def _extract_subtitle_params(match: re.Match, query: str) -> dict:
    lang = "id"
    if re.search(r'\b(english|inggris|en\b)', query, re.IGNORECASE):
        lang = "en"
    return {"language": lang}

def _extract_trim_params(match: re.Match, query: str) -> dict:
    return {}

def _extract_layout_params(match: re.Match, query: str) -> dict:
    return {}

def _extract_effects_params(match: re.Match, query: str) -> dict:
    return {}

def _extract_render_params(match: re.Match, query: str) -> dict:
    return {}

def _extract_batch_params(match: re.Match, query: str) -> dict:
    return {}


_INTENT_PATTERNS = [
    (re.compile(
        r'\b(buat|bikin|buatkan|hasilkan|create|make|generate)\s.*\b(video|konten|content)\b',
        re.IGNORECASE
    ), EditingIntent.AUTO_EDIT, _extract_auto_edit_params),
    (re.compile(
        r'\bvideo\b.*\b(\d+)\s*(detik|second|menit|minute|min)\b',
        re.IGNORECASE
    ), EditingIntent.AUTO_EDIT, _extract_auto_edit_params),
    (re.compile(
        r'\b(voiceover|narasi|dubbing|suara|audio|voice)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_VOICEOVER, _extract_voiceover_params),
    (re.compile(
        r'\b(subtitle|teks|caption|takarir|terjemahan|srt)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_SUBTITLE, _extract_subtitle_params),
    (re.compile(
        r'\b(potong|trim|cut|hapus|remove|buang)\b',
        re.IGNORECASE
    ), EditingIntent.TRIM, _extract_trim_params),
    (re.compile(
        r'\b(layout|tata letak|posisi|template|templat|susun|atur|arrange)\b',
        re.IGNORECASE
    ), EditingIntent.CHANGE_LAYOUT, _extract_layout_params),
    (re.compile(
        r'\b(efek|filter|transisi|color|warna|grading|effect)\b',
        re.IGNORECASE
    ), EditingIntent.ADD_EFFECTS, _extract_effects_params),
    (re.compile(
        r'\b(batch|semua|all|massal|render\s+semua)\b',
        re.IGNORECASE
    ), EditingIntent.BATCH_RENDER, _extract_batch_params),
    (re.compile(
        r'\b(render|export|simpan|download|save|publikasi|publish)\b',
        re.IGNORECASE
    ), EditingIntent.RENDER, _extract_render_params),
]


class IntentRouter:
    """Route user commands to the correct editing intent using pattern matching."""

    def classify(self, query: str) -> tuple[EditingIntent, dict]:
        if not query or not query.strip():
            return EditingIntent.UNKNOWN, {}
        for pattern, intent, extractor in _INTENT_PATTERNS:
            match = pattern.search(query)
            if match:
                params = extractor(match, query)
                return intent, params
        return EditingIntent.UNKNOWN, {}

    def extract_duration(self, query: str) -> Optional[int]:
        patterns = [
            r'(\d+)\s*(detik|second|s\b)',
            r'(\d+)\s*(menit|minute|min|m\b)',
            r'durasi?\s*(\d+)',
        ]
        for pat in patterns:
            match = re.search(pat, query, re.IGNORECASE)
            if match:
                val = int(match.group(1))
                unit = match.group(2).lower() if len(match.groups()) > 1 else 'detik'
                if unit in ('menit', 'minute', 'min', 'm'):
                    val *= 60
                return val
        return None

    def extract_style(self, query: str) -> Optional[str]:
        style_keywords = {
            'cinematic': ['cinematic', 'film', 'movie', 'sinematik'],
            'vlog': ['vlog', 'daily', 'harian'],
            'tutorial': ['tutorial', 'guide', 'panduan'],
            'product': ['product', 'produk', 'review', 'unboxing'],
            'music': ['music', 'musik', 'lyric', 'lirik'],
        }
        query_lower = query.lower()
        for style, keywords in style_keywords.items():
            if any(kw in query_lower for kw in keywords):
                return style
        return None

    def extract_platform(self, query: str) -> Optional[str]:
        platform_keywords = {
            'tiktok': ['tiktok', 'tk'],
            'youtube': ['youtube', 'yt'],
            'instagram': ['instagram', 'ig', 'reels'],
            'shorts': ['shorts', 'short'],
        }
        query_lower = query.lower()
        for platform, keywords in platform_keywords.items():
            if any(kw in query_lower for kw in keywords):
                return platform
        return None

    def extract_aspect_ratio(self, query: str) -> Optional[str]:
        ratio_patterns = [
            (r'16\s*[:\/]\s*9', '16:9'),
            (r'9\s*[:\/]\s*16', '9:16'),
            (r'1\s*[:\/]\s*1', '1:1'),
            (r'4\s*[:\/]\s*3', '4:3'),
            (r'21\s*[:\/]\s*9', '21:9'),
            (r'\b(vertikal|vertical|potrait|tiktok)\b', '9:16'),
            (r'\b(horizontal|landscape|youtube|cinematic)\b', '16:9'),
            (r'\b(persegi|square|instagram)\b', '1:1'),
        ]
        query_lower = query.lower()
        for pat, ratio in ratio_patterns:
            if re.search(pat, query_lower):
                return ratio
        return None

    def create_plan(self, query: str) -> EditingPlan:
        intent, _ = self.classify(query)
        plan = EditingPlan(
            intent=intent,
            duration=self.extract_duration(query) or 30,
            style=self._parse_style(self.extract_style(query)),
            target_platform=self._parse_platform(self.extract_platform(query)),
        )
        ratio = self.extract_aspect_ratio(query)
        if ratio:
            plan.aspect_ratio = self._parse_aspect_ratio(ratio)
        elif plan.target_platform in (Platform.TIKTOK, Platform.REELS, Platform.SHORTS):
            plan.aspect_ratio = AspectRatio.RATIO_9_16
        elif plan.target_platform == Platform.INSTAGRAM:
            plan.aspect_ratio = AspectRatio.RATIO_1_1
        else:
            plan.aspect_ratio = AspectRatio.RATIO_16_9
        return plan

    def _parse_style(self, style: Optional[str]) -> EditingStyle:
        mapping = {
            'cinematic': EditingStyle.CINEMATIC,
            'vlog': EditingStyle.VLOG,
            'tutorial': EditingStyle.TUTORIAL,
            'product': EditingStyle.PRODUCT,
            'music': EditingStyle.MUSIC,
        }
        return mapping.get(style, EditingStyle.CUSTOM)

    def _parse_platform(self, platform: Optional[str]) -> Platform:
        mapping = {
            'tiktok': Platform.TIKTOK,
            'youtube': Platform.YOUTUBE,
            'instagram': Platform.INSTAGRAM,
            'shorts': Platform.SHORTS,
        }
        return mapping.get(platform, Platform.CUSTOM)

    def _parse_aspect_ratio(self, ratio: str) -> AspectRatio:
        mapping = {
            '16:9': AspectRatio.RATIO_16_9,
            '9:16': AspectRatio.RATIO_9_16,
            '1:1': AspectRatio.RATIO_1_1,
            '4:3': AspectRatio.RATIO_4_3,
            '21:9': AspectRatio.RATIO_21_9,
        }
        return mapping.get(ratio, AspectRatio.RATIO_16_9)
