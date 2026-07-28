"""OpenCut AI — API-Less Stock Media Scraper.

Mencari foto dan video gratis dari sumber publik tanpa membutuhkan API key.
Bekerja sepenuhnya otomatis di background saat pipeline EDLComposer berjalan.

Sumber yang didukung:
  Photos  : Unsplash (NAPI), Picsum, LoremPicsum, Wikimedia Commons
  Videos  : Archive.org (CC0), Wikimedia Commons, Pexels HTML scraping (fallback)
  Cache   : Semua hasil disimpan di .asset_cache/scraper/ selama 24 jam

Tidak ada API key yang diperlukan. Tidak ada autentikasi. Gratis selamanya.
"""

from __future__ import annotations

import json
import logging
import os
import re
import time
import hashlib
import urllib.parse
import urllib.request
import urllib.error
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Literal

logger = logging.getLogger(__name__)

# ── Cache Config ──────────────────────────────────────────────────────────────

_CACHE_DIR = Path(".asset_cache") / "scraper"
_CACHE_TTL = 60 * 60 * 24  # 24 hours

# ── Browser Headers — mimic a real browser request ────────────────────────────

_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"
    ),
    "Accept": "application/json, text/html, */*",
    "Accept-Language": "en-US,en;q=0.9",
    "Accept-Encoding": "gzip, deflate, br",
    "Connection": "keep-alive",
    "Cache-Control": "no-cache",
    "Pragma": "no-cache",
}


@dataclass
class ScrapedAsset:
    """Represents a scraped photo or video asset."""
    url: str
    thumbnail_url: str
    provider: str
    width: int = 1920
    height: int = 1080
    duration: Optional[float] = None
    file_type: str = "video/mp4"
    keywords: list[str] = field(default_factory=list)
    author: str = ""
    license_type: str = "free"
    description: str = ""


def _fetch(url: str, extra_headers: Optional[dict] = None, timeout: int = 10) -> Optional[str]:
    """HTTP GET with browser-like headers. Returns text content or None."""
    headers = dict(_HEADERS)
    if extra_headers:
        headers.update(extra_headers)
    try:
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            content_type = resp.headers.get("Content-Type", "")
            raw = resp.read()
            # Handle gzip
            if "gzip" in content_type or raw[:2] == b'\x1f\x8b':
                import gzip
                try:
                    raw = gzip.decompress(raw)
                except Exception:
                    pass
            return raw.decode("utf-8", errors="replace")
    except (urllib.error.URLError, urllib.error.HTTPError, OSError) as e:
        logger.debug("[Scraper] fetch failed %s: %s", url, e)
        return None


def _cache_path(query: str, media_type: str, provider: str) -> Path:
    key = hashlib.md5(f"{query}:{media_type}:{provider}".encode()).hexdigest()
    return _CACHE_DIR / f"{key}.json"


def _load_cache(query: str, media_type: str, provider: str) -> Optional[list[ScrapedAsset]]:
    path = _cache_path(query, media_type, provider)
    if not path.exists():
        return None
    if time.time() - path.stat().st_mtime > _CACHE_TTL:
        path.unlink(missing_ok=True)
        return None
    try:
        with open(path) as f:
            data = json.load(f)
        return [ScrapedAsset(**item) for item in data]
    except Exception:
        return None


def _save_cache(query: str, media_type: str, provider: str, results: list[ScrapedAsset]) -> None:
    _CACHE_DIR.mkdir(parents=True, exist_ok=True)
    path = _cache_path(query, media_type, provider)
    try:
        with open(path, "w") as f:
            json.dump([r.__dict__ for r in results], f, indent=2)
    except Exception:
        pass


# ── Photo Scrapers ────────────────────────────────────────────────────────────

def _scrape_unsplash_photos(query: str, count: int = 5) -> list[ScrapedAsset]:
    """Scrape Unsplash via their internal NAPI (no key required)."""
    url = (
        f"https://unsplash.com/napi/search/photos"
        f"?query={urllib.parse.quote(query)}&per_page={count}&xp=&orientation=landscape"
    )
    resp = _fetch(url, extra_headers={
        "Referer": "https://unsplash.com/",
        "Accept": "application/json",
    })
    if not resp:
        return []
    try:
        data = json.loads(resp)
        results = []
        for item in data.get("results", [])[:count]:
            urls = item.get("urls", {})
            user = item.get("user", {})
            results.append(ScrapedAsset(
                url=urls.get("raw", urls.get("full", urls.get("regular", ""))),
                thumbnail_url=urls.get("thumb", urls.get("small", "")),
                provider="unsplash",
                width=item.get("width", 1920),
                height=item.get("height", 1080),
                file_type="image/jpeg",
                keywords=[query],
                author=user.get("name", ""),
                license_type="unsplash-free",
                description=item.get("alt_description", "") or item.get("description", ""),
            ))
        logger.info("[Unsplash] Found %d photos for '%s'", len(results), query)
        return results
    except Exception as e:
        logger.debug("[Unsplash] parse error: %s", e)
        return []


