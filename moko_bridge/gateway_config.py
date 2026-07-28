"""Centralized gateway configuration for OpenCut AI.

Free API Gateways (priority order):
  1. opencode   — localhost:4096  (is_mandor=True, highest priority)
  2. openrouter — openrouter.ai   (cloud free-tier models)
  3. omnirouter — localhost:20128 (local gateway)
  4. ninerouter — localhost:20130 (local gateway)
  5. local      — GGUF fallback

All providers expose an OpenAI-compatible /v1/chat/completions endpoint.
"""

from __future__ import annotations
import os
from dataclasses import dataclass, field


@dataclass
class GatewayConfig:
    """Configuration for a single AI gateway provider."""
    name: str
    provider: str
    api_base: str
    api_keys: list[str]
    model_name: str = "auto"
    timeout: int = 45
    is_mandor: bool = False
    enabled: bool = True
    # Free models for this gateway (auto-discovered or hardcoded)
    free_models: list[str] = field(default_factory=list)


OPENROUTER_FREE_MODELS = [
    "openrouter/free",
    "google/gemma-4-26b-a4b-it:free",
    "google/gemma-3-12b-it:free",
    "qwen/qwen-2.5-7b-instruct:free",
]

# Best model for JSON/structured output from OpenRouter free tier
OPENROUTER_BEST_JSON_MODEL = "openrouter/free"



def build_gateway_configs() -> list[GatewayConfig]:
    """Build list of all configured gateways from env + defaults."""
    configs: list[GatewayConfig] = []

    # ── 1. OpenCode (local, highest priority, is_mandor) ─────────────────────
    opencode_base = (
        os.environ.get("MOKO_OPENCODE_BASE")
        or os.environ.get("OPENCODE_API_BASE")
        or "http://localhost:4096/v1"
    )
    opencode_key_env = os.environ.get("MOKO_OPENCODE_KEYS") or os.environ.get("OPENCODE_API_KEY") or ""
    opencode_keys = [k.strip() for k in opencode_key_env.split(",") if k.strip()] or ["opencode-free"]
    configs.append(GatewayConfig(
        name="opencode-free",
        provider="opencode",
        api_base=opencode_base,
        api_keys=opencode_keys,
        model_name="auto",
        timeout=30,
        is_mandor=True,
        enabled=True,
    ))

    # ── 2. OpenRouter (cloud, free tier) ─────────────────────────────────────
    openrouter_key = (
        os.environ.get("OPENROUTER_API_KEY")
        or os.environ.get("MOKO_OPENROUTER_KEY")
        or ""
    )
    openrouter_enabled = bool(openrouter_key)  # enabled only if key present
    configs.append(GatewayConfig(
        name="openrouter-free",
        provider="openrouter",
        api_base="https://openrouter.ai/api/v1",
        api_keys=[openrouter_key] if openrouter_key else [""],
        model_name=OPENROUTER_BEST_JSON_MODEL,
        timeout=60,
        is_mandor=False,
        enabled=openrouter_enabled,
        free_models=OPENROUTER_FREE_MODELS,
    ))

    # ── 3. OmniRouter (local) ────────────────────────────────────────────────
    omni_base = (
        os.environ.get("MOKO_OMNIROUTE_BASE")
        or os.environ.get("OMNIROUTE_API_BASE")
        or "http://localhost:20128/v1"
    )
    omni_key_env = os.environ.get("MOKO_OMNIROUTE_KEYS") or os.environ.get("OMNIROUTE_API_KEY") or ""
    omni_keys = [k.strip() for k in omni_key_env.split(",") if k.strip()] or ["omni-free"]
    configs.append(GatewayConfig(
        name="omnirouter-free",
        provider="omnirouter",
        api_base=omni_base,
        api_keys=omni_keys,
        model_name="auto",
        timeout=30,
        is_mandor=False,
        enabled=True,
    ))

    # ── 4. 9Router (local) ───────────────────────────────────────────────────
    nine_base = (
        os.environ.get("MOKO_NINEROUTE_BASE")
        or os.environ.get("NINEROUTE_API_BASE")
        or "http://localhost:20130/v1"
    )
    nine_key_env = os.environ.get("MOKO_NINEROUTE_KEYS") or os.environ.get("NINEROUTE_API_KEY") or ""
    nine_keys = [k.strip() for k in nine_key_env.split(",") if k.strip()] or ["nine-free"]
    configs.append(GatewayConfig(
        name="ninerouter-free",
        provider="ninerouter",
        api_base=nine_base,
        api_keys=nine_keys,
        model_name="auto",
        timeout=30,
        is_mandor=False,
        enabled=True,
    ))

    return configs


def get_priority_ordered_configs(configs: list[GatewayConfig]) -> list[GatewayConfig]:
    """Return only enabled configs sorted by priority."""
    PRIORITY = {"opencode": 0, "openrouter": 1, "omnirouter": 2, "ninerouter": 3, "local": 99}
    enabled = [c for c in configs if c.enabled]
    return sorted(enabled, key=lambda c: PRIORITY.get(c.provider, 50))


# Singleton config list — built once, reused everywhere
_GATEWAY_CONFIGS: list[GatewayConfig] | None = None


def get_gateway_configs() -> list[GatewayConfig]:
    """Get (or build) the global gateway config list."""
    global _GATEWAY_CONFIGS
    if _GATEWAY_CONFIGS is None:
        _GATEWAY_CONFIGS = build_gateway_configs()
    return _GATEWAY_CONFIGS


def reset_gateway_configs() -> None:
    """Force re-build of configs (useful after env changes)."""
    global _GATEWAY_CONFIGS
    _GATEWAY_CONFIGS = None
