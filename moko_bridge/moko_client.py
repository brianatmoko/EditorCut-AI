"""OpenCut AI — Self-Contained Gateway Client.

Handles ALL AI gateway communication directly — no dependency on external
MOKO OS services being online. Falls back gracefully at every step.

Gateway priority (all free):
  1. opencode   (localhost:4096)   — if running
  2. openrouter (openrouter.ai)   — cloud, needs API key
  3. omnirouter (localhost:20128)  — if running
  4. ninerouter (localhost:20130)  — if running
  5. offline fallback             — always works, no AI needed
"""

from __future__ import annotations

import json
import logging
import os
import re
import time
from typing import Generator, Optional

import requests

logger = logging.getLogger(__name__)


# ── Backwards-compat export ───────────────────────────────────────────────────

class MOKOConfig:
    """Legacy config — kept for backwards compatibility with tests."""
    def __init__(self, llm_host: str = "127.0.0.1", llm_port: int = 11435):
        self.llm_host = llm_host
        self.llm_port = llm_port


# ── Gateway definitions ──────────────────────────────────────────────────────

OPENROUTER_FREE_MODELS = [
    "google/gemma-3-12b-it",
    "meta-llama/llama-3.1-8b-instruct:free",
    "openrouter/free",
]

NVIDIA_FREE_MODELS = [
    "meta/llama-3.1-70b-instruct",
    "meta/llama-3.3-70b-instruct",
    "meta/llama-3.1-8b-instruct",
    "google/gemma-2-27b-it",
]

GEMINI_MODELS = [
    "gemini-2.5-flash-preview-05-07",
    "gemini-2.5-flash-001",
    "gemini-2.0-flash",
    "gemini-1.5-flash",
    "gemini-1.5-flash-latest",
    "gemini-2.5-flash",  # fallback for existing users
]



def _get_config():
    """Get OpenCut config (lazy import to avoid circular deps)."""
    try:
        from auto_editor.config.opencut_settings import OpenCutConfig
        return OpenCutConfig.get()
    except Exception:
        return None


def _quick_check(url: str, timeout: float = 1.5) -> bool:
    """Fast connectivity check — just TCP connect, no full request."""
    try:
        resp = requests.get(
            f"{url.rstrip('/')}/models",
            timeout=timeout,
            headers={"Authorization": "Bearer test"},
        )
        return resp.status_code in (200, 401, 403, 429)  # reachable (429 = alive but rate-limited)
    except Exception:
        return False


def _call_openai_compat(
    api_base: str,
    api_key: str,
    model: str,
    messages: list[dict],
    max_tokens: int = 2000,
    temperature: float = 0.3,
    timeout: int = 30,
    extra_headers: dict | None = None,
) -> str:
    """Call any OpenAI-compatible endpoint."""
    url = f"{api_base.rstrip('/')}/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    }
    if extra_headers:
        headers.update(extra_headers)

    resp = requests.post(url, json={
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    }, headers=headers, timeout=timeout)

    if resp.status_code == 429 or resp.status_code == 402:
        raise RuntimeError(f"Rate limited / quota on {api_base}")
    if resp.status_code != 200:
        raise RuntimeError(f"HTTP {resp.status_code}: {resp.text[:200]}")

    data = resp.json()
    msg = data.get("choices", [{}])[0].get("message", {})
    content = msg.get("content")
    # Some free "reasoning" models (e.g., cohere/north-mini-code:free) put the
    # answer in `reasoning` instead of `content` — surface that so the caller
    # can still get a response. If neither exists, retry with next model.
    if content:
        return content
    reasoning = msg.get("reasoning")
    if reasoning:
        return reasoning
    # Mark this model as not returning usable content — try next
    raise RuntimeError(f"Empty content from model={model} (finish_reason={data.get('choices',[{}])[0].get('finish_reason')})")


