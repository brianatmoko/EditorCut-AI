"""Search video/image assets from public APIs (Pexels, Pixabay).

REST API calls — 0 token cost. Results are cached locally.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Literal
import os
import json
import time
import hashlib
from pathlib import Path
from urllib.request import urlopen, Request
from urllib.parse import urlencode
from urllib.error import URLError


@dataclass
class AssetResult:
    url: str
    thumbnail_url: str
    provider: str
    width: int
    height: int
    duration: Optional[float] = None
    file_type: str = "video/mp4"
    keywords: list[str] = field(default_factory=list)
    author: str = ""
    license_type: str = "free"


class AssetCrawler:
    def __init__(self, cache_dir: str = ".asset_cache"):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self._pexels_key = self._resolve_key("pexels_api_key", "PEXELS_API_KEY")
        self._pixabay_key = self._resolve_key("pixabay_api_key", "PIXABAY_API_KEY")

    @staticmethod
    def _resolve_key(config_key: str, env_key: str) -> str:
        key = os.environ.get(env_key, "")
        if not key:
            try:
                from auto_editor.config.opencut_settings import OpenCutConfig
                cfg = OpenCutConfig.get()
                key = getattr(cfg, config_key, "")
            except Exception:
                pass
        return key or ""

    def search(
        self,
        query: str,
        media_type: Literal["video", "image"] = "video",
        max_results: int = 10,
        min_duration: float = 3.0,
        preferred_provider: Optional[str] = None
    ) -> list[AssetResult]:
        cache_key = self._cache_key(query, media_type)
        cached = self._load_cache(cache_key)
        if cached is not None:
            return cached[:max_results]

        results = []

        # ── Try official APIs first (if keys are configured) ──────────────────
        if preferred_provider in (None, "pexels") and self._pexels_key:
            try:
                pexels_results = self._search_pexels(query, media_type, max_results)
                results.extend(pexels_results)
            except (URLError, json.JSONDecodeError):
                pass

        if preferred_provider in (None, "pixabay") and self._pixabay_key:
            try:
                pixabay_results = self._search_pixabay(query, media_type, max_results)
                results.extend(pixabay_results)
            except (URLError, json.JSONDecodeError):
                pass

        # ── API-Less fallback: use MediaScraper when no keys configured ───────
        if not results:
            try:
                from auto_editor.workers.asset_finder.media_scraper import (
                    MediaScraper, ScrapedAsset
                )
                scraper = MediaScraper()
                scraper_type = "photo" if media_type == "image" else media_type
                scraped = scraper.search(query, scraper_type, max_results)

                for item in scraped:
                    results.append(AssetResult(
                        url=item.url,
                        thumbnail_url=item.thumbnail_url,
                        provider=item.provider,
                        width=item.width,
                        height=item.height,
                        duration=item.duration,
                        file_type=item.file_type,
                        keywords=item.keywords,
                        author=item.author,
                        license_type=item.license_type,
                    ))
            except Exception as e:
                import logging
                logging.getLogger(__name__).warning("[Crawler] MediaScraper failed: %s", e)

        if media_type == "video":
            results = [
                r for r in results
                if r.duration is None or r.duration >= min_duration
            ]

        seen_urls = set()
        unique = []
        for r in results:
            if r.url not in seen_urls:
                seen_urls.add(r.url)
                unique.append(r)

        self._save_cache(cache_key, unique)
        return unique[:max_results]


    def _search_pexels(self, query: str, media_type: str, per_page: int) -> list[AssetResult]:
        endpoint = "videos/search" if media_type == "video" else "photos/search"
        url = f"https://api.pexels.com/{endpoint}?{urlencode({'query': query, 'per_page': per_page})}"

        req = Request(url, headers={"Authorization": self._pexels_key})
        with urlopen(req, timeout=15) as resp:
            data = json.loads(resp.read().decode())

        results = []
        items = data.get("videos", data.get("photos", []))
        for item in items:
            if media_type == "video":
                video_files = item.get("video_files", [])
                best = self._best_quality(video_files)
                if best:
                    results.append(AssetResult(
                        url=best["link"],
                        thumbnail_url=item.get("image", ""),
                        provider="pexels",
                        width=best.get("width", 1920),
                        height=best.get("height", 1080),
                        duration=float(item.get("duration", 0)),
                        file_type=best.get("file_type", "video/mp4"),
                        keywords=[query],
                        author=item.get("user", {}).get("name", ""),
                    ))
            else:
                src = item.get("src", {})
                results.append(AssetResult(
                    url=src.get("original", ""),
                    thumbnail_url=src.get("medium", ""),
                    provider="pexels",
                    width=item.get("width", 1920),
                    height=item.get("height", 1080),
                    file_type="image/jpeg",
                    keywords=[query],
                    author=item.get("photographer", ""),
                ))

        return results

    def _search_pixabay(self, query: str, media_type: str, per_page: int) -> list[AssetResult]:
        endpoint = "videos" if media_type == "video" else "photos"
        url = (
            f"https://pixabay.com/api/{endpoint}/"
            f"?{urlencode({'key': self._pixabay_key, 'q': query, 'per_page': per_page})}"
        )

        with urlopen(url, timeout=15) as resp:
            data = json.loads(resp.read().decode())

        results = []
        hits = data.get("hits", [])
        for hit in hits:
            if media_type == "video":
                videos = hit.get("videos", {})
                best = videos.get("large", videos.get("medium", videos.get("small", {})))
                if best:
                    results.append(AssetResult(
                        url=best.get("url", ""),
                        thumbnail_url=hit.get("pageURL", ""),
                        provider="pixabay",
                        width=int(hit.get("width", 1920)),
                        height=int(hit.get("height", 1080)),
                        duration=float(hit.get("duration", 0)),
                        file_type="video/mp4",
                        keywords=[query],
                        author=hit.get("user", ""),
                    ))
            else:
                results.append(AssetResult(
                    url=hit.get("largeImageURL", ""),
                    thumbnail_url=hit.get("previewURL", ""),
                    provider="pixabay",
                    width=hit.get("imageWidth", 1920),
                    height=hit.get("imageHeight", 1080),
                    file_type="image/jpeg",
                    keywords=[query],
                    author=hit.get("user", ""),
                ))

        return results

    def _best_quality(self, video_files: list[dict]) -> Optional[dict]:
        if not video_files:
            return None

        sorted_files = sorted(
            video_files,
            key=lambda f: (
                f.get("width", 0) * f.get("height", 0),
                f.get("quality", "sd") == "hd",
            ),
            reverse=True
        )
        return sorted_files[0]

    def _cache_key(self, query: str, media_type: str) -> str:
        raw = f"{query}:{media_type}"
        return hashlib.md5(raw.encode()).hexdigest()

    def _load_cache(self, key: str) -> Optional[list[AssetResult]]:
        cache_file = self.cache_dir / f"{key}.json"
        if not cache_file.exists():
            return None

        if time.time() - cache_file.stat().st_mtime > 3600:
            cache_file.unlink(missing_ok=True)
            return None

        try:
            with open(cache_file) as f:
                data = json.load(f)
            return [AssetResult(**item) for item in data]
        except (json.JSONDecodeError, TypeError):
            return None

    def _save_cache(self, key: str, results: list[AssetResult]) -> None:
        cache_file = self.cache_dir / f"{key}.json"
        try:
            with open(cache_file, "w") as f:
                json.dump([r.__dict__ for r in results], f)
        except IOError:
            pass

    def estimate_tokens(self, query: str) -> int:
        return 0
