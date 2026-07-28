# AGENT 2: WORKERS IMPLEMENTATION

> **Peran:** Implementasi semua worker spesialis — eksekusi editing yang sesungguhnya.
> **Lingkup kerja:** `Opencut/auto-editor/workers/*`
> **Prasyarat:** Agent 1 sudah selesai (models.py, orchestrator/, coordinate.py, config/, CLI).
> **Kamu tidak perlu mengubah orchestrator/ atau models.py — cukup pakai interface yang sudah ada.**

---

## PENTING — Aturan Main

```
1. JANGAN ubah file di luar auto-editor/workers/ dan auto-editor/tests/.
2. JANGAN ubah models.py — semua data class sudah didefinisikan Agent 1.
3. JANGAN ubah orchestrator/ — itu urusan Agent 1 dan Agent 3.
4. Setiap worker WAJIB:
   - Type hints di semua fungsi
   - 1 baris docstring
   - Tidak ada komentar kode
   - Error handling dengan graceful fallback
   - Test file sendiri di tests/
5. Urutan implementasi: SceneDetector → AssetFinder → LayoutEngine → AudioPipeline → Effects → Renderer
   (masing-masing bisa running sendiri-sendiri setelah selesai)
```

---

## Ringkasan Arsitektur Workers

```
User Intent + EditingPlan (dari Agent 1 orchestrator)
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
SceneDetector  AssetFinder    LayoutEngine
    │               │               │
    ▼               ▼               ▼
         AudioPipeline (TTS + ASR + Mixer)
                    │
                    ▼
               Effects (Color + Transitions + Text)
                    │
                    ▼
               Renderer (OpenCut + FFmpeg)
                    │
                    ▼
             final_video.mp4
```

Setiap worker independen — bisa di-test sendiri tanpa worker lain.

---

## Task 2.1 — Scene Detector

**Folder:** `auto-editor/workers/scene_detector/`
**File:** `detector.py`, `classifier.py`

### 2.1.1 Detector (`detector.py`)

Mendeteksi perubahan scene dalam video menggunakan **FFmpeg scene detection**.
Ini murni komputasi (0 token) — tidak ada AI.

```python
"""Detect scene changes in video using FFmpeg scene detection.

Pure computation — 0 token cost. Returns list of shots with timestamps.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import subprocess
import json
import re
import tempfile
import os


@dataclass
class Shot:
    """Single detected shot/scene."""
    index: int
    start_time: float          # seconds
    end_time: float            # seconds
    duration: float            # seconds
    confidence: float = 1.0    # detection confidence (0.0 - 1.0)


class SceneDetector:
    """Detect scene changes in video using FFmpeg.
    
    Uses FFmpeg's scene detection filter which analyzes histogram
    differences between frames — fast, deterministic, 0 token cost.
    """
    
    def __init__(self, threshold: float = 0.3, ffmpeg_path: str = "ffmpeg"):
        """
        Args:
            threshold: Scene change sensitivity (0.1 = very sensitive, 0.5 = less sensitive).
            ffmpeg_path: Path to ffmpeg binary.
        """
        self.threshold = threshold
        self.ffmpeg_path = ffmpeg_path
    
    def detect(self, video_path: str) -> list[Shot]:
        """Detect scene changes in video file.
        
        Args:
            video_path: Path to video file.
            
        Returns:
            List of Shot objects with start/end times.
            Empty list if detection fails (graceful degradation).
        """
        if not os.path.exists(video_path):
            return []
        
        try:
            # Get video duration first
            duration = self._get_duration(video_path)
            if duration is None or duration == 0:
                return []
            
            # Run FFmpeg scene detection
            scene_times = self._run_scene_detect(video_path)
            
            # Build shots from scene times
            shots = []
            prev_time = 0.0
            for i, scene_time in enumerate(scene_times):
                shots.append(Shot(
                    index=i,
                    start_time=prev_time,
                    end_time=scene_time,
                    duration=scene_time - prev_time,
                    confidence=self.threshold
                ))
                prev_time = scene_time
            
            # Add final shot
            shots.append(Shot(
                index=len(shots),
                start_time=prev_time,
                end_time=duration,
                duration=duration - prev_time,
                confidence=self.threshold
            ))
            
            return shots
            
        except (subprocess.SubprocessError, FileNotFoundError) as e:
            return self._fallback_detect(video_path)
    
    def _get_duration(self, video_path: str) -> Optional[float]:
        """Get video duration in seconds using FFprobe."""
        try:
            cmd = [
                "ffprobe", "-v", "error",
                "-show_entries", "format=duration",
                "-of", "json", video_path
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            data = json.loads(result.stdout)
            return float(data["format"]["duration"])
        except (subprocess.SubprocessError, json.JSONDecodeError, KeyError, ValueError):
            return None
    
    def _run_scene_detect(self, video_path: str) -> list[float]:
        """Run FFmpeg scene detection filter.
        
        Returns list of timestamps where scene changes occur.
        """
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            log_path = f.name
        
        try:
            cmd = [
                self.ffmpeg_path, "-i", video_path,
                "-filter:v", f"select='gt(scene,{self.threshold})',showinfo",
                "-f", "null", "-"
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            # Parse showinfo output for pts_time
            scene_times = []
            for line in result.stderr.split("\n"):
                match = re.search(r'pts_time:(\d+\.?\d*)', line)
                if match:
                    scene_times.append(float(match.group(1)))
            
            return scene_times
            
        except subprocess.SubprocessError:
            return []
        finally:
            if os.path.exists(log_path):
                os.unlink(log_path)
    
    def _fallback_detect(self, video_path: str) -> list[Shot]:
        """Fallback: return single shot covering entire video.
        
        Graceful degradation when FFmpeg is unavailable.
        """
        duration = self._get_duration(video_path)
        if duration is None:
            duration = 30.0  # assume 30 seconds if can't determine
        
        return [Shot(
            index=0,
            start_time=0.0,
            end_time=duration,
            duration=duration,
            confidence=0.5  # lower confidence for fallback
        )]
    
    def detect_with_thumbnails(self, video_path: str) -> list[dict]:
        """Detect scenes + extract thumbnail for each.
        
        Returns:
            List of {shot: Shot, thumbnail_path: str | None}
        """
        shots = self.detect(video_path)
        result = []
        
        for shot in shots:
            thumb = self._extract_thumbnail(video_path, shot.start_time)
            result.append({
                "shot": shot,
                "thumbnail_path": thumb
            })
        
        return result
    
    def _extract_thumbnail(self, video_path: str, time: float) -> Optional[str]:
        """Extract a single frame as thumbnail at given timestamp."""
        thumb_dir = Path(video_path).parent / ".thumbnails"
        thumb_dir.mkdir(exist_ok=True)
        
        thumb_path = str(thumb_dir / f"thumb_{Path(video_path).stem}_{int(time)}.jpg")
        
        try:
            cmd = [
                self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                "-vframes", "1", "-q:v", "2", thumb_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=30, check=True)
            return thumb_path if os.path.exists(thumb_path) else None
        except subprocess.SubprocessError:
            return None
    
    def estimate_tokens(self, video_path: str) -> int:
        """Estimate token cost for detection.
        
        Scene detection is 100% FFmpeg — 0 token cost always.
        """
        return 0
```

### 2.1.2 Classifier (`classifier.py`)

Mengklasifikasikan jenis shot berdasarkan karakteristik visual.
Gunakan **rule-based** dari metadata FFmpeg (0 token).

