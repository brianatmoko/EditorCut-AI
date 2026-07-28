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
        cache_key = hashlib.md5(url.encode()).hexdigest()

        if output_path:
            dest = Path(output_path)
        elif filename:
            dest = self.cache_dir / filename
        else:
            ext = self._guess_extension(url)
            dest = self.cache_dir / f"{cache_key}{ext}"

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
        results = []
        for i, url in enumerate(urls):
            if progress_callback:
                progress_callback(i, len(urls))
            results.append(self.download(url))
        return results

    def _guess_extension(self, url: str) -> str:
        path = url.split("?")[0]
        _, ext = os.path.splitext(path)
        if ext:
            return ext
        return ".mp4"

    def clear_cache(self, max_age_hours: int = 24) -> int:
        import time
        now = time.time()
        deleted = 0
        for f in self.cache_dir.iterdir():
            if f.is_file() and (now - f.stat().st_mtime) > max_age_hours * 3600:
                f.unlink()
                deleted += 1
        return deleted

    def get_cache_size(self) -> int:
        return sum(f.stat().st_size for f in self.cache_dir.rglob("*") if f.is_file())
