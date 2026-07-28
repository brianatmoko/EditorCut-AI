"""OpenCut AI — Unified Settings & Configuration.

Single source of truth for all OpenCut AI settings.
Stored in `opencut_config.json` at the project root.

Manages:
  - API gateway credentials (OpenRouter, Pexels, Pixabay)
  - Gateway URLs and enable/disable toggles
  - Rendering preferences (output dir, codec, resolution)
  - Language & platform defaults
"""

from __future__ import annotations

import json
import logging
import os
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

# Config file location — stored in OpenCut project root
_CONFIG_PATH = Path(__file__).parent.parent.parent / "opencut_config.json"

# ── Default Configuration ─────────────────────────────────────────────────────

_DEFAULTS = {
    # API Keys — set these to enable cloud services
    "openrouter_api_key": "",
    "pexels_api_key": "",
    "pixabay_api_key": "",

    # Gateway URLs — local MOKO OS services
    "gateways": {
        "opencode": {
            "enabled": True,
            "url": "http://localhost:4096/v1",
            "timeout": 5,
        },
        "openrouter": {
            "enabled": True,
            "url": "https://openrouter.ai/api/v1",
            "timeout": 30,
            "model": "meta-llama/llama-3.1-8b-instruct",
        },
        "omnirouter": {
            "enabled": True,
            "url": "http://localhost:20128/v1",
            "timeout": 5,
        },
        "ninerouter": {
            "enabled": True,
            "url": "http://localhost:20130/v1",
            "timeout": 5,
        },
    },

    # Rendering defaults
    "render": {
        "output_dir": "./output",
        "codec": "h264",
        "resolution": "1080p",
        "fps": 30,
    },

    # AI Director defaults
    "director": {
        "default_duration": 30,
        "default_style": "cinematic",
        "default_platform": "youtube",
        "default_language": "id",
        "voiceover_enabled": True,
        "voiceover_engine": "edge-tts",  # edge-tts | gtts | none
    },

    # Performance — skip slow checks
    "performance": {
        "gateway_connect_timeout": 2.0,  # seconds for fast check
        "skip_offline_gateways": True,
        "max_retry_per_gateway": 1,
    },
}


def _deep_merge(base: dict, override: dict) -> dict:
    """Deep merge override into base."""
    result = base.copy()
    for key, val in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(val, dict):
            result[key] = _deep_merge(result[key], val)
        else:
            result[key] = val
    return result