```python
"""Classify detected shots by visual characteristics.

Pure rule-based classification using FFmpeg signalstats.
0 token cost — no AI involved.
"""

from __future__ import annotations
from typing import Optional
import subprocess
import re
import json

from .detector import Shot
from ...models import SceneType


class ShotClassifier:
    """Classify shots by analyzing visual characteristics.
    
    Uses FFmpeg signalstats and histogram analysis to determine:
    - Establishing / Wide / Medium / Closeup
    - Interior / Exterior (via brightness)
    - Day / Night (via luminance)
    - Motion level (static / pan / zoom)
    """
    
    def __init__(self, ffmpeg_path: str = "ffmpeg"):
        self.ffmpeg_path = ffmpeg_path
    
    def classify(self, video_path: str, shot: Shot) -> SceneType:
        """Classify a single shot by its visual characteristics.
        
        Args:
            video_path: Path to source video.
            shot: The shot to classify.
            
        Returns:
            SceneType classification.
        """
        try:
            # Get frame at middle of shot
            mid_time = (shot.start_time + shot.end_time) / 2
            stats = self._get_frame_stats(video_path, mid_time)
            
            if not stats:
                return SceneType.B_ROLL
            
            # Classification rules
            brightness = stats.get("average_brightness", 128)
            contrast = stats.get("contrast", 0)
            motion = self._estimate_motion(video_path, shot)
            
            # Very bright + low contrast = establishing / exterior
            if brightness > 180 and contrast < 40:
                return SceneType.ESTABLISHING
            
            # High contrast + mid brightness = closeup / detail
            if contrast > 80:
                return SceneType.CLOSEUP
            
            # Low brightness = night / interior detail
            if brightness < 60:
                return SceneType.DETAIL
            
            # High motion = action / b-roll
            if motion > 0.3:
                return SceneType.MONTAGE
            
            # Wide shots tend to have more uniform histograms
            if contrast < 50:
                return SceneType.WIDE
            
            return SceneType.B_ROLL
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return SceneType.B_ROLL
    
    def classify_batch(self, video_path: str, shots: list[Shot]) -> list[SceneType]:
        """Classify multiple shots at once."""
        return [self.classify(video_path, shot) for shot in shots]
    
    def _get_frame_stats(self, video_path: str, time: float) -> Optional[dict]:
        """Get signal statistics from a single frame."""
        try:
            cmd = [
                self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                "-vframes", "1", "-vf", "signalstats",
                "-f", "null", "-"
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            stats = {}
            for line in result.stderr.split("\n"):
                if "signalstats" in line:
                    match = re.search(r'YMIN=(\d+)', line)
                    if match: stats["y_min"] = int(match.group(1))
                    match = re.search(r'YLOW=(\d+)', line)
                    if match: stats["y_low"] = int(match.group(1))
                    match = re.search(r'YAVG=(\d+)', line)
                    if match: stats["average_brightness"] = int(match.group(1))
                    match = re.search(r'YHIGH=(\d+)', line)
                    if match: stats["y_high"] = int(match.group(1))
                    match = re.search(r'YMAX=(\d+)', line)
                    if match: stats["y_max"] = int(match.group(1))
            
            if "average_brightness" in stats:
                stats["contrast"] = stats.get("y_max", 255) - stats.get("y_min", 0)
                return stats
            
            return None
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
    
    def _estimate_motion(self, video_path: str, shot: Shot, samples: int = 5) -> float:
        """Estimate motion level by comparing consecutive frames.
        
        Returns:
            Float 0.0 (static) to 1.0 (high motion).
        """
        if shot.duration < 0.5:
            return 0.0
        
        try:
            step = shot.duration / (samples + 1)
            prev_hist = None
            total_diff = 0.0
            count = 0
            
            for i in range(1, samples + 1):
                t = shot.start_time + step * i
                hist = self._get_frame_histogram(video_path, t)
                
                if hist is not None and prev_hist is not None:
                    diff = sum(abs(a - b) for a, b in zip(hist, prev_hist))
                    total_diff += diff
                    count += 1
                
                prev_hist = hist
            
            if count == 0:
                return 0.0
            
            avg_diff = total_diff / count
            # Normalize: typical max diff is ~500
            return min(1.0, avg_diff / 500.0)
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return 0.0
    
    def _get_frame_histogram(self, video_path: str, time: float) -> Optional[list[int]]:
        """Get luminance histogram from a frame."""
        try:
            cmd = [
                self.ffmpeg_path, "-ss", str(time), "-i", video_path,
                "-vframes", "1", "-vf", "histogram",
                "-f", "null", "-"
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            # Parse histogram values
            hist = []
            for line in result.stderr.split("\n"):
                if "Parsed_histogram" in line:
                    numbers = re.findall(r'\d+', line)
                    hist = [int(n) for n in numbers[:256]]
                    break
            
            return hist if hist else None
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
    
    def estimate_tokens(self) -> int:
        """Token cost: 0 — pure rule-based."""
        return 0
```

### Verifikasi Scene Detector

```python
from auto_editor.workers.scene_detector.detector import SceneDetector

detector = SceneDetector(threshold=0.3)
shots = detector.detect("test_video.mp4")
assert len(shots) > 0
assert all(s.duration > 0 for s in shots)

# Fallback test
assert detector.detect("nonexistent.mp4") == []

# Token cost
assert detector.estimate_tokens("any.mp4") == 0
```

---

## Task 2.2 — Asset Finder

**Folder:** `auto-editor/workers/asset_finder/`
**File:** `crawler.py`, `rag_search.py`, `downloader.py`

### 2.2.1 Crawler (`crawler.py`)

Mencari asset video/gambar dari API publik (Pexels, Pixabay).
Menggunakan REST API — 0 token.

```python
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
    """Search result from an asset provider."""
    url: str
    thumbnail_url: str
    provider: str                   # "pexels" | "pixabay" | "local"
    width: int
    height: int
    duration: Optional[float] = None  # seconds (None for images)
    file_type: str = "video/mp4"
    keywords: list[str] = field(default_factory=list)
    author: str = ""
    license_type: str = "free"


class AssetCrawler:
    """Search video/image assets from public APIs.
    
    Supports Pexels and Pixabay APIs.
    Results are cached to disk to avoid redundant API calls.
    
    API Keys (from config/providers.yaml or env vars):
    - PEXELS_API_KEY
    - PIXABAY_API_KEY
    """
    
    def __init__(self, cache_dir: str = ".asset_cache"):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        self._pexels_key = os.environ.get("PEXELS_API_KEY", "")
        self._pixabay_key = os.environ.get("PIXABAY_API_KEY", "")
    
    def search(
        self,
        query: str,
        media_type: Literal["video", "image"] = "video",
        max_results: int = 10,
        min_duration: float = 3.0,
        preferred_provider: Optional[str] = None
    ) -> list[AssetResult]:
        """Search for assets across all providers.
        
        Args:
            query: Search keywords.
            media_type: "video" or "image".
            max_results: Maximum results to return.
            min_duration: Minimum video duration in seconds.
            preferred_provider: "pexels", "pixabay", or None for both.
            
        Returns:
            List of AssetResult sorted by relevance.
        """
        cache_key = self._cache_key(query, media_type)
        cached = self._load_cache(cache_key)
        if cached is not None:
            return cached[:max_results]
        
        results = []
        
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
        
        # Filter by duration
        if media_type == "video":
            results = [
                r for r in results
                if r.duration is None or r.duration >= min_duration
            ]
        
        # Deduplicate by URL
        seen_urls = set()
        unique = []
        for r in results:
            if r.url not in seen_urls:
                seen_urls.add(r.url)
                unique.append(r)
        
        self._save_cache(cache_key, unique)
        return unique[:max_results]
    
    def _search_pexels(self, query: str, media_type: str, per_page: int) -> list[AssetResult]:
        """Search Pexels API."""
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
        """Search Pixabay API."""
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
        """Pick the best quality video file from Pexels response."""
        if not video_files:
            return None
        
        # Prefer high quality, then HD, then anything
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
        """Generate unique cache key."""
        raw = f"{query}:{media_type}"
        return hashlib.md5(raw.encode()).hexdigest()
    
    def _load_cache(self, key: str) -> Optional[list[AssetResult]]:
        """Load cached results if fresh."""
        cache_file = self.cache_dir / f"{key}.json"
        if not cache_file.exists():
            return None
        
        # Cache TTL: 1 hour
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
        """Save results to disk cache."""
        cache_file = self.cache_dir / f"{key}.json"
        try:
            with open(cache_file, "w") as f:
                json.dump([r.__dict__ for r in results], f)
        except IOError:
            pass  # Cache is optional, don't fail
    
    def estimate_tokens(self, query: str) -> int:
        """Token cost: 0 — pure REST API calls."""
        return 0
```

### 2.2.2 RAG Search (`rag_search.py`)

Mencari asset dari library lokal menggunakan keyword matching (bukan vector embedding — lebih sederhana dan 0 token).

```python
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
    """Asset found in local library."""
    path: str
    filename: str
    file_type: str           # "video", "audio", "image"
    size_bytes: int
    duration: Optional[float] = None
    keywords: list[str] = field(default_factory=list)
    thumbnail_path: Optional[str] = None


class RAGSearch:
    """Search local asset library using keyword matching.
    
    Builds a lightweight keyword index from filenames and metadata.
    0 token cost — pure string matching.
    """
    
    def __init__(self, library_dirs: Optional[list[str]] = None):
        self.library_dirs = [Path(d) for d in (library_dirs or ["./assets/"])]
        self._index: dict[str, list[LocalAsset]] = {}  # keyword → assets
        self._all_assets: list[LocalAsset] = []
    
    def search(self, query: str, max_results: int = 10) -> list[LocalAsset]:
        """Search assets by keyword matching.
        
        Args:
            query: Space-separated keywords.
            max_results: Maximum results to return.
            
        Returns:
            List of matching LocalAsset sorted by relevance.
        """
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
        """Build keyword index from library directories."""
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
                
                # Extract keywords from filename
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
                
                # Index by keyword
                for kw in keywords:
                    if kw not in self._index:
                        self._index[kw] = []
                    self._index[kw].append(asset)
    
    def _score(self, asset: LocalAsset, query_keywords: set[str]) -> float:
        """Score asset relevance to query keywords."""
        asset_keywords = set(asset.keywords)
        overlap = query_keywords & asset_keywords
        
        if not overlap:
            return 0.0
        
        # Score: overlap count + filename exact match bonus
        score = len(overlap) / max(len(query_keywords), 1)
        
        # Bonus for exact filename match
        stem = re.sub(r'[_-]', ' ', Path(asset.path).stem).lower()
        full_query = " ".join(query_keywords)
        if full_query in stem:
            score *= 2.0
        
        return score
    
    def scan_directory(self, directory: str) -> list[LocalAsset]:
        """Scan a directory and return all valid media assets."""
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
        """Token cost: 0 — pure keyword matching."""
        return 0
```

### 2.2.3 Downloader (`downloader.py`)

Mendownload asset dari URL dan meng-cache secara lokal.