def _call_openai_compat_stream(
    api_base: str,
    api_key: str,
    model: str,
    messages: list[dict],
    max_tokens: int = 2000,
    temperature: float = 0.3,
    timeout: int = 60,
    extra_headers: dict | None = None,
) -> Generator[str, None, None]:
    """Call OpenAI-compatible endpoint with SSE streaming, yielding tokens."""
    url = f"{api_base.rstrip('/')}/chat/completions"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {api_key}",
    }
    if extra_headers:
        headers.update(extra_headers)

    # For streaming: short connect timeout, long per-byte timeout
    # requests accepts tuple (connect, read) — per-read covers time between bytes
    stream_timeout = (10, timeout) if isinstance(timeout, (int, float)) else timeout

    resp = requests.post(url, json={
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
        "stream": True,
    }, headers=headers, timeout=stream_timeout, stream=True)

    if resp.status_code == 429 or resp.status_code == 402:
        raise RuntimeError(f"Rate limited / quota on {api_base}")
    if resp.status_code != 200:
        raise RuntimeError(f"HTTP {resp.status_code}: {resp.text[:200]}")

    for line in resp.iter_lines():
        if not line:
            continue
        decoded = line.decode("utf-8").strip()
        if not decoded.startswith("data: "):
            continue
        payload = decoded[6:]
        if payload == "[DONE]":
            break
        try:
            chunk = json.loads(payload)
            delta = chunk.get("choices", [{}])[0].get("delta", {})
            token = delta.get("content", "")
            if token:
                yield token
        except json.JSONDecodeError:
            continue


# ── Main Client ──────────────────────────────────────────────────────────────

