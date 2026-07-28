"""Shared type definitions for MOKO OS bridge communication."""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class MOKOInferenceRequest:
    """Request to MOKO OS LLM."""
    prompt: str
    system_prompt: Optional[str] = None
    max_tokens: int = 1000
    temperature: float = 0.7
    stream: bool = False


@dataclass
class MOKOInferenceResponse:
    """Response from MOKO OS LLM."""
    content: str
    tokens_used: int = 0
    confidence: float = 0.0
    model: str = "MOKO-AI-4B"
    error: Optional[str] = None


@dataclass
class MOKORAGRequest:
    """Request to MOKO RAG server."""
    query: str
    top_k: int = 10
    min_score: float = 0.5


@dataclass
class MOKORAGResult:
    """Single result from RAG search."""
    path: str
    score: float
    metadata: dict = field(default_factory=dict)
    snippet: Optional[str] = None


@dataclass
class MOKOHealth:
    """MOKO OS service health status."""
    llm_available: bool = False
    rag_available: bool = False
    native_available: bool = False
    version: str = "unknown"
    uptime: float = 0.0