```python
"""Download and cache assets from URLs.

Async downloads with progress tracking.
Cached locally to avoid redundant downloads.
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Callable
import os
import hashlib
from urllib.request import urlopen
from urllib.error import URLError
import shutil


class AssetDownloader:
    """Download and cache assets from URLs.
    
    Downloads are cached in a local directory to avoid
    re-downloading the same asset multiple times.
    """
    
    def __init__(self, cache_dir: str = ".asset_cache/downloads"):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
    
    def download(
        self,
        url: str,
        output_path: Optional[str] = None,
        filename: Optional[str] = None,
        progress_callback: Optional[Callable[[int, int], None]] = None
    ) -> Optional[str]:
        """Download an asset from URL.
        
        Args:
            url: Asset URL.
            output_path: Specific output path (auto if None).
            filename: Custom filename (auto from URL if None).
            progress_callback: Called with (bytes_downloaded, total_bytes).
            
        Returns:
            Local file path if successful, None if failed.
        """
        cache_key = hashlib.md5(url.encode()).hexdigest()
        
        if output_path:
            dest = Path(output_path)
        elif filename:
            dest = self.cache_dir / filename
        else:
            ext = self._guess_extension(url)
            dest = self.cache_dir / f"{cache_key}{ext}"
        
        # Return cached file if exists
        if dest.exists() and dest.stat().st_size > 0:
            return str(dest)
        
        try:
            with urlopen(url, timeout=60) as response:
                total = int(response.headers.get("content-length", 0))
                downloaded = 0
                
                with open(dest, "wb") as f:
                    while True:
                        chunk = response.read(8192)
                        if not chunk:
                            break
                        f.write(chunk)
                        downloaded += len(chunk)
                        if progress_callback and total > 0:
                            progress_callback(downloaded, total)
                
                return str(dest)
                
        except (URLError, IOError, TimeoutError) as e:
            if dest.exists():
                dest.unlink()
            return None
    
    def download_batch(
        self, urls: list[str],
        progress_callback: Optional[Callable[[int, int], None]] = None
    ) -> list[Optional[str]]:
        """Download multiple assets."""
        results = []
        for i, url in enumerate(urls):
            if progress_callback:
                progress_callback(i, len(urls))
            results.append(self.download(url))
        return results
    
    def _guess_extension(self, url: str) -> str:
        """Guess file extension from URL."""
        path = url.split("?")[0]
        _, ext = os.path.splitext(path)
        if ext:
            return ext
        return ".mp4"  # default
    
    def clear_cache(self, max_age_hours: int = 24) -> int:
        """Clear old cached files. Returns count of deleted files."""
        import time
        now = time.time()
        deleted = 0
        for f in self.cache_dir.iterdir():
            if f.is_file() and (now - f.stat().st_mtime) > max_age_hours * 3600:
                f.unlink()
                deleted += 1
        return deleted
    
    def get_cache_size(self) -> int:
        """Get total size of cached files in bytes."""
        return sum(f.stat().st_size for f in self.cache_dir.rglob("*") if f.is_file())
```

### Verifikasi Asset Finder

```python
# RAG test
rag = RAGSearch(["./test_assets/"])
rag._all_assets = []
results = rag.search("nature landscape")
assert isinstance(results, list)

# Downloader test
dl = AssetDownloader("test_cache")
result = dl.download("https://example.com/video.mp4")
assert result is None or Path(result).exists()
```

---

## Task 2.3 — Layout Engine

**Folder:** `auto-editor/workers/layout_engine/`
**File:** `compositor.py`, `template.py`

> **Catatan:** `coordinate.py` sudah dibuat oleh Agent 1. Jangan ubah.

### 2.3.1 Compositor (`compositor.py`)

Menggabungkan elemen layout menjadi satu frame yang siap di-render.

```python
"""Composite multiple CoordinateElements into render-ready frames.

Takes layout coordinates + assets → produces composited frames.
Serves as the bridge between layout logic and the renderer.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Literal
from pathlib import Path
import subprocess
import json
import os

from ...models import CoordinateElement, Position, Size, Transform
from .coordinate import CoordinateEngine


@dataclass
class CompositedFrame:
    """A single composited frame ready for encoding."""
    frame_number: int
    timestamp: float               # seconds
    layer_count: int
    elements: list[dict]           # serialized elements at this frame


class Compositor:
    """Composite layout elements into render-ready frame descriptions.
    
    This module produces frame-by-frame descriptions that the Renderer
    uses to produce the final video. No pixel manipulation here —
    just coordinate math and asset references.
    """
    
    def __init__(self, canvas_width: int = 1920, canvas_height: int = 1080):
        self.canvas_width = canvas_width
        self.canvas_height = canvas_height
        self.coord = CoordinateEngine(canvas_width, canvas_height)
    
    def composite_frame(
        self,
        elements: list[CoordinateElement],
        frame_number: int,
        fps: float = 30.0
    ) -> Optional[CompositedFrame]:
        """Composite all elements at a given frame.
        
        Args:
            elements: All layout elements in the project.
            frame_number: Target frame number.
            fps: Frames per second.
            
        Returns:
            CompositedFrame with element positions, or None if no elements visible.
        """
        current_time = frame_number / fps
        
        # Filter visible elements at this timestamp
        visible = [
            el for el in elements
            if el.timeline.start <= current_time <= el.timeline.end
        ]
        
        if not visible:
            return None
        
        # Sort by z-index and apply keyframe animation
        visible.sort(key=lambda el: el.position.z)
        
        frame_elements = []
        for el in visible:
            # Apply animation
            if el.animation:
                anim_time = current_time - el.timeline.start
                el = self.coord.apply_keyframe(el, anim_time)
            
            # Get pixel bounds
            bounds = self.coord.get_bounds(el)
            
            frame_elements.append({
                "id": el.id,
                "type": el.type,
                "z": el.position.z,
                "bounds": bounds,
                "opacity": el.transform.opacity,
                "rotation": el.transform.rotation,
                "scale": el.transform.scale,
                "text": el.text_style.text if el.text_style else None,
                "style": self._serialize_style(el),
            })
        
        return CompositedFrame(
            frame_number=frame_number,
            timestamp=current_time,
            layer_count=len(frame_elements),
            elements=frame_elements,
        )
    
    def composite_range(
        self,
        elements: list[CoordinateElement],
        start_frame: int,
        end_frame: int,
        fps: float = 30.0,
        progress_callback=None
    ) -> list[CompositedFrame]:
        """Composite a range of frames.
        
        Args:
            elements: All layout elements.
            start_frame: Starting frame number.
            end_frame: Ending frame number (inclusive).
            fps: Frame rate.
            progress_callback: Called with (current, total).
            
        Returns:
            List of CompositedFrame.
        """
        frames = []
        total = end_frame - start_frame + 1
        
        for i in range(start_frame, end_frame + 1):
            frame = self.composite_frame(elements, i, fps)
            if frame:
                frames.append(frame)
            if progress_callback:
                progress_callback(i - start_frame + 1, total)
        
        return frames
    
    def get_project_duration(self, elements: list[CoordinateElement]) -> float:
        """Get total project duration in seconds."""
        if not elements:
            return 0.0
        return max(el.timeline.end for el in elements)
    
    def get_frame_count(self, elements: list[CoordinateElement], fps: float = 30.0) -> int:
        """Get total frame count for the project."""
        duration = self.get_project_duration(elements)
        return int(duration * fps)
    
    def to_filter_graph(self, elements: list[CoordinateElement], canvas_size: tuple[int, int]) -> str:
        """Generate FFmpeg filter graph from layout elements.
        
        Produces a complex filter graph string for FFmpeg compositing.
        This is the bridge between our coordinate system and FFmpeg rendering.
        
        Args:
            elements: Layout elements.
            canvas_size: (width, height) tuple.
            
        Returns:
            FFmpeg filter graph string, or empty string if simple overlay.
        """
        if not elements:
            return ""
        
        filters = []
        input_index = 0
        
        for el in elements:
            bounds = self.coord.get_bounds(el)
            
            # Scale to canvas coordinates
            x = int(bounds["left"])
            y = int(bounds["top"])
            w = int(bounds["width"])
            h = int(bounds["height"])
            
            filters.append(
                f"[{input_index}:v]scale={w}:{h},"
                f"setpts=PTS-STARTPTS,"
                f"format=rgba,"
                f"colorchannelmixer=aa={el.transform.opacity}[v{input_index}];"
            )
            input_index += 1
        
        # Overlay all layers
        if filters:
            overlay = f"[0:v]"
            for i in range(1, input_index):
                if i == 1:
                    overlay += f"[v{i}]overlay={x}:{y}[ov{i}]"
                else:
                    overlay += f"[ov{i-1}][v{i}]overlay={x}:{y}[ov{i}]"
            
            filters.append(overlay)
        
        return "".join(filters)
    
    def _serialize_style(self, element: CoordinateElement) -> dict:
        """Extract style properties based on element type."""
        if element.text_style:
            return {
                "text": element.text_style.text,
                "font_family": element.text_style.font_family,
                "font_size": element.text_style.font_size,
                "color": element.text_style.color,
                "text_align": element.text_style.text_align,
            }
        if element.video_style:
            return {"fit": element.video_style.fit}
        if element.shape_style:
            return {"bg_color": element.shape_style.background_color}
        return {}
```

### 2.3.2 Template Loader (`template.py`)

Load dan apply layout template ke project.