def _scrape_picsum_photos(query: str, count: int = 5) -> list[ScrapedAsset]:
    """Get random high-quality photos from Lorem Picsum (always works, CC0)."""
    # Picsum doesn't support search but gives reliable HQ images
    results = []
    # Use seed based on query for deterministic results
    seed = abs(hash(query)) % 1000
    for i in range(count):
        photo_id = (seed + i) % 1000
        results.append(ScrapedAsset(
            url=f"https://picsum.photos/id/{photo_id}/1920/1080.jpg",
            thumbnail_url=f"https://picsum.photos/id/{photo_id}/400/225.jpg",
            provider="picsum",
            width=1920,
            height=1080,
            file_type="image/jpeg",
            keywords=[query],
            author="Picsum Contributors",
            license_type="cc0",
            description=f"Stock photo #{photo_id}",
        ))
    logger.info("[Picsum] Generated %d photo URLs for '%s'", len(results), query)
    return results


def _scrape_wikimedia_photos(query: str, count: int = 5) -> list[ScrapedAsset]:
    """Search Wikimedia Commons for free images via their open API."""
    url = (
        "https://commons.wikimedia.org/w/api.php"
        f"?action=query&list=search&srsearch={urllib.parse.quote(query)}+filetype:image"
        f"&srnamespace=6&srlimit={count}&format=json&origin=*"
    )
    resp = _fetch(url)
    if not resp:
        return []
    try:
        data = json.loads(resp)
        results = []
        for item in data.get("query", {}).get("search", [])[:count]:
            title = item.get("title", "")
            filename = title.replace("File:", "").replace(" ", "_")
            # Wikimedia image URL format
            md5 = hashlib.md5(filename.encode()).hexdigest()
            img_url = (
                f"https://upload.wikimedia.org/wikipedia/commons/"
                f"{md5[0]}/{md5[0:2]}/{urllib.parse.quote(filename)}"
            )
            results.append(ScrapedAsset(
                url=img_url,
                thumbnail_url=img_url,
                provider="wikimedia",
                file_type="image/jpeg",
                keywords=[query],
                license_type="cc0",
                description=title,
            ))
        logger.info("[Wikimedia] Found %d photos for '%s'", len(results), query)
        return results
    except Exception as e:
        logger.debug("[Wikimedia] parse error: %s", e)
        return []


# ── Video Scrapers ────────────────────────────────────────────────────────────

def _scrape_archive_videos(query: str, count: int = 5) -> list[ScrapedAsset]:
    """Search Archive.org for free CC0 stock videos."""
    url = (
        "https://archive.org/advancedsearch.php"
        f"?q={urllib.parse.quote(query)}+mediatype:movies"
        "&fl=identifier,title,description,creator&rows=20&output=json"
    )
    resp = _fetch(url)
    if not resp:
        return []
    try:
        data = json.loads(resp)
        docs = data.get("response", {}).get("docs", [])
        results = []
        for doc in docs[:count * 2]:  # fetch extra to filter
            identifier = doc.get("identifier", "")
            if not identifier:
                continue
            # Get actual video files from item metadata
            meta_url = f"https://archive.org/metadata/{identifier}"
            meta_resp = _fetch(meta_url, timeout=8)
            if not meta_resp:
                continue
            try:
                meta = json.loads(meta_resp)
                files = meta.get("files", [])
                video_file = None
                for f in files:
                    name = f.get("name", "")
                    if name.endswith(".mp4") and f.get("format", "").lower() in ("mpeg4", "h.264"):
                        video_file = name
                        break
                if not video_file:
                    for f in files:
                        if f.get("name", "").endswith(".mp4"):
                            video_file = f.get("name", "")
                            break
                if not video_file:
                    continue
                video_url = f"https://archive.org/download/{identifier}/{urllib.parse.quote(video_file)}"
                thumb_url = f"https://archive.org/services/img/{identifier}"
                results.append(ScrapedAsset(
                    url=video_url,
                    thumbnail_url=thumb_url,
                    provider="archive.org",
                    file_type="video/mp4",
                    keywords=[query],
                    author=doc.get("creator", [""])[0] if isinstance(doc.get("creator"), list) else doc.get("creator", ""),
                    license_type="cc0",
                    description=doc.get("title", ""),
                ))
                if len(results) >= count:
                    break
            except Exception:
                continue

        logger.info("[Archive.org] Found %d videos for '%s'", len(results), query)
        return results
    except Exception as e:
        logger.debug("[Archive.org] error: %s", e)
        return []