class OpenCutConfig:
    """Unified configuration manager for OpenCut AI."""

    _instance: Optional["OpenCutConfig"] = None

    def __init__(self, config_path: Optional[Path] = None):
        self._path = config_path or _CONFIG_PATH
        self._data: dict = {}
        self._load()

    @classmethod
    def get(cls) -> "OpenCutConfig":
        """Get singleton config instance."""
        if cls._instance is None:
            cls._instance = OpenCutConfig()
        return cls._instance

    @classmethod
    def reset(cls) -> None:
        """Reset singleton (reload from disk)."""
        cls._instance = None

    def _load(self) -> None:
        """Load config from disk, merging with defaults."""
        self._data = _DEFAULTS.copy()
        if self._path.exists():
            try:
                with open(self._path) as f:
                    saved = json.load(f)
                self._data = _deep_merge(_DEFAULTS, saved)
                logger.info("[Config] Loaded from %s", self._path)
            except (json.JSONDecodeError, IOError) as e:
                logger.warning("[Config] Failed to load %s: %s, using defaults", self._path, e)
        else:
            # Create config file with defaults on first run
            self.save()
            logger.info("[Config] Created default config at %s", self._path)

    def save(self) -> None:
        """Save current config to disk."""
        try:
            with open(self._path, "w") as f:
                json.dump(self._data, f, indent=2, ensure_ascii=False)
            logger.info("[Config] Saved to %s", self._path)
        except IOError as e:
            logger.error("[Config] Failed to save: %s", e)

    # ── Getters ───────────────────────────────────────────────────────────────

    @property
    def openrouter_api_key(self) -> str:
        """Get OpenRouter API key from config or environment."""
        return (
            os.environ.get("OPENROUTER_API_KEY")
            or self._data.get("openrouter_api_key", "")
        )

    @property
    def nvidia_api_key(self) -> str:
        """Get NVIDIA API key from config or environment."""
        return (
            os.environ.get("NVIDIA_API_KEY")
            or self._data.get("nvidia_api_key", "")
        )

    @property
    def nvidia_api_key_alt(self) -> str:
        """Get fallback NVIDIA API key from config or environment."""
        return (
            os.environ.get("NVIDIA_API_KEY_ALT")
            or self._data.get("nvidia_api_key_alt", "")
        )

    @property
    def openrouter_api_key_alt(self) -> str:
        """Get fallback OpenRouter API key from config or environment."""
        return (
            os.environ.get("OPENROUTER_API_KEY_ALT")
            or self._data.get("openrouter_api_key_alt", "")
        )

    @property
    def pexels_api_key(self) -> str:
        return (
            os.environ.get("PEXELS_API_KEY")
            or self._data.get("pexels_api_key", "")
        )

    @property
    def pixabay_api_key(self) -> str:
        return (
            os.environ.get("PIXABAY_API_KEY")
            or self._data.get("pixabay_api_key", "")
        )

    @property
    def gemini_api_key(self) -> str:
        return (
            os.environ.get("GEMINI_API_KEY")
            or os.environ.get("GOOGLE_API_KEY")
            or self._data.get("gemini_api_key", "")
        )

    @property
    def gemini_api_key_alt(self) -> str:
        return (
            os.environ.get("GEMINI_API_KEY_ALT")
            or self._data.get("gemini_api_key_alt", "")
        )

    @property
    def gemini_api_keys(self) -> list[str]:
        """All Gemini API keys: primary + alt + extra keys array."""
        keys = []
        pk = self.gemini_api_key
        if pk:
            keys.append(pk)
        alt = self.gemini_api_key_alt
        if alt and alt != pk:
            keys.append(alt)
        for k in self._data.get("gemini_api_keys", []):
            if k and k not in keys:
                keys.append(k)
        return keys

    @property
    def swiftrouter_api_keys(self) -> list[str]:
        """All SwiftRouter API keys (primary + all alts)."""
        keys = []
        for i in range(9):
            env_key = f"SWIFTRouter_API_KEY_ALT{i}" if i > 0 else "SWIFTRouter_API_KEY"
            k = os.environ.get(env_key) or self._data.get(f"swiftrouter_api_key_alt{i}" if i > 0 else "swiftrouter_api_key", "")
            if k:
                keys.append(k)
        return keys

    @property
    def gateways(self) -> dict:
        return self._data.get("gateways", {})

    @property
    def render_config(self) -> dict:
        return self._data.get("render", {})

    @property
    def director_config(self) -> dict:
        return self._data.get("director", {})

    @property
    def performance(self) -> dict:
        return self._data.get("performance", {})

    def get_gateway(self, name: str) -> dict:
        """Get gateway config by name."""
        return self.gateways.get(name, {})

    def is_gateway_enabled(self, name: str) -> bool:
        gw = self.get_gateway(name)
        return gw.get("enabled", False)

    # ── Setters ───────────────────────────────────────────────────────────────

    def set(self, key: str, value) -> None:
        """Set a top-level config value and save."""
        self._data[key] = value
        self.save()

    def set_nested(self, path: str, value) -> None:
        """Set a nested config value. Path uses dots: 'gateways.openrouter.enabled'"""
        keys = path.split(".")
        d = self._data
        for k in keys[:-1]:
            d = d.setdefault(k, {})
        d[keys[-1]] = value
        self.save()

    def update_all(self, data: dict) -> None:
        """Replace entire config and save."""
        self._data = _deep_merge(_DEFAULTS, data)
        self.save()

    def to_dict(self) -> dict:
        """Return full config as dict (safe for JSON serialization)."""
        # Mask API keys for security
        result = json.loads(json.dumps(self._data))
        for key in ("openrouter_api_key", "pexels_api_key", "pixabay_api_key"):
            val = result.get(key, "")
            if val and len(val) > 8:
                result[key] = val[:4] + "..." + val[-4:]
        return result

    def to_dict_full(self) -> dict:
        """Return full config without masking (for internal use)."""
        return dict(self._data)