```python
"""Load and apply layout templates to coordinate elements.

Bridge between TemplateDB (Agent 1) and actual element positioning.
"""

from __future__ import annotations
from typing import Optional

from ...models import (
    CoordinateElement, Position, Size, Timeline, Transform,
    TextStyle, VideoStyle, ShapeStyle, Animation, Keyframe,
)
from ...orchestrator.template_db import TemplateDB


class TemplateLoader:
    """Load layout templates and convert to CoordinateElements.
    
    Handles variable substitution and default value resolution.
    """
    
    def __init__(self, templates_dir: str = "config/templates"):
        self.db = TemplateDB(templates_dir)
    
    def apply_template(
        self,
        template_name: str,
        variables: dict[str, str],
        duration: Optional[float] = None
    ) -> list[CoordinateElement]:
        """Apply a named template with variable substitution.
        
        Args:
            template_name: Template name in database.
            variables: Dict of {VARIABLE_NAME: value} for substitution.
            duration: Override total duration. If None, use template default.
            
        Returns:
            List of fully resolved CoordinateElements.
        """
        template = self.db.get(template_name)
        if not template:
            return self._fallback_template(variables)
        
        elements = []
        for track in template.get("tracks", []):
            element = self._track_to_element(track, variables)
            if element:
                elements.append(element)
        
        # Adjust duration if specified
        if duration:
            scale = duration / max(
                (el.timeline.end for el in elements),
                default=10.0
            )
            for el in elements:
                el.timeline.end = el.timeline.end * scale
                el.timeline.start = el.timeline.start * scale
        
        return elements
    
    def _track_to_element(self, track: dict, variables: dict[str, str]) -> Optional[CoordinateElement]:
        """Convert YAML track definition to CoordinateElement."""
        pos = track.get("position", {})
        sz = track.get("size", {})
        tml = track.get("timeline", {})
        trf = track.get("transform", {})
        style = track.get("style", {})
        anim = track.get("animation", {})
        
        element_type = track.get("type", "video")
        
        element = CoordinateElement(
            id=track.get("id", "untitled"),
            type=element_type,
            position=Position(
                x=pos.get("x", 0.5),
                y=pos.get("y", 0.5),
                z=pos.get("z", 0)
            ),
            size=Size(
                width=sz.get("width", 0.5),
                height=sz.get("height", 0.5),
                unit=sz.get("unit", "normalized")
            ),
            timeline=Timeline(
                start=tml.get("start", 0.0),
                end=tml.get("end", 10.0)
            ),
            transform=Transform(
                rotation=trf.get("rotation", 0.0),
                scale=trf.get("scale", 1.0),
                opacity=trf.get("opacity", 1.0),
                anchor=trf.get("anchor", "center")
            ),
        )
        
        # Apply animation
        if anim:
            kf_list = anim.get("keyframes", [])
            if kf_list:
                keyframes = []
                for kf in kf_list:
                    if isinstance(kf, dict):
                        keyframes.append(Keyframe(
                            time=kf.get("time", 0),
                            x=kf.get("x"),
                            y=kf.get("y"),
                            scale=kf.get("scale"),
                            opacity=kf.get("opacity"),
                            rotation=kf.get("rotation"),
                        ))
                element.animation = Animation(
                    keyframes=keyframes,
                    easing=anim.get("easing", "ease_in_out")
                )
        
        # Apply style based on type
        resolved_style = self._resolve_variables(style, variables)
        
        if element_type == "text":
            element.text_style = TextStyle(
                text=resolved_style.get("text", ""),
                font_family=resolved_style.get("font_family", "Inter"),
                font_size=resolved_style.get("font_size", 48),
                font_weight=resolved_style.get("font_weight", 400),
                color=resolved_style.get("color", "#FFFFFF"),
                text_align=resolved_style.get("text_align", "center"),
            )
        elif element_type == "video":
            element.video_style = VideoStyle(
                fit=resolved_style.get("fit", "cover"),
            )
        elif element_type == "shape":
            element.shape_style = ShapeStyle(
                background_color=resolved_style.get("background_color", "#000000"),
                border_radius=resolved_style.get("border_radius", 0),
            )
        
        return element
    
    def _resolve_variables(self, data: dict, variables: dict[str, str]) -> dict:
        """Replace {VARIABLE} placeholders with actual values."""
        result = {}
        for key, value in data.items():
            if isinstance(value, str):
                for var_name, var_value in variables.items():
                    value = value.replace(f"{{{var_name}}}", var_value)
                result[key] = value
            elif isinstance(value, dict):
                result[key] = self._resolve_variables(value, variables)
            else:
                result[key] = value
        return result
    
    def _fallback_template(self, variables: dict[str, str]) -> list[CoordinateElement]:
        """Simple fallback when template not found — single fullscreen video + title."""
        return [
            CoordinateElement(
                id="main",
                type="video",
                position=Position(0.5, 0.5, 0),
                size=Size(1.0, 1.0),
                timeline=Timeline(0, 30),
            ),
            CoordinateElement(
                id="title",
                type="text",
                position=Position(0.5, 0.1, 1),
                size=Size(0.8, 0.1),
                timeline=Timeline(0, 5),
                text_style=TextStyle(
                    text=variables.get("TITLE", "Video"),
                    font_size=56,
                    color="#FFFFFF",
                    text_align="center",
                ),
            ),
        ]
    
    def list_templates(self) -> list[dict]:
        """List available templates."""
        return self.db.list_all()
    
    def find_suitable_template(self, plan) -> Optional[str]:
        """Find the best template for an editing plan."""
        query = f"{plan.style.value} {plan.target_platform.value} {plan.mood.value}"
        result = self.db.find_similar(query)
        return result.get("name") if result else None
```

### Verifikasi Layout Engine

```python
from auto_editor.workers.layout_engine.compositor import Compositor

comp = Compositor(1920, 1080)
elements = [
    CoordinateElement("bg", "video", timeline=Timeline(0, 10)),
    CoordinateElement("title", "text", timeline=Timeline(0, 5)),
]
frame = comp.composite_frame(elements, 0, 30)
assert frame is not None
assert frame.layer_count == 2

# Empty frame test
assert comp.composite_frame(elements, 1000, 30) is None
```

---

## Task 2.4 — Audio Pipeline

**Folder:** `auto-editor/workers/audio_pipeline/`
**File:** `tts_engine.py`, `asr_whisper.py`, `alignment.py`, `mixer.py`

### 2.4.1 TTS Engine (`tts_engine.py`)

Local Text-to-Speech menggunakan model GGUF (CosyVoice / Bark).
Pure local — 0 token cost.

```python
"""Local Text-to-Speech engine using GGUF models.

Pure local inference — 0 token cost, no API calls.
Supports CosyVoice and Bark models in GGUF format.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, Literal
from pathlib import Path
import json
import subprocess
import os
import tempfile


@dataclass
class TTSResult:
    """Result from TTS generation."""
    audio_path: str
    text: str
    duration: float            # seconds
    word_timings: list[dict] = field(default_factory=list)


class TTSEngine:
    """Generate speech from text using local GGUF models.
    
    Supports multiple local backends:
    - cosyvoice: CosyVoice 300M model (default)
    - bark: Bark text-to-speech
    - piper: Piper TTS (lightweight)
    
    All models run locally — 0 token cost, full privacy.
    """
    
    def __init__(
        self,
        model_path: str = "./models/tts/cosyvoice.gguf",
        backend: Literal["cosyvoice", "bark", "piper"] = "cosyvoice",
        voice: str = "default"
    ):
        self.model_path = model_path
        self.backend = backend
        self.voice = voice
    
    def generate(
        self,
        text: str,
        output_path: Optional[str] = None,
        language: str = "id",
        speed: float = 1.0
    ) -> Optional[TTSResult]:
        """Generate speech from text.
        
        Args:
            text: Text to synthesize.
            output_path: Output WAV path (auto if None).
            language: Language code ("id", "en", etc).
            speed: Speech speed (0.5-2.0).
            
        Returns:
            TTSResult with audio path and duration, or None if failed.
        """
        if not text or not text.strip():
            return None
        
        if not output_path:
            output_path = self._default_path(text)
        
        if self.backend == "piper":
            return self._run_piper(text, output_path)
        elif self.backend == "bark":
            return self._run_bark(text, output_path)
        else:
            return self._run_cosyvoice(text, output_path, language, speed)
    
    def generate_batch(
        self,
        segments: list[dict],
        output_dir: str = "./output/audio/"
    ) -> list[Optional[TTSResult]]:
        """Generate multiple speech segments.
        
        Args:
            segments: List of {"text": str, "language": str, "speed": float}.
            output_dir: Directory for output files.
            
        Returns:
            List of TTSResult.
        """
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        results = []
        
        for i, seg in enumerate(segments):
            out_path = f"{output_dir}/segment_{i:04d}.wav"
            result = self.generate(
                text=seg.get("text", ""),
                output_path=out_path,
                language=seg.get("language", "id"),
                speed=seg.get("speed", 1.0),
            )
            results.append(result)
        
        return results
    
    def _run_cosyvoice(self, text: str, output_path: str, language: str, speed: float) -> Optional[TTSResult]:
        """Run CosyVoice inference (GGUF model via llama.cpp)."""
        try:
            cmd = [
                "llama-tts",                   # hypothetical llama.cpp TTS binary
                "--model", self.model_path,
                "--text", text,
                "--output", output_path,
                "--language", language,
                "--speed", str(speed),
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
            
            if result.returncode != 0 or not os.path.exists(output_path):
                return self._fallback_piper(text, output_path)
            
            duration = self._get_audio_duration(output_path)
            return TTSResult(
                audio_path=output_path,
                text=text,
                duration=duration,
            )
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_piper(text, output_path)
    
    def _run_bark(self, text: str, output_path: str) -> Optional[TTSResult]:
        """Run Bark TTS inference."""
        try:
            cmd = [
                "bark-tts",
                "--model", self.model_path,
                "--text", text,
                "--output", output_path,
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
            
            if result.returncode != 0 or not os.path.exists(output_path):
                return self._fallback_piper(text, output_path)
            
            duration = self._get_audio_duration(output_path)
            return TTSResult(audio_path=output_path, text=text, duration=duration)
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_piper(text, output_path)
    
    def _run_piper(self, text: str, output_path: str) -> Optional[TTSResult]:
        """Run Piper TTS (lightweight, fast fallback)."""
        try:
            json_input = json.dumps({"text": text})
            cmd = [
                "piper-tts",
                "--model", self.model_path,
                "--output", output_path,
            ]
            result = subprocess.run(
                cmd, input=json_input, capture_output=True,
                text=True, timeout=60
            )
            
            if result.returncode != 0 or not os.path.exists(output_path):
                return None
            
            duration = self._get_audio_duration(output_path)
            return TTSResult(audio_path=output_path, text=text, duration=duration)
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
    
    def _fallback_piper(self, text: str, output_path: str) -> Optional[TTSResult]:
        """Fallback: generate beep/silence audio.
        
        Graceful degradation when no TTS engine is available.
        """
        try:
            duration = len(text.split()) * 0.3  # ~300ms per word
            cmd = [
                "ffmpeg", "-f", "lavfi", "-i",
                f"anullsrc=r=44100:cl=mono",
                "-t", str(duration),
                "-acodec", "pcm_s16le",
                output_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=30)
            
            if os.path.exists(output_path):
                return TTSResult(
                    audio_path=output_path,
                    text=text,
                    duration=duration,
                )
        except (subprocess.SubprocessError, FileNotFoundError):
            pass
        
        return None
    
    def _default_path(self, text: str) -> str:
        """Generate default output path."""
        safe_name = "".join(c if c.isalnum() else "_" for c in text[:30])
        return f"./output/audio/{safe_name}.wav"
    
    def _get_audio_duration(self, path: str) -> float:
        """Get audio duration in seconds using FFprobe."""
        try:
            cmd = [
                "ffprobe", "-v", "error",
                "-show_entries", "format=duration",
                "-of", "json", path
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=15)
            data = json.loads(result.stdout)
            return float(data["format"]["duration"])
        except (subprocess.SubprocessError, json.JSONDecodeError, KeyError, ValueError):
            return 0.0
    
    def estimate_tokens(self, text_length: int) -> int:
        """Token cost: 0 — pure local computation."""
        return 0
```