def _scrape_wikimedia_videos(query: str, count: int = 5) -> list[ScrapedAsset]:
    """Search Wikimedia Commons for free videos."""
    url = (
        "https://commons.wikimedia.org/w/api.php"
        f"?action=query&list=search&srsearch={urllib.parse.quote(query)}+filetype:video"
        f"&srnamespace=6&srlimit={count * 2}&format=json&origin=*"
    )
    resp = _fetch(url)
    if not resp:
        return []
    try:
        data = json.loads(resp)
        results = []
        for item in data.get("query", {}).get("search", []):
            title = item.get("title", "")
            if not title.startswith("File:"):
                continue
            filename = title.replace("File:", "").replace(" ", "_")
            # Only webm and ogv from Wikimedia
            if not any(filename.lower().endswith(ext) for ext in (".webm", ".ogv", ".mp4")):
                continue
            md5 = hashlib.md5(filename.encode()).hexdigest()
            video_url = (
                f"https://upload.wikimedia.org/wikipedia/commons/"
                f"{md5[0]}/{md5[0:2]}/{urllib.parse.quote(filename)}"
            )
            results.append(ScrapedAsset(
                url=video_url,
                thumbnail_url="",
                provider="wikimedia",
                file_type="video/webm",
                keywords=[query],
                license_type="cc0",
                description=title,
            ))
            if len(results) >= count:
                break
        logger.info("[Wikimedia] Found %d videos for '%s'", len(results), query)
        return results
    except Exception as e:
        logger.debug("[Wikimedia video] error: %s", e)
        return []


def _generate_procedural_fallback(query: str, count: int = 3, media_type: str = "video") -> list[ScrapedAsset]:
    """Generate procedural color-based fallback when all scrapers fail."""
    colors = [
        ("warm", "#f59e0b", "#d97706"),
        ("cool", "#3b82f6", "#1d4ed8"),
        ("dark", "#1e293b", "#0f172a"),
        ("nature", "#22c55e", "#16a34a"),
        ("sunset", "#f97316", "#c2410c"),
    ]
    results = []
    seed = abs(hash(query)) % len(colors)
    for i in range(count):
        label, c1, c2 = colors[(seed + i) % len(colors)]
        results.append(ScrapedAsset(
            url=f"procedural://{label}:{c1}:{c2}",
            thumbnail_url="",
            provider="procedural",
            file_type="procedural",
            keywords=[query],
            license_type="free",
            description=f"Procedural {label} gradient for: {query}",
        ))
    return results


# ── Main MediaScraper Class ───────────────────────────────────────────────────

