"""Search local asset library using keyword matching.

Pure keyword-based search — no embedding costs, 0 tokens.
Falls back to filesystem glob if no index exists.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import os
import json
import re


@dataclass
class LocalAsset:
    path: str
    filename: str
    file_type: str
    size_bytes: int
    duration: Optional[float] = None
    keywords: list[str] = field(default_factory=list)
    thumbnail_path: Optional[str] = None


class RAGSearch:
    def __init__(self, library_dirs: Optional[list[str]] = None):
        self.library_dirs = [Path(d) for d in (library_dirs or ["./assets/"])]
        self._index: dict[str, list[LocalAsset]] = {}
        self._all_assets: list[LocalAsset] = []

    def search(self, query: str, max_results: int = 10) -> list[LocalAsset]:
        self._ensure_indexed()

        query_keywords = set(re.findall(r'\w+', query.lower()))
        if not query_keywords:
            return self._all_assets[:max_results]

        scored = []
        for asset in self._all_assets:
            score = self._score(asset, query_keywords)
            if score > 0:
                scored.append((score, asset))

        scored.sort(key=lambda x: -x[0])
        return [asset for _, asset in scored[:max_results]]

    def _ensure_indexed(self) -> None:
        if self._all_assets:
            return

        video_exts = {".mp4", ".mov", ".avi", ".mkv", ".webm"}
        audio_exts = {".mp3", ".wav", ".flac", ".aac", ".ogg"}
        image_exts = {".jpg", ".jpeg", ".png", ".webp", ".gif"}

        for lib_dir in self.library_dirs:
            if not lib_dir.exists():
                continue

            for file_path in lib_dir.rglob("*"):
                if not file_path.is_file():
                    continue

                ext = file_path.suffix.lower()
                if ext not in video_exts | audio_exts | image_exts:
                    continue

                if ext in video_exts:
                    ftype = "video"
                elif ext in audio_exts:
                    ftype = "audio"
                else:
                    ftype = "image"

                stem = re.sub(r'[_-]', ' ', file_path.stem)
                keywords = list(set(re.findall(r'\w+', stem.lower())))

                asset = LocalAsset(
                    path=str(file_path),
                    filename=file_path.name,
                    file_type=ftype,
                    size_bytes=file_path.stat().st_size,
                    keywords=keywords,
                )

                self._all_assets.append(asset)

                for kw in keywords:
                    if kw not in self._index:
                        self._index[kw] = []
                    self._index[kw].append(asset)

    def _score(self, asset: LocalAsset, query_keywords: set[str]) -> float:
        asset_keywords = set(asset.keywords)
        overlap = query_keywords & asset_keywords

        if not overlap:
            return 0.0

        score = len(overlap) / max(len(query_keywords), 1)

        stem = re.sub(r'[_-]', ' ', Path(asset.path).stem).lower()
        full_query = " ".join(query_keywords)
        if full_query in stem:
            score *= 2.0

        return score

    def scan_directory(self, directory: str) -> list[LocalAsset]:
        assets = []
        video_exts = {".mp4", ".mov", ".avi"}
        audio_exts = {".mp3", ".wav", ".flac"}

        dir_path = Path(directory)
        if not dir_path.exists():
            return assets

        for f in dir_path.iterdir():
            if not f.is_file():
                continue
            ext = f.suffix.lower()
            if ext in video_exts:
                assets.append(LocalAsset(
                    path=str(f), filename=f.name, file_type="video",
                    size_bytes=f.stat().st_size,
                    keywords=re.findall(r'\w+', f.stem.lower())
                ))
            elif ext in audio_exts:
                assets.append(LocalAsset(
                    path=str(f), filename=f.name, file_type="audio",
                    size_bytes=f.stat().st_size,
                    keywords=re.findall(r'\w+', f.stem.lower())
                ))

        return assets

    def estimate_tokens(self) -> int:
        return 0