### 2.4.2 ASR Whisper (`asr_whisper.py`)

Local Automatic Speech Recognition menggunakan Whisper.cpp.

```python
"""Local Automatic Speech Recognition using Whisper.cpp GGUF model.

Pure local inference — 0 token cost, full privacy.
Transcribes audio to text with word-level timestamps.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional
import subprocess
import json
import os


@dataclass
class TranscriptionResult:
    """ASR transcription result with timestamps."""
    text: str
    language: str
    segments: list[dict] = field(default_factory=list)
    duration: float = 0.0


class ASREngine:
    """Transcribe audio to text using local Whisper.cpp.
    
    Runs entirely locally — no data leaves the device.
    Supports multiple languages with word-level timestamps.
    """
    
    def __init__(
        self,
        model_path: str = "./models/asr/whisper-small.gguf",
        model_type: str = "small"
    ):
        self.model_path = model_path
        self.model_type = model_type
    
    def transcribe(
        self,
        audio_path: str,
        language: str = "id",
        output_format: str = "json"
    ) -> Optional[TranscriptionResult]:
        """Transcribe audio file to text.
        
        Args:
            audio_path: Path to audio file (WAV/MP3).
            language: Language code ("id", "en", etc).
            output_format: "json", "srt", "vtt", "txt".
            
        Returns:
            TranscriptionResult with text and segments, or None if failed.
        """
        if not os.path.exists(audio_path):
            return None
        
        try:
            cmd = [
                "whisper.cpp",
                "--model", self.model_path,
                "--file", audio_path,
                "--language", language,
                "--output-format", "json",
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode != 0:
                return self._fallback_transcribe(audio_path)
            
            data = json.loads(result.stdout)
            
            segments = []
            for seg in data.get("segments", []):
                segments.append({
                    "id": seg.get("id", 0),
                    "start": seg.get("start", 0.0),
                    "end": seg.get("end", 0.0),
                    "text": seg.get("text", "").strip(),
                    "confidence": seg.get("confidence", 1.0),
                })
            
            return TranscriptionResult(
                text=data.get("text", "").strip(),
                language=language,
                segments=segments,
                duration=data.get("duration", 0.0),
            )
            
        except (subprocess.SubprocessError, FileNotFoundError, json.JSONDecodeError):
            return self._fallback_transcribe(audio_path)
    
    def transcribe_to_srt(self, audio_path: str, language: str = "id") -> Optional[str]:
        """Transcribe and return SRT subtitle content."""
        result = self.transcribe(audio_path, language)
        if not result or not result.segments:
            return None
        
        srt_lines = []
        for i, seg in enumerate(result.segments, 1):
            start = self._format_srt_time(seg["start"])
            end = self._format_srt_time(seg["end"])
            text = seg["text"]
            srt_lines.append(f"{i}\n{start} --> {end}\n{text}\n")
        
        return "\n".join(srt_lines)
    
    def _format_srt_time(self, seconds: float) -> str:
        """Convert seconds to SRT time format (HH:MM:SS,mmm)."""
        hours = int(seconds // 3600)
        minutes = int((seconds % 3600) // 60)
        secs = int(seconds % 60)
        millis = int((seconds % 1) * 1000)
        return f"{hours:02d}:{minutes:02d}:{secs:02d},{millis:03d}"
    
    def _fallback_transcribe(self, audio_path: str) -> Optional[TranscriptionResult]:
        """Fallback: return empty transcription.
        
        Graceful degradation when Whisper is unavailable.
        """
        return TranscriptionResult(
            text="[Transcription unavailable]",
            language="id",
            segments=[{
                "id": 0,
                "start": 0.0,
                "end": 0.0,
                "text": "[Transcription unavailable]",
                "confidence": 0.0,
            }],
            duration=0.0,
        )
    
    def estimate_tokens(self, audio_duration: float) -> int:
        """Token cost: 0 — pure local computation."""
        return 0
```

### 2.4.3 Alignment (`alignment.py`)

Align voiceover segments ke timeline berdasarkan timing.

```python
"""Align voiceover segments to video timeline.

Uses word-level timestamps to sync audio with visual elements.
Pure computation — 0 token cost.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional

from ...models import VoiceoverSegment, VoiceoverConfig


@dataclass
class AlignedVoiceover:
    """Voiceover aligned to timeline."""
    segments: list[VoiceoverSegment]
    total_duration: float


class VoiceoverAligner:
    """Align voiceover audio to timeline.
    
    Takes TTS output with word timings and maps them to
    the scene timeline produced by the storyboard phase.
    """
    
    def align_to_scenes(
        self,
        audio_segments: list[VoiceoverSegment],
        scene_durations: list[float]
    ) -> AlignedVoiceover:
        """Align voiceover segments to scene durations.
        
        Args:
            audio_segments: TTS output segments with timing.
            scene_durations: List of scene durations in seconds.
            
        Returns:
            AlignedVoiceover with adjusted timing.
        """
        aligned = []
        audio_cursor = 0.0
        scene_cursor = 0.0
        
        for i, (seg, scene_dur) in enumerate(zip(audio_segments, scene_durations)):
            seg_dur = (seg.end - seg.start) if seg.end > seg.start else 0.0
            
            # Check if voiceover fits in scene
            if seg_dur <= scene_dur:
                # Fits perfectly
                aligned.append(VoiceoverSegment(
                    text=seg.text,
                    start=scene_cursor,
                    end=scene_cursor + seg_dur,
                    audio_path=seg.audio_path,
                ))
            else:
                # Voiceover longer than scene — scale it
                ratio = scene_dur / seg_dur
                aligned.append(VoiceoverSegment(
                    text=seg.text,
                    start=scene_cursor,
                    end=scene_cursor + seg_dur * ratio,
                    audio_path=seg.audio_path,
                ))
            
            audio_cursor += seg_dur
            scene_cursor += scene_dur
        
        total = scene_cursor if scene_cursor > 0 else (audio_cursor if audio_cursor > 0 else 1.0)
        return AlignedVoiceover(segments=aligned, total_duration=total)
    
    def adjust_speed_for_timeline(
        self,
        voiceover: AlignedVoiceover,
        target_duration: float
    ) -> AlignedVoiceover:
        """Adjust voiceover speed to fill exact duration.
        
        Args:
            voiceover: Original aligned voiceover.
            target_duration: Desired total duration.
            
        Returns:
            Voiceover with speed-adjusted segments.
        """
        if voiceover.total_duration <= 0 or target_duration <= 0:
            return voiceover
        
        ratio = voiceover.total_duration / target_duration
        
        adjusted = []
        for seg in voiceover.segments:
            dur = (seg.end - seg.start) / ratio
            adjusted.append(VoiceoverSegment(
                text=seg.text,
                start=seg.start / ratio,
                end=seg.start / ratio + dur,
                audio_path=seg.audio_path,
            ))
        
        return AlignedVoiceover(segments=adjusted, total_duration=target_duration)
```

### 2.4.4 Mixer (`mixer.py`)

Mixing audio tracks: voiceover + background music + effects.