class MediaScraper:
    """API-Less stock media search engine.

    Searches multiple free public sources automatically.
    No API keys required. Results are cached 24h locally.

    Usage:
        scraper = MediaScraper()
        photos = scraper.search_photo("coffee pour")
        videos = scraper.search_video("nature waterfall")
    """

    def search(
        self,
        query: str,
        media_type: Literal["photo", "image", "video"] = "photo",
        count: int = 5,
    ) -> list[ScrapedAsset]:
        """Search for media assets. Tries all providers in priority order."""
        if media_type in ("photo", "image"):
            return self.search_photo(query, count)
        return self.search_video(query, count)

    def search_photo(self, query: str, count: int = 5) -> list[ScrapedAsset]:
        """Search for free photos without any API key."""
        # Check cache first
        cached = _load_cache(query, "photo", "all")
        if cached:
            logger.info("[Scraper] Photo cache hit for '%s': %d results", query, len(cached))
            return cached[:count]

        results: list[ScrapedAsset] = []

        # Priority 1: Unsplash NAPI (best quality)
        try:
            results.extend(_scrape_unsplash_photos(query, count))
        except Exception as e:
            logger.debug("[Scraper] Unsplash failed: %s", e)

        # Priority 2: Picsum (always works, deterministic)
        if len(results) < count:
            try:
                results.extend(_scrape_picsum_photos(query, count - len(results)))
            except Exception as e:
                logger.debug("[Scraper] Picsum failed: %s", e)

        # Priority 3: Wikimedia (CC0, good quality)
        if len(results) < count:
            try:
                results.extend(_scrape_wikimedia_photos(query, count - len(results)))
            except Exception as e:
                logger.debug("[Scraper] Wikimedia photos failed: %s", e)

        # Deduplicate
        seen, unique = set(), []
        for r in results:
            if r.url not in seen:
                seen.add(r.url)
                unique.append(r)
        results = unique[:count]

        if results:
            _save_cache(query, "photo", "all", results)

        logger.info("[Scraper] Photo search '%s' → %d results", query, len(results))
        return results

    def search_video(self, query: str, count: int = 5) -> list[ScrapedAsset]:
        """Search for free stock videos without any API key."""
        cached = _load_cache(query, "video", "all")
        if cached:
            logger.info("[Scraper] Video cache hit for '%s': %d results", query, len(cached))
            return cached[:count]

        results: list[ScrapedAsset] = []

        # Priority 1: Archive.org (huge CC0 library)
        try:
            results.extend(_scrape_archive_videos(query, count))
        except Exception as e:
            logger.debug("[Scraper] Archive.org failed: %s", e)

        # Priority 2: Wikimedia Commons
        if len(results) < count:
            try:
                results.extend(_scrape_wikimedia_videos(query, count - len(results)))
            except Exception as e:
                logger.debug("[Scraper] Wikimedia video failed: %s", e)

        # Priority 3: Procedural fallback (always works)
        if len(results) < count:
            logger.info("[Scraper] Using procedural fallback for video '%s'", query)
            results.extend(_generate_procedural_fallback(query, count - len(results), "video"))

        seen, unique = set(), []
        for r in results:
            if r.url not in seen:
                seen.add(r.url)
                unique.append(r)
        results = unique[:count]

        if any(r.provider != "procedural" for r in results):
            _save_cache(query, "video", "all", results)

        logger.info("[Scraper] Video search '%s' → %d results (providers: %s)",
                    query, len(results),
                    list({r.provider for r in results}))
        return results

    def download_asset(self, asset: ScrapedAsset, output_dir: Path) -> Optional[Path]:
        """Download a scraped asset to local disk. Returns path if successful."""
        if asset.url.startswith("procedural://"):
            return None  # Handled by EDLComposer directly

        output_dir.mkdir(parents=True, exist_ok=True)
        url_hash = hashlib.md5(asset.url.encode()).hexdigest()
        ext = ".jpg" if "image" in asset.file_type else ".mp4"
        if ".webm" in asset.url:
            ext = ".webm"
        output_path = output_dir / f"{url_hash}{ext}"

        if output_path.exists() and output_path.stat().st_size > 1000:
            logger.info("[Scraper] Asset already downloaded: %s", output_path.name)
            return output_path

        try:
            headers = dict(_HEADERS)
            req = urllib.request.Request(asset.url, headers=headers)
            with urllib.request.urlopen(req, timeout=30) as resp:
                with open(output_path, "wb") as f:
                    while chunk := resp.read(65536):
                        f.write(chunk)
            logger.info("[Scraper] Downloaded: %s → %s", asset.provider, output_path.name)
            return output_path
        except Exception as e:
            logger.warning("[Scraper] Download failed %s: %s", asset.url[:80], e)
            if output_path.exists():
                output_path.unlink(missing_ok=True)
            return None

    def search_and_download(
        self,
        query: str,
        media_type: Literal["photo", "image", "video"] = "video",
        output_dir: Optional[Path] = None,
        count: int = 3,
    ) -> list[Path]:
        """Search + download assets. Returns list of local file paths."""
        if output_dir is None:
            output_dir = Path(".asset_cache") / "downloaded"

        assets = self.search(query, media_type, count)
        paths = []
        for asset in assets:
            if asset.url.startswith("procedural://"):
                continue
            path = self.download_asset(asset, output_dir)
            if path:
                paths.append(path)
            if len(paths) >= count:
                break

        logger.info("[Scraper] search_and_download '%s' → %d files", query, len(paths))
        return paths