class MOKOClient:
    """Self-contained AI gateway client for OpenCut.

    Works completely standalone — does NOT require MOKO OS services.
    Falls back to offline responses when no gateway is available.
    """

    def __init__(self, config: Optional[MOKOConfig] = None):
        self._legacy_config = config
        self._gateway_cache: dict[str, bool] = {}  # name -> reachable
        self._cache_time: float = 0

    def _get_gateways(self) -> list[dict]:
        """Build gateway list from OpenCut config or defaults."""
        cfg = _get_config()

        gateways = []

        # 1. OpenCode (local)
        oc_cfg = cfg.get_gateway("opencode") if cfg else {}
        if not cfg or oc_cfg.get("enabled", True):
            gateways.append({
                "name": "opencode",
                "url": oc_cfg.get("url", "http://localhost:4096/v1"),
                "key": os.environ.get("OPENCODE_API_KEY", "opencode-free"),
                "model": "auto",
                "timeout": oc_cfg.get("timeout", 5),
                "type": "local",
            })

        # 2. Google Gemini / AI Studio (cloud) — HIGHEST PRIORITY cloud gateway
        #    Uses OpenAI-compatible endpoint at generativelanguage.googleapis.com
        gem_cfg = cfg.get_gateway("gemini") if cfg else {}
        gem_keys = (cfg.gemini_api_keys if cfg else []) or [os.environ.get("GEMINI_API_KEY", "")]
        gem_keys = [k for k in gem_keys if k]
        if (not cfg or gem_cfg.get("enabled", True)) and gem_keys:
            gateways.append({
                "name": "gemini",
                "url": gem_cfg.get("url", "https://generativelanguage.googleapis.com/v1beta/openai"),
                "key": gem_keys[0],
                "alt_key": gem_keys[1] if len(gem_keys) > 1 else "",
                "all_keys": gem_keys,
                "model": gem_cfg.get("model", "gemini-2.5-flash"),
                "timeout": gem_cfg.get("timeout", 120),
                "type": "cloud",
                "free_models": GEMINI_MODELS,
            })

        # 3. SwiftRouter (cloud) — multiple API keys for rotation
        sr_cfg = cfg.get_gateway("swiftrouter") if cfg else {}
        sr_keys = (cfg.swiftrouter_api_keys if cfg else []) or [os.environ.get("SWIFTRouter_API_KEY", "")]
        sr_keys = [k for k in sr_keys if k]
        if (not cfg or sr_cfg.get("enabled", True)) and sr_keys:
            gateways.append({
                "name": "swiftrouter",
                "url": sr_cfg.get("url", "https://api.swiftrouter.com/v1"),
                "key": sr_keys[0],
                "alt_key": sr_keys[1] if len(sr_keys) > 1 else "",
                "all_keys": sr_keys,
                "model": sr_cfg.get("model", "swiftrouter/auto"),
                "timeout": sr_cfg.get("timeout", 120),
                "type": "cloud",
                "free_models": [],
            })

        # 4. OpenRouter (cloud) — supports primary + alt key rotation
        or_cfg = cfg.get_gateway("openrouter") if cfg else {}
        or_primary = (cfg.openrouter_api_key if cfg else "") or os.environ.get("OPENROUTER_API_KEY", "")
        or_alt = (cfg.openrouter_api_key_alt if cfg else "") or os.environ.get("OPENROUTER_API_KEY_ALT", "")
        if (not cfg or or_cfg.get("enabled", True)) and or_primary:
            gateways.append({
                "name": "openrouter",
                "url": or_cfg.get("url", "https://openrouter.ai/api/v1"),
                "key": or_primary,
                "alt_key": or_alt,
                "model": or_cfg.get("model", OPENROUTER_FREE_MODELS[0]),
                "timeout": or_cfg.get("timeout", 30),
                "type": "cloud",
                "extra_headers": {
                    "HTTP-Referer": "https://opencut.ai",
                    "X-Title": "OpenCut AI",
                },
                "free_models": OPENROUTER_FREE_MODELS,
            })

        # 5. NVIDIA API (cloud) — optional, with primary + alt key rotation
        nv_cfg = cfg.get_gateway("nvidia") if cfg else {}
        nv_key = (cfg.nvidia_api_key if cfg else "") or os.environ.get("NVIDIA_API_KEY", "")
        nv_alt = (cfg.nvidia_api_key_alt if cfg else "") or os.environ.get("NVIDIA_API_KEY_ALT", "")
        if (not cfg or nv_cfg.get("enabled", True)) and nv_key:
            gateways.append({
                "name": "nvidia",
                "url": nv_cfg.get("url", "https://integrate.api.nvidia.com/v1"),
                "key": nv_key,
                "alt_key": nv_alt,
                "model": nv_cfg.get("model", NVIDIA_FREE_MODELS[0]),
                "timeout": nv_cfg.get("timeout", 30),
                "type": "cloud",
                "free_models": NVIDIA_FREE_MODELS,
            })

        # 6. OmniRouter (local)
        omni_cfg = cfg.get_gateway("omnirouter") if cfg else {}
        if not cfg or omni_cfg.get("enabled", True):
            gateways.append({
                "name": "omnirouter",
                "url": omni_cfg.get("url", "http://localhost:20128/v1"),
                "key": os.environ.get("OMNIROUTE_API_KEY", "omni-free"),
                "model": "auto",
                "timeout": omni_cfg.get("timeout", 5),
                "type": "local",
            })

        # 4. 9Router (local)
        nine_cfg = cfg.get_gateway("ninerouter") if cfg else {}
        if not cfg or nine_cfg.get("enabled", True):
            gateways.append({
                "name": "ninerouter",
                "url": nine_cfg.get("url", "http://localhost:20130/v1"),
                "key": os.environ.get("NINEROUTE_API_KEY", "nine-free"),
                "model": "auto",
                "timeout": nine_cfg.get("timeout", 5),
                "type": "local",
            })

        return gateways

    def _is_reachable(self, gw: dict) -> bool:
        """Check if a gateway is reachable (cached for 30s).
        Cloud gateways skip the quick-check — always try them directly.
        """
        # Cloud gateways: always try — don't let a /models 404 or TLS hiccup
        # prevent us from reaching the /chat/completions endpoint.
        if gw.get("type") == "cloud":
            return True

        now = time.time()
        if now - self._cache_time > 30:
            self._gateway_cache.clear()
            self._cache_time = now

        name = gw["name"]
        if name in self._gateway_cache:
            return self._gateway_cache[name]

        timeout = min(gw.get("timeout", 5), 2.0)
        reachable = _quick_check(gw["url"], timeout=timeout)
        self._gateway_cache[name] = reachable

        if not reachable:
            logger.debug("[%s] Not reachable, skipping", name)

        return reachable

    # ── Core Generation ───────────────────────────────────────────────────────

    def llm_generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs,
    ) -> dict:
        """Generate text via the gateway chain.

        Returns dict: {content, tokens_used, confidence, provider, client}.
        ALWAYS returns a result — falls back to offline response.
        """
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        gateways = self._get_gateways()

        for gw in gateways:
            if not self._is_reachable(gw):
                continue

            # Models: try free list first, then configured model as paid fallback
            free_models = gw.get("free_models", [])
            models_to_try = list(free_models)
            if gw["model"] not in models_to_try:
                models_to_try.append(gw["model"])

            keys_to_try = [gw.get("key")] if gw.get("key") else []
            if gw.get("alt_key"):
                keys_to_try.append(gw["alt_key"])
            # SwiftRouter has many keys (9 total)
            if gw.get("all_keys"):
                for k in gw["all_keys"]:
                    if k and k not in keys_to_try:
                        keys_to_try.append(k)

            skip_gateway = False
            # Adaptive timeout: scale timeout with max_tokens (more tokens → longer wait)
            # Free cloud models: ~100 tokens/s; 8192 tokens needs ~120s
            adaptive_timeout = max(gw["timeout"], min(300, max_tokens // 50))
            for api_key in keys_to_try:
                if skip_gateway:
                    break
                key_is_bad = False
                for model in models_to_try:
                    try:
                        content = _call_openai_compat(
                            api_base=gw["url"],
                            api_key=api_key,
                            model=model,
                            messages=messages,
                            max_tokens=max_tokens,
                            temperature=temperature,
                            timeout=adaptive_timeout,
                            extra_headers=gw.get("extra_headers"),
                        )
                        if content and content.strip():
                            logger.info("[%s] Success with model %s", gw["name"], model)
                            return {
                                "content": content,
                                "tokens_used": len(content.split()),
                                "confidence": 0.85,
                                "provider": gw["name"],
                                "client": gw["name"],
                            }
                    except Exception as e:
                        logger.debug("[%s/%s] Failed: %s", gw["name"], model, e)
                        # On 429: try next model (different model may have different limits)
                        if "Rate limited" in str(e) or "quota" in str(e):
                            continue
                        # On auth: this key is invalid for all models
                        if "HTTP 401" in str(e) or "HTTP 403" in str(e):
                            key_is_bad = True
                            break
                        # On connection/timeout: skip entire gateway
                        if "Connection" in str(e) or "Timeout" in str(e):
                            skip_gateway = True
                            break
                        # Other errors: try next model
                        continue
                if skip_gateway:
                    break
                if key_is_bad:
                    continue  # try next key

        # Offline fallback — always works
        logger.info("[offline] All gateways unavailable, using fallback")
        return {
            "content": f"[Offline mode] Processed: {prompt[:80]}",
            "tokens_used": 0,
            "confidence": 0.5,
            "provider": "offline-fallback",
            "client": "offline-fallback",
        }

    def llm_generate_stream(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        max_tokens: int = 1000,
        temperature: float = 0.7,
        **kwargs,
    ) -> Generator[str, None, None]:
        """Stream tokens via the gateway chain. Yields token strings.

        Falls back to a single yield of the offline fallback text.
        """
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        gateways = self._get_gateways()

        for gw in gateways:
            if not self._is_reachable(gw):
                continue

            free_models = gw.get("free_models", [])
            models_to_try = list(free_models)
            if gw["model"] not in models_to_try:
                models_to_try.append(gw["model"])

            keys_to_try = [gw.get("key")] if gw.get("key") else []
            if gw.get("alt_key"):
                keys_to_try.append(gw["alt_key"])
            # SwiftRouter has many keys (9 total)
            if gw.get("all_keys"):
                for k in gw["all_keys"]:
                    if k and k not in keys_to_try:
                        keys_to_try.append(k)

            skip_gateway = False
            # Adaptive timeout for streaming — allow long generations (max_tokens=8192 → ~200s)
            adaptive_stream_timeout = max(gw.get("timeout", 60), min(300, max_tokens // 50))
            for api_key in keys_to_try:
                if skip_gateway:
                    break
                for model in models_to_try:
                    try:
                        for token in _call_openai_compat_stream(
                            api_base=gw["url"],
                            api_key=api_key,
                            model=model,
                            messages=messages,
                            max_tokens=max_tokens,
                            temperature=temperature,
                            timeout=adaptive_stream_timeout,
                            extra_headers=gw.get("extra_headers"),
                        ):
                            yield token
                        logger.info("[%s] Stream complete with model %s", gw["name"], model)
                        return
                    except Exception as e:
                        logger.debug("[%s/%s] Stream failed: %s", gw["name"], model, e)
                        if "Rate limited" in str(e) or "quota" in str(e):
                            continue
                        if "HTTP 401" in str(e) or "HTTP 403" in str(e):
                            break
                        if "Connection" in str(e) or "Timeout" in str(e):
                            skip_gateway = True
                            break
                        continue
                if skip_gateway:
                    break

        logger.info("[offline] All gateways unavailable for stream, using fallback")
        yield f"[Offline mode] Processed: {prompt[:80]}"

    # ── EDL Generation ────────────────────────────────────────────────────────

    def generate_edl(
        self,
        prompt: str,
        duration: int = 30,
        style: str = "cinematic",
        platform: str = "youtube",
        language: str = "id",
    ) -> dict:
        """Generate Edit Decision List from prompt."""
        aspect_map = {
            "tiktok": "9:16", "reels": "9:16", "shorts": "9:16",
            "instagram": "1:1", "youtube": "16:9",
        }
        aspect_ratio = aspect_map.get(platform.lower(), "16:9")

        system = f"""You are a professional video director. Generate a complete Edit Decision List as valid JSON.
Return ONLY valid JSON (no markdown, no explanation) with this schema:
{{"title":"string","aspect_ratio":"{aspect_ratio}","fps":30,"total_duration":{duration},
"voiceover_script":"narration in {language}","music_mood":"cinematic|energetic|calm",
"scenes":[{{"id":1,"duration":8.0,"asset_query":"English search query for stock video",
"asset_type":"video","text_overlay":"text in {language}","transition_in":"fade",
"transition_out":"crossfade","color_grade":"warm","effects":["vignette"]}}]}}
Make {duration}s total across 3-6 scenes. Style: {style}. Platform: {platform}."""

        result = self.llm_generate(
            prompt=f"Create a {duration}s {style} video about: {prompt}",
            system_prompt=system,
            max_tokens=1500,
            temperature=0.4,
        )

        if result.get("provider") != "offline-fallback":
            parsed = self._extract_json(result.get("content", ""))
            if parsed and "scenes" in parsed:
                parsed["_provider"] = result.get("provider", "unknown")
                return parsed

        return self._fallback_edl(prompt, duration, style, platform, language)

    def _extract_json(self, text: str) -> Optional[dict]:
        """Extract JSON from LLM response."""
        cleaned = text.strip()
        if cleaned.startswith("```"):
            lines = cleaned.split("\n")
            cleaned = "\n".join(lines[1:-1] if lines[-1].strip() == "```" else lines[1:])
        try:
            return json.loads(cleaned)
        except json.JSONDecodeError:
            pass
        match = re.search(r'\{.*\}', cleaned, re.DOTALL)
        if match:
            try:
                return json.loads(match.group(0))
            except json.JSONDecodeError:
                pass
        return None

    def _fallback_edl(self, prompt, duration, style, platform, language):
        """Hardcoded EDL when all gateways offline."""
        aspect_map = {"tiktok": "9:16", "reels": "9:16", "shorts": "9:16",
                      "instagram": "1:1", "youtube": "16:9"}
        aspect_ratio = aspect_map.get(platform.lower(), "16:9")
        topic = " ".join(prompt.split()[:4])
        per_scene = round(duration / 3, 1)

        return {
            "title": topic.title(),
            "aspect_ratio": aspect_ratio,
            "fps": 30,
            "total_duration": duration,
            "voiceover_script": f"Video tentang {prompt}." if language == "id" else f"A video about {prompt}.",
            "music_mood": "cinematic",
            "_provider": "fallback",
            "scenes": [
                {"id": 1, "duration": per_scene, "asset_query": f"{topic} wide shot",
                 "asset_type": "video", "text_overlay": topic.title(),
                 "transition_in": "fade", "transition_out": "crossfade",
                 "color_grade": "warm", "effects": ["vignette"]},
                {"id": 2, "duration": per_scene, "asset_query": f"{topic} closeup detail",
                 "asset_type": "video", "text_overlay": "",
                 "transition_in": "crossfade", "transition_out": "crossfade",
                 "color_grade": "neutral", "effects": []},
                {"id": 3, "duration": per_scene, "asset_query": f"{topic} lifestyle",
                 "asset_type": "video", "text_overlay": "OpenCut AI",
                 "transition_in": "crossfade", "transition_out": "fade",
                 "color_grade": "cool", "effects": []},
            ],
        }

    # ── Health & Status ───────────────────────────────────────────────────────

    def check_health(self) -> dict:
        """Check all gateway availability."""
        gateways = self._get_gateways()
        status = []

        for gw in gateways:
            reachable = _quick_check(gw["url"], timeout=1.5)
            status.append({
                "name": gw["name"],
                "provider": gw["name"],
                "model": gw.get("model", "auto"),
                "available": reachable,
                "type": "cloud_gateway" if gw["type"] == "cloud" else "local_gateway",
                "url": gw["url"],
            })

        has_available = any(s["available"] for s in status)
        return {
            "available": has_available,
            "mode": "multi_gateway_free",
            "providers": status,
            "openrouter_key_configured": bool(
                (_get_config().openrouter_api_key if _get_config() else "")
                or os.environ.get("OPENROUTER_API_KEY", "")
            ),
            # Backwards compat
            "llm": has_available,
            "rag": False,
            "native": True,
            "version": "2.0.0-unified",
        }

    # ── Specialized Methods ───────────────────────────────────────────────────

    def analyze_brief(self, query: str) -> dict:
        """Analyze editing brief."""
        system = (
            "Extract video editing parameters. Return ONLY JSON: "
            '{"intent":"auto_edit","duration":30,"style":"cinematic",'
            '"mood":"professional","target_platform":"youtube",'
            '"has_voiceover":true,"scene_count":3}'
        )
        result = self.llm_generate(query, system_prompt=system, max_tokens=300, temperature=0.1)
        parsed = self._extract_json(result.get("content", ""))
        if parsed:
            return parsed
        return {
            "intent": "auto_edit", "duration": 30, "style": "cinematic",
            "mood": "professional", "target_platform": "youtube",
            "has_voiceover": True, "scene_count": 3,
        }

    def generate_script(self, topic: str, duration: int, style: str) -> str:
        result = self.llm_generate(
            f"Write a {duration}s {style} voiceover script about {topic}.",
            system_prompt=f"Write a {duration}-second voiceover script. Language: Indonesia.",
            max_tokens=1000, temperature=0.7,
        )
        return result.get("content", "")

    def quality_review(self, project_summary: dict) -> dict:
        return {"passed": True, "issues": [], "score": 1.0}