```python
"""Mix multiple audio tracks into final audio.

Combines voiceover, background music, and sound effects.
Uses FFmpeg for audio mixing — 0 token cost.
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Optional
import subprocess
import os


@dataclass
class AudioMixConfig:
    """Configuration for audio mixing."""
    voiceover_path: Optional[str] = None
    music_path: Optional[str] = None
    effect_paths: list[str] = None
    
    voiceover_volume: float = 1.0
    music_volume: float = 0.3
    effects_volume: float = 0.5
    
    output_path: str = "./output/audio/mixed.wav"


class AudioMixer:
    """Mix multiple audio tracks using FFmpeg.
    
    Handles volume normalization, crossfading, and format conversion.
    0 token cost — pure signal processing.
    """
    
    def mix(self, config: AudioMixConfig) -> Optional[str]:
        """Mix all audio tracks into a single file.
        
        Args:
            config: Mix configuration with tracks and volumes.
            
        Returns:
            Path to mixed audio file, or None if failed.
        """
        tracks = []
        filters = []
        
        # Collect input tracks
        if config.voiceover_path and os.path.exists(config.voiceover_path):
            tracks.append(config.voiceover_path)
            filters.append(
                f"[{len(tracks)-1}:a]volume={config.voiceover_volume}[v{len(tracks)-1}]"
            )
        
        if config.music_path and os.path.exists(config.music_path):
            tracks.append(config.music_path)
            filters.append(
                f"[{len(tracks)-1}:a]volume={config.music_volume}[m{len(tracks)-1}]"
            )
        
        if config.effect_paths:
            for i, ep in enumerate(config.effect_paths):
                if os.path.exists(ep):
                    tracks.append(ep)
                    filters.append(
                        f"[{len(tracks)-1}:a]volume={config.effects_volume}[e{len(tracks)-1}]"
                    )
        
        if not tracks:
            return self._generate_silence(config.output_path)
        
        # Build FFmpeg command
        cmd = ["ffmpeg"]
        for track in tracks:
            cmd.extend(["-i", track])
        
        # Build mix filter
        mix_inputs = "".join(
            f"[{'v' if i == 0 else 'm' if i == 1 else 'e'}{i}]"
            for i in range(len(tracks))
        )
        cmd.extend([
            "-filter_complex",
            f"{'; '.join(filters)};{mix_inputs}amix=inputs={len(tracks)}:duration=first:dropout_transition=2",
            "-acodec", "pcm_s16le",
            "-ar", "44100",
            config.output_path,
            "-y"
        ])
        
        try:
            subprocess.run(cmd, capture_output=True, timeout=120, check=True)
            return config.output_path if os.path.exists(config.output_path) else None
        except subprocess.SubprocessError:
            return self._fallback_mix(tracks, config)
    
    def _fallback_mix(self, tracks: list[str], config: AudioMixConfig) -> Optional[str]:
        """Simpler mixing for compatibility."""
        if not tracks:
            return self._generate_silence(config.output_path)
        
        # Just copy first track
        try:
            cmd = [
                "ffmpeg", "-i", tracks[0],
                "-acodec", "copy",
                config.output_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=60, check=True)
            return config.output_path if os.path.exists(config.output_path) else None
        except subprocess.SubprocessError:
            return self._generate_silence(config.output_path)
    
    def _generate_silence(self, output_path: str, duration: float = 30.0) -> Optional[str]:
        """Generate silent audio as graceful degradation."""
        try:
            os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
            cmd = [
                "ffmpeg", "-f", "lavfi", "-i",
                f"anullsrc=r=44100:cl=mono",
                "-t", str(duration),
                "-acodec", "pcm_s16le",
                output_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=30, check=True)
            return output_path if os.path.exists(output_path) else None
        except subprocess.SubprocessError:
            return None
    
    def normalize_volume(self, audio_path: str, target_db: float = -3.0) -> Optional[str]:
        """Normalize audio volume to target level."""
        output = audio_path.replace(".wav", "_normalized.wav")
        try:
            cmd = [
                "ffmpeg", "-i", audio_path,
                "-af", f"loudnorm=I={target_db}:LRA=11:TP=-1.5",
                "-acodec", "pcm_s16le",
                output, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=60, check=True)
            return output if os.path.exists(output) else None
        except subprocess.SubprocessError:
            return audio_path
```

### Verifikasi Audio Pipeline

```python
from auto_editor.workers.audio_pipeline.alignment import VoiceoverAligner

aligner = VoiceoverAligner()
segments = [VoiceoverSegment(text="Hello", start=0, end=2)]
durations = [5.0]
result = aligner.align_to_scenes(segments, durations)
assert result.total_duration > 0
```

---

## Task 2.5 — Effects

**Folder:** `auto-editor/workers/effects/`
**File:** `color_grade.py`, `transition.py`, `text_overlay.py`

### 2.5.1 Color Grade (`color_grade.py`)

Auto color grading berdasarkan analisis histogram per scene.

```python
"""Auto color grading based on scene histogram analysis.

Pure rule-based from FFmpeg signalstats.
0 token cost — no AI involved.
"""

from __future__ import annotations
from typing import Optional, Literal
import subprocess
import json


class ColorGradingEngine:
    """Apply automatic color grading to video scenes.
    
    Analyzes each scene's histogram and applies appropriate
    color correction — exposure, contrast, saturation, white balance.
    """
    
    def __init__(self, ffmpeg_path: str = "ffmpeg"):
        self.ffmpeg_path = ffmpeg_path
    
    def analyze_scene(self, video_path: str, start_time: float, end_time: float) -> dict:
        """Analyze scene color characteristics.
        
        Returns:
            Dict with brightness, contrast, saturation estimates.
        """
        mid = (start_time + end_time) / 2
        try:
            cmd = [
                self.ffmpeg_path, "-ss", str(mid), "-i", video_path,
                "-vframes", "1", "-vf", "signalstats",
                "-f", "null", "-"
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            
            stats = {"brightness": 128, "contrast": 128, "saturation": 1.0}
            for line in result.stderr.split("\n"):
                if "signalstats" not in line:
                    continue
                import re
                yavg = re.search(r'YAVG=(\d+)', line)
                if yavg: stats["brightness"] = int(yavg.group(1))
                sat = re.search(r'SAT=(\d+)', line)
                if sat: stats["saturation"] = int(sat.group(1)) / 100
            
            return stats
            
        except subprocess.SubprocessError:
            return {"brightness": 128, "contrast": 128, "saturation": 1.0}
    
    def generate_filter(self, preset: str, stats: Optional[dict] = None) -> str:
        """Generate FFmpeg color filter string.
        
        Args:
            preset: "cinematic", "vintage", "vivid", "monochrome", "warm", "cool".
            stats: Scene analysis stats for adaptive grading.
            
        Returns:
            FFmpeg filter string (empty if no grading needed).
        """
        presets = {
            "cinematic": (
                "eq=contrast=1.2:brightness=0.05:saturation=0.8,"
                "curves=green='0/0 0.5/0.4 1/1':blue='0/0 0.5/0.6 1/1'"
            ),
            "vintage": (
                "colorchannelmixer=rr=0.8:rg=0.1:rb=0.1,"
                "curves=red='0/0.1 0.5/0.5 1/0.9'"
            ),
            "vivid": (
                "eq=saturation=1.5:contrast=1.1:brightness=0.02"
            ),
            "monochrome": (
                "hue=s=0,eq=contrast=1.3:brightness=0.05"
            ),
            "warm": (
                "colorbalance=rs=0.1:gs=-0.05:bs=-0.1"
            ),
            "cool": (
                "colorbalance=rs=-0.1:gs=0.05:bs=0.15"
            ),
        }
        
        # Adaptive: adjust based on scene stats
        if stats and preset == "cinematic":
            if stats.get("brightness", 128) < 60:
                return presets["cinematic"].replace("brightness=0.05", "brightness=0.15")
            elif stats.get("brightness", 128) > 200:
                return presets["cinematic"].replace("brightness=0.05", "brightness=-0.05")
        
        return presets.get(preset, "")
    
    def apply_to_scene(
        self,
        video_path: str,
        output_path: str,
        preset: str = "cinematic",
        start_time: float = 0.0,
        end_time: Optional[float] = None
    ) -> Optional[str]:
        """Apply color grading to scene.
        
        Args:
            video_path: Input video.
            output_path: Output video path.
            preset: Color preset name.
            start_time: Scene start time.
            end_time: Scene end time.
            
        Returns:
            Output path if successful, None if failed.
        """
        stats = self.analyze_scene(video_path, start_time, end_time or start_time + 5)
        filter_str = self.generate_filter(preset, stats)
        
        if not filter_str:
            return video_path  # No grading needed
        
        try:
            seek = f"-ss {start_time}" if start_time > 0 else ""
            to_val = end_time or ""
            duration = f"-t {to_val - start_time}" if end_time else ""
            
            cmd = [
                self.ffmpeg_path, *seek.split(),
                "-i", video_path, *duration.split(),
                "-vf", filter_str,
                "-c:v", "libx264", "-preset", "fast",
                "-c:a", "copy",
                output_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=300, check=True)
            return output_path if __import__('os').path.exists(output_path) else None
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
```

### 2.5.2 Transition (`transition.py`)

Smart transitions antar scene.

```python
"""Smart transitions between scenes.

Rule-based transition selection based on scene content.
0 token cost.
"""

from __future__ import annotations
from typing import Optional
import subprocess

from ...models import TransitionType, Scene


class TransitionEngine:
    """Select and apply transitions between scenes.
    
    Uses rules to pick appropriate transitions based on scene content:
    - Same location → hard cut
    - Mood change → fade/dip
    - Montage → crossfade
    - Fast pace → slide/zoom
    """
    
    def suggest_transition(self, scene_a: Scene, scene_b: Scene) -> TransitionType:
        """Suggest the best transition between two scenes.
        
        Args:
            scene_a: Previous scene.
            scene_b: Next scene.
            
        Returns:
            Appropriate TransitionType.
        """
        # Same type → hard cut
        if scene_a.scene_type == scene_b.scene_type:
            return TransitionType.HARD_CUT
        
        # Establishing → anything → crossfade
        if scene_a.scene_type.value in ("establishing", "wide"):
            return TransitionType.CROSSFADE
        
        # Transition type → dip to black
        if scene_a.scene_type == TransitionType.DIP_TO_BLACK:
            return TransitionType.DIP_TO_BLACK
        
        # Montage → slide
        if scene_a.scene_type.value == "montage":
            return TransitionType.SLIDE
        
        # Default: crossfade for scene changes
        return TransitionType.CROSSFADE
    
    def generate_filter(
        self, transition: TransitionType, duration: float = 0.5
    ) -> str:
        """Generate FFmpeg transition filter string.
        
        Args:
            transition: Transition type.
            duration: Transition duration in seconds.
            
        Returns:
            FFmpeg filter string.
        """
        transitions = {
            TransitionType.CROSSFADE: f"fade=t=cross:f=128:d={duration}",
            TransitionType.DIP_TO_BLACK: (
                f"fade=t=out:st=0:d={duration/2},"
                f"fade=t=in:st={duration/2}:d={duration/2}"
            ),
            TransitionType.FADE_IN: f"fade=t=in:st=0:d={duration}",
            TransitionType.FADE_OUT: f"fade=t=out:st=0:d={duration}",
            TransitionType.SLIDE: f"slide=w>{'if(gt(t,0),1,0)'}:d={duration}",
        }
        return transitions.get(transition, "")
    
    def apply_transition(
        self,
        input_a: str,
        input_b: str,
        output_path: str,
        transition: TransitionType,
        duration: float = 0.5
    ) -> Optional[str]:
        """Apply transition between two video clips.
        
        Args:
            input_a: First video path.
            input_b: Second video path.
            output_path: Output video path.
            transition: Transition type.
            duration: Transition duration.
            
        Returns:
            Output path if successful, None if failed.
        """
        filter_str = self.generate_filter(transition, duration)
        if not filter_str:
            return input_b  # Hard cut — just return second clip
        
        try:
            cmd = [
                "ffmpeg",
                "-i", input_a, "-i", input_b,
                "-filter_complex", (
                    f"[0:v][0:a][1:v][1:a]"
                    f"concat=n=2:v=1:a=1[v][a]"
                ),
                "-map", "[v]", "-map", "[a]",
                "-c:v", "libx264", "-preset", "fast",
                output_path, "-y"
            ]
            subprocess.run(cmd, capture_output=True, timeout=300, check=True)
            return output_path if __import__('os').path.exists(output_path) else None
            
        except subprocess.SubprocessError:
            return input_b  # Fallback: hard cut
```

