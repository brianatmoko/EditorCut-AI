"""Tests for MOKO bridge client."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from moko_bridge.moko_client import MOKOClient, MOKOConfig


def test_health_check():
    client = MOKOClient()
    health = client.check_health()
    assert "llm" in health
    assert "rag" in health
    assert "native" in health
    assert "version" in health


def test_analyze_brief():
    client = MOKOClient()
    result = client.analyze_brief("buat video cinematic 30 detik")
    assert "intent" in result
    assert result["intent"] == "auto_edit"
    assert "duration" in result


def test_llm_fallback():
    client = MOKOClient(MOKOConfig(llm_host="0.0.0.0", llm_port=1))
    result = client.llm_generate("test", max_tokens=10)
    assert result is not None
    assert "content" in result
