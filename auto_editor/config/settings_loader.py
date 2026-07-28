"""Configuration loader — YAML file to AutoEditorConfig dataclass."""

from __future__ import annotations
from pathlib import Path
from typing import Optional
import yaml
import os

from ..models import AutoEditorConfig, Mode, LocalConfig, APIConfig, ResourceConfig, BehaviorConfig


DEFAULT_CONFIG_PATH = Path(__file__).parent / "settings.yaml"


def load_config(path: Optional[str] = None) -> AutoEditorConfig:
    config = _default_config()
    config_path = Path(path) if path else DEFAULT_CONFIG_PATH
    if config_path.exists():
        with open(config_path) as f:
            yaml_data = yaml.safe_load(f)
        if yaml_data:
            config = _merge_yaml(config, yaml_data)
    config = _apply_env_overrides(config)
    return config


def _default_config() -> AutoEditorConfig:
    return AutoEditorConfig(
        mode=Mode.HYBRID, local=LocalConfig(), api=APIConfig(),
        resources=ResourceConfig(), behavior=BehaviorConfig(),
    )


def _merge_yaml(config: AutoEditorConfig, yaml_data: dict) -> AutoEditorConfig:
    if "mode" in yaml_data:
        config.mode = Mode(yaml_data["mode"])
    if "local" in yaml_data:
        local = yaml_data["local"]
        if "llm_model" in local: config.local.llm_model = local["llm_model"]
        if "tts_model" in local: config.local.tts_model = local["tts_model"]
        if "asr_model" in local: config.local.asr_model = local["asr_model"]
        if "models_dir" in local: config.local.models_dir = local["models_dir"]
    if "api" in yaml_data:
        api = yaml_data["api"]
        if "llm" in api:
            if "provider" in api["llm"]: config.api.llm_provider = api["llm"]["provider"]
            if "model" in api["llm"]: config.api.llm_model = api["llm"]["model"]
            if "max_tokens" in api["llm"]: config.api.llm_max_tokens = api["llm"]["max_tokens"]
        if "pexels" in api and "api_key" in api["pexels"]:
            config.api.pexels_api_key = api["pexels"]["api_key"]
    if "resources" in yaml_data:
        res = yaml_data["resources"]
        if "max_vram_gb" in res: config.resources.max_vram_gb = res["max_vram_gb"]
        if "max_ram_gb" in res: config.resources.max_ram_gb = res["max_ram_gb"]
        if "max_threads" in res: config.resources.max_threads = res["max_threads"]
    if "behavior" in yaml_data:
        beh = yaml_data["behavior"]
        if "confidence_threshold" in beh: config.behavior.confidence_threshold = beh["confidence_threshold"]
        if "max_retries" in beh: config.behavior.max_retries = beh["max_retries"]
        if "cache_enabled" in beh: config.behavior.cache_enabled = beh["cache_enabled"]
        if "cache_ttl_minutes" in beh: config.behavior.cache_ttl_minutes = beh["cache_ttl_minutes"]
    return config


def _apply_env_overrides(config: AutoEditorConfig) -> AutoEditorConfig:
    env_map = {
        "OPENCUT_AI_MODE": ("mode", lambda v: Mode(v)),
        "OPENCUT_AI_LLM_MODEL": ("local.llm_model", str),
        "OPENCUT_AI_TTS_MODEL": ("local.tts_model", str),
        "OPENCUT_AI_ASR_MODEL": ("local.asr_model", str),
        "OPENCUT_AI_MODELS_DIR": ("local.models_dir", str),
        "OPENCUT_AI_API_PROVIDER": ("api.llm_provider", str),
        "OPENCUT_AI_API_MODEL": ("api.llm_model", str),
        "OPENCUT_AI_API_MAX_TOKENS": ("api.llm_max_tokens", int),
        "OPENCUT_AI_PEXELS_KEY": ("api.pexels_api_key", str),
        "OPENCUT_AI_MAX_VRAM": ("resources.max_vram_gb", int),
        "OPENCUT_AI_MAX_RAM": ("resources.max_ram_gb", int),
        "OPENCUT_AI_THREADS": ("resources.max_threads", int),
        "OPENCUT_AI_CONFIDENCE": ("behavior.confidence_threshold", float),
        "OPENCUT_AI_CACHE": ("behavior.cache_enabled", lambda v: v.lower() == "true"),
    }
    for env_name, (attr_path, converter) in env_map.items():
        env_val = os.environ.get(env_name)
        if env_val is not None:
            parts = attr_path.split(".")
            obj = config
            for part in parts[:-1]:
                obj = getattr(obj, part)
            try:
                setattr(obj, parts[-1], converter(env_val))
            except (ValueError, TypeError):
                pass
    return config


def save_config(config: AutoEditorConfig, path: str) -> None:
    data = {
        "mode": config.mode.value,
        "local": {"llm_model": config.local.llm_model, "tts_model": config.local.tts_model,
                  "asr_model": config.local.asr_model, "models_dir": config.local.models_dir},
        "api": {"llm": {"provider": config.api.llm_provider, "model": config.api.llm_model,
                        "max_tokens": config.api.llm_max_tokens},
                "pexels": {"api_key": config.api.pexels_api_key if config.api.pexels_api_key else ""}},
        "resources": {"max_vram_gb": config.resources.max_vram_gb, "max_ram_gb": config.resources.max_ram_gb,
                      "max_threads": config.resources.max_threads},
        "behavior": {"confidence_threshold": config.behavior.confidence_threshold,
                     "max_retries": config.behavior.max_retries, "cache_enabled": config.behavior.cache_enabled,
                     "cache_ttl_minutes": config.behavior.cache_ttl_minutes},
    }
    with open(path, "w") as f:
        yaml.dump(data, f, default_flow_style=False, allow_unicode=True)