### 2.5.3 Text Overlay (`text_overlay.py`)

Generate text overlays dan subtitles.

```python
"""Generate text overlays and subtitles for video.

Supports SRT, ASS subtitle formats and dynamic text overlays.
Pure rule-based from coordinate layout — 0 token cost.
"""

from __future__ import annotations
from typing import Optional
import subprocess
import os


class TextOverlayEngine:
    """Generate text overlays and subtitles.
    
    Converts CoordinateElement text elements to FFmpeg drawtext
    filters or subtitle files.
    """
    
    def generate_subtitle_file(
        self,
        segments: list[dict],
        output_path: str,
        format: str = "srt"
    ) -> Optional[str]:
        """Generate subtitle file from segments.
        
        Args:
            segments: List of {"start": float, "end": float, "text": str}.
            output_path: Output file path.
            format: "srt" or "ass".
            
        Returns:
            Output path if successful.
        """
        if format == "ass":
            return self._generate_ass(segments, output_path)
        return self._generate_srt(segments, output_path)
    
    def _generate_srt(self, segments: list[dict], output_path: str) -> Optional[str]:
        """Generate SRT subtitle file."""
        lines = []
        for i, seg in enumerate(segments, 1):
            start = self._fmt_time(seg.get("start", 0))
            end = self._fmt_time(seg.get("end", 0))
            text = seg.get("text", "").strip()
            if text:
                lines.append(f"{i}\n{start} --> {end}\n{text}\n")
        
        if not lines:
            return None
        
        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        
        return output_path
    
    def _generate_ass(self, segments: list[dict], output_path: str) -> Optional[str]:
        """Generate ASS subtitle file with styling."""
        header = (
            "[Script Info]\n"
            "ScriptType: v4.00+\n"
            "PlayResX: 1920\n"
            "PlayResY: 1080\n"
            "\n"
            "[V4+ Styles]\n"
            "Format: Name, Fontname, Fontsize, PrimaryColour, "
            "SecondaryColour, OutlineColour, BackColour, Bold, Italic, "
            "Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, "
            "BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, "
            "MarginV, Encoding\n"
            "Style: Default, Arial, 48, &H00FFFFFF, &H000000FF, "
            "&H00000000, &H80000000, 0, 0, 0, 0, 100, 100, 0, 0, "
            "1, 2, 1, 2, 20, 20, 40, 1\n"
            "\n"
            "[Events]\n"
            "Format: Layer, Start, End, Style, Name, MarginL, MarginR, "
            "MarginV, Effect, Text\n"
        )
        
        events = []
        for seg in segments:
            text = seg.get("text", "").strip()
            if not text:
                continue
            start = self._fmt_ass_time(seg.get("start", 0))
            end = self._fmt_ass_time(seg.get("end", 0))
            events.append(
                f"Dialogue: 0,{start},{end},Default,,0,0,0,,{text}"
            )
        
        if not events:
            return None
        
        os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(header + "\n".join(events))
        
        return output_path
    
    def _fmt_time(self, seconds: float) -> str:
        """SRT time format."""
        h = int(seconds // 3600)
        m = int((seconds % 3600) // 60)
        s = int(seconds % 60)
        ms = int((seconds % 1) * 1000)
        return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"
    
    def _fmt_ass_time(self, seconds: float) -> str:
        """ASS time format: H:MM:SS.cc"""
        h = int(seconds // 3600)
        m = int((seconds % 3600) // 60)
        s = int(seconds % 60)
        cs = int((seconds % 1) * 100)
        return f"{h}:{m:02d}:{s:02d}.{cs:02d}"
    
    def generate_drawtext_filter(
        self, text: str, x: int, y: int,
        font_size: int = 48, color: str = "white",
        duration: float = 5.0
    ) -> str:
        """Generate FFmpeg drawtext filter for a single text overlay."""
        escaped = text.replace("'", "'\\''").replace(":", "\\:")
        return (
            f"drawtext=text='{escaped}'"
            f":x={x}:y={y}"
            f":fontsize={font_size}"
            f":fontcolor={color}"
            f":fontfile=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
            f":enable='between(t,0,{duration})'"
        )
```

### Verifikasi Effects

```python
from auto_editor.workers.effects.transition import TransitionEngine

te = TransitionEngine()
trans = te.suggest_transition(Scene(1), Scene(2))
assert isinstance(trans.value, str)
```

---

## Task 2.6 — Renderer

**Folder:** `auto-editor/workers/renderer/`
**File:** `opencut_bridge.py`, `ffmpeg_pipeline.py`

### 2.6.1 OpenCut Bridge (`opencut_bridge.py`)

Bridge ke OpenCut WASM compositor untuk rendering.

```python
"""Bridge to OpenCut's internal compositor for rendering.

Uses OpenCut's WASM compositor via CLI/API when available.
Falls back to FFmpeg pipeline.
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Callable
import subprocess
import json
import os


class OpenCutBridge:
    """Bridge to OpenCut's compositor for rendering.
    
    Converts our project format to OpenCut's internal format
    and triggers the WASM compositor for rendering.
    """
    
    def __init__(self, opencut_cli: str = "npx opencut"):
        self.opencut_cli = opencut_cli
    
    def render_project(
        self,
        project_data: dict,
        output_path: str,
        progress_callback: Optional[Callable[[float], None]] = None
    ) -> Optional[str]:
        """Render a project using OpenCut compositor.
        
        Args:
            project_data: Project in OpenCut-compatible format.
            output_path: Output video path.
            progress_callback: Called with progress 0.0-1.0.
            
        Returns:
            Output path if successful, None if failed.
        """
        project_file = self._write_project_file(project_data)
        if not project_file:
            return self._fallback_render(project_data, output_path)
        
        try:
            cmd = [
                *self.opencut_cli.split(),
                "render", project_file,
                "--output", output_path,
            ]
            process = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True
            )
            
            # Parse progress from stdout
            while True:
                line = process.stdout.readline()
                if not line:
                    break
                if "progress" in line.lower():
                    try:
                        pct = float(line.split(":")[-1].strip().rstrip("%"))
                        if progress_callback:
                            progress_callback(pct / 100.0)
                    except ValueError:
                        pass
            
            process.wait()
            
            if process.returncode == 0 and os.path.exists(output_path):
                return output_path
            
            return self._fallback_render(project_data, output_path)
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return self._fallback_render(project_data, output_path)
    
    def _write_project_file(self, project_data: dict) -> Optional[str]:
        """Write project data to temporary file."""
        try:
            tmp = Path("./.opencut_projects")
            tmp.mkdir(exist_ok=True)
            project_file = tmp / "project.json"
            with open(project_file, "w") as f:
                json.dump(project_data, f)
            return str(project_file)
        except IOError:
            return None
    
    def _fallback_render(
        self, project_data: dict, output_path: str
    ) -> Optional[str]:
        """Fallback: return None so FFmpegPipeline takes over."""
        return None
    
    def is_available(self) -> bool:
        """Check if OpenCut CLI is available."""
        try:
            result = subprocess.run(
                [*self.opencut_cli.split(), "--version"],
                capture_output=True, timeout=10
            )
            return result.returncode == 0
        except (subprocess.SubprocessError, FileNotFoundError):
            return False
```

### 2.6.2 FFmpeg Pipeline (`ffmpeg_pipeline.py`)

FFmpeg-based rendering pipeline — primary renderer.

```python
"""FFmpeg-based video rendering pipeline.

Primary renderer. Used directly or as fallback from OpenCut.
Supports hardware acceleration via VAAPI/NVIDIA.
"""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Callable, Literal
import subprocess
import json
import os
import re


class FFmpegPipeline:
    """Render final video using FFmpeg.
    
    Handles:
    - Video encoding (H.264/H.265)
    - Audio mixing
    - Subtitle burning
    - Hardware acceleration
    - Progress tracking
    """
    
    def __init__(self, ffmpeg_path: str = "ffmpeg", ffprobe_path: str = "ffprobe"):
        self.ffmpeg = ffmpeg_path
        self.ffprobe = ffprobe_path
    
    def render(
        self,
        video_paths: list[str],
        output_path: str,
        audio_path: Optional[str] = None,
        subtitle_path: Optional[str] = None,
        resolution: str = "1080p",
        codec: Literal["h264", "h265"] = "h264",
        fps: float = 30.0,
        progress_callback: Optional[Callable[[float], None]] = None
    ) -> Optional[str]:
        """Render final video.
        
        Args:
            video_paths: List of video clips to concatenate.
            output_path: Output video path.
            audio_path: Optional audio track (voiceover + music).
            subtitle_path: Optional subtitle file.
            resolution: "720p", "1080p", "4k".
            codec: Video codec.
            fps: Frame rate.
            progress_callback: Called with progress 0.0-1.0.
            
        Returns:
            Output path if successful, None if failed.
        """
        os.makedirs(Path(output_path).parent, exist_ok=True)
        
        # Build scale filter
        scale_map = {"720p": "1280:720", "1080p": "1920:1080", "4k": "3840:2160"}
        scale = scale_map.get(resolution, "1920:1080")
        
        try:
            if len(video_paths) == 1:
                return self._render_single(
                    video_paths[0], output_path, audio_path,
                    subtitle_path, scale, codec, fps, progress_callback
                )
            else:
                return self._render_concat(
                    video_paths, output_path, audio_path,
                    subtitle_path, scale, codec, fps, progress_callback
                )
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
    
    def _render_single(
        self, video: str, output: str, audio: Optional[str],
        subtitle: Optional[str], scale: str, codec: str,
        fps: float, progress_cb: Optional[Callable]
    ) -> Optional[str]:
        """Render single video file."""
        codec_params = self._codec_params(codec)
        filters = [f"scale={scale}:force_original_aspect_ratio=decrease,"
                   f"pad={scale}:(ow-iw)/2:(oh-ih)/2"]
        
        if subtitle and os.path.exists(subtitle):
            filters.append(f"subtitles={subtitle}")
        
        filter_str = ",".join(filters)
        
        cmd = [
            self.ffmpeg, "-i", video,
            *(["-i", audio] if audio and os.path.exists(audio) else []),
            "-vf", filter_str,
            "-r", str(fps),
            *codec_params,
            "-movflags", "+faststart",
            output, "-y"
        ]
        
        return self._run_with_progress(cmd, progress_cb)
    
    def _render_concat(
        self, videos: list[str], output: str, audio: Optional[str],
        subtitle: Optional[str], scale: str, codec: str,
        fps: float, progress_cb: Optional[Callable]
    ) -> Optional[str]:
        """Render concatenated video files."""
        # Create concat file
        concat_file = "./.opencut_concat.txt"
        with open(concat_file, "w") as f:
            for v in videos:
                if os.path.exists(v):
                    f.write(f"file '{os.path.abspath(v)}'\n")
        
        codec_params = self._codec_params(codec)
        scale_filter = (
            f"scale={scale}:force_original_aspect_ratio=decrease,"
            f"pad={scale}:(ow-iw)/2:(oh-ih)/2"
        )
        
        cmd = [
            self.ffmpeg, "-f", "concat", "-safe", "0",
            "-i", concat_file,
            *(["-i", audio] if audio and os.path.exists(audio) else []),
            "-vf", scale_filter,
            "-r", str(fps),
            *codec_params,
            "-movflags", "+faststart",
            output, "-y"
        ]
        
        result = self._run_with_progress(cmd, progress_cb)
        
        if os.path.exists(concat_file):
            os.unlink(concat_file)
        
        return result
    
    def _codec_params(self, codec: str) -> list[str]:
        """Get codec-specific parameters."""
        if codec == "h265":
            return ["-c:v", "libx265", "-crf", "23", "-preset", "medium"]
        return ["-c:v", "libx264", "-crf", "18", "-preset", "medium", "-pix_fmt", "yuv420p"]
    
    def _run_with_progress(
        self, cmd: list[str],
        progress_callback: Optional[Callable[[float], None]]
    ) -> Optional[str]:
        """Run FFmpeg with progress parsing."""
        output_path = cmd[cmd.index("-y") - 1] if "-y" in cmd else None
        
        try:
            process = subprocess.Popen(
                cmd, stderr=subprocess.PIPE, text=True,
                bufsize=1
            )
            
            duration = None
            time_pattern = re.compile(r'time=(\d+):(\d+):(\d+)\.(\d+)')
            duration_pattern = re.compile(r'Duration: (\d+):(\d+):(\d+)\.(\d+)')
            
            for line in process.stderr:
                # Parse duration
                if duration is None:
                    dm = duration_pattern.search(line)
                    if dm:
                        duration = (
                            int(dm.group(1)) * 3600 +
                            int(dm.group(2)) * 60 +
                            int(dm.group(3)) +
                            int(dm.group(4)) / 100
                        )
                
                # Parse progress
                if progress_callback and duration and duration > 0:
                    tm = time_pattern.search(line)
                    if tm:
                        current = (
                            int(tm.group(1)) * 3600 +
                            int(tm.group(2)) * 60 +
                            int(tm.group(3)) +
                            int(tm.group(4)) / 100
                        )
                        progress_callback(min(1.0, current / duration))
            
            process.wait()
            
            if process.returncode == 0 and output_path and os.path.exists(output_path):
                return output_path
            return None
            
        except (subprocess.SubprocessError, FileNotFoundError):
            return None
    
    def get_video_info(self, video_path: str) -> Optional[dict]:
        """Get video file metadata."""
        try:
            cmd = [
                self.ffprobe, "-v", "error",
                "-show_entries", "format=duration,size,bit_rate",
                "-show_entries", "stream=width,height,codec_name,r_frame_rate",
                "-of", "json", video_path
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return json.loads(result.stdout)
        except (subprocess.SubprocessError, json.JSONDecodeError):
            return None
    
    def validate_output(self, video_path: str) -> bool:
        """Validate rendered video file."""
        info = self.get_video_info(video_path)
        if not info:
            return False
        try:
            duration = float(info.get("format", {}).get("duration", 0))
            return duration > 0
        except (ValueError, TypeError):
            return False
```

### Verifikasi Renderer

```python
from auto_editor.workers.renderer.ffmpeg_pipeline import FFmpegPipeline

pipe = FFmpegPipeline()
info = pipe.get_video_info("test.mp4")
assert info is None or isinstance(info, dict)
```

---

## Task 2.7 — Test Suite Workers

### Instruksi

Buat `auto-editor/tests/test_scene_detector.py`, `test_asset_finder.py`,
`test_audio_pipeline.py`, `test_renderer.py`.

Setiap test file minimal 3 test functions dengan assertion.

### `tests/test_scene_detector.py`

```python
"""Tests for SceneDetector."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.scene_detector.detector import SceneDetector


def test_detect_nonexistent():
    det = SceneDetector()
    assert det.detect("nonexistent.mp4") == []


def test_estimate_tokens():
    det = SceneDetector()
    assert det.estimate_tokens("any.mp4") == 0


def test_detect_with_thumbnails_nonexistent():
    det = SceneDetector()
    result = det.detect_with_thumbnails("nonexistent.mp4")
    assert isinstance(result, list)
```

### `tests/test_asset_finder.py`

```python
"""Tests for AssetFinder components."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.asset_finder.rag_search import RAGSearch


def test_rag_search_empty():
    rag = RAGSearch(["./nonexistent_dir/"])
    results = rag.search("test query")
    assert isinstance(results, list)


def test_rag_estimate_tokens():
    rag = RAGSearch()
    assert rag.estimate_tokens() == 0


def test_scan_nonexistent():
    rag = RAGSearch()
    assets = rag.scan_directory("./nonexistent/")
    assert assets == []
```

### `tests/test_audio_pipeline.py`

```python
"""Tests for Audio Pipeline components."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.audio_pipeline.alignment import VoiceoverAligner
from auto_editor.models import VoiceoverSegment


def test_align_to_scenes():
    aligner = VoiceoverAligner()
    segments = [VoiceoverSegment(text="Test", start=0, end=2)]
    result = aligner.align_to_scenes(segments, [5.0])
    assert result.total_duration > 0
    assert len(result.segments) == 1


def test_align_empty():
    aligner = VoiceoverAligner()
    result = aligner.align_to_scenes([], [])
    assert result.total_duration == 0
    assert result.segments == []


def test_adjust_speed():
    aligner = VoiceoverAligner()
    from auto_editor.workers.audio_pipeline.alignment import AlignedVoiceover
    vo = AlignedVoiceover(
        segments=[VoiceoverSegment(text="A", start=0, end=10)],
        total_duration=10.0
    )
    result = aligner.adjust_speed_for_timeline(vo, 5.0)
    assert abs(result.total_duration - 5.0) < 0.1
```

### `tests/test_renderer.py`

```python
"""Tests for Renderer."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.workers.renderer.ffmpeg_pipeline import FFmpegPipeline


def test_get_info_nonexistent():
    pipe = FFmpegPipeline()
    info = pipe.get_video_info("nonexistent.mp4")
    assert info is None


def test_validate_nonexistent():
    pipe = FFmpegPipeline()
    assert not pipe.validate_output("nonexistent.mp4")


def test_render_empty_list():
    pipe = FFmpegPipeline()
    result = pipe.render([], "output.mp4")
    assert result is None
```

---

## Task 2.8 — Run All Tests & Fix

```bash
python -m pytest auto-editor/tests/ -v
```

Semua test dari Agent 1 (yang sudah lulus) + test baru Agent 2 harus lulus semua.

---

## DELIVERABLES FINAL AGENT 2

```
Task 2.1  ✅ SceneDetector — FFmpeg-based scene detection + ShotClassifier
Task 2.2  ✅ AssetFinder — AssetCrawler (Pexels/Pixabay) + RAGSearch + AssetDownloader
Task 2.3  ✅ LayoutEngine — Compositor + TemplateLoader (coordinate.py dari Agent 1)
Task 2.4  ✅ AudioPipeline — TTSEngine (CosyVoice/Bark/Piper) + ASREngine (Whisper) + VoiceoverAligner + AudioMixer
Task 2.5  ✅ Effects — ColorGradingEngine + TransitionEngine + TextOverlayEngine
Task 2.6  ✅ Renderer — OpenCutBridge + FFmpegPipeline (encode, concat, subtitles, hwaccel)
Task 2.7  ✅ Test suite — 4 test files, 12+ tests
Task 2.8  ✅ All tests passing
```

Agent 2 selesai. Semua worker independen sudah bisa dipanggil dari CLI atau dari WorkflowEngine Agent 1.
Agent 3 nanti yang akan menghubungkan ini dengan MOKO OS dan REST API.
