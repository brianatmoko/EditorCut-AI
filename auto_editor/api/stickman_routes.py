"""Stickman Video API endpoints.

POST /api/stickman/script  — Generate stickman animation script dari prompt
POST /api/stickman/upload  — Terima WebM dari browser, konversi ke MP4
GET  /api/stickman/{id}    — Status konversi
GET  /api/stickman/{id}/download — Download MP4 final
"""

from __future__ import annotations

from pathlib import Path
from typing import Optional
import json
import uuid
import time
import logging
import subprocess
import threading

from fastapi import APIRouter, HTTPException, UploadFile, File, Form
from fastapi.responses import FileResponse
from pydantic import BaseModel

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/stickman")

# ── Job store ─────────────────────────────────────────────────────────────────

_jobs: dict[str, dict] = {}
OUTPUT_DIR = Path("./output/stickman")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

UPLOAD_DIR = Path("./output/stickman/uploads")
UPLOAD_DIR.mkdir(parents=True, exist_ok=True)


# ── Models ────────────────────────────────────────────────────────────────────

class StickmanScriptRequest(BaseModel):
    prompt: str
    duration: float = 10.0
    character_count: int = 1


# ── Script Generator ──────────────────────────────────────────────────────────

def _generate_script_from_prompt(prompt: str, duration: float) -> dict:
    """Generate stickman animation script dari text prompt.

    Menggunakan rule-based parser sebagai default.
    Bisa diganti dengan LLM call bila AI gateway tersedia.
    """
    p = prompt.lower()

    timeline = []
    t = 0.0

    def add(action: str, dur: float, direction: Optional[str] = None, speed: float = 1.0):
        nonlocal t
        clip = {"t": round(t, 2), "action": action, "duration": round(dur, 2)}
        if direction:
            clip["direction"] = direction
        if speed != 1.0:
            clip["speed"] = speed
        timeline.append(clip)
        t += dur

    # Selalu mulai dengan idle singkat
    add("idle", 0.8)

    # Parse kata kunci dalam prompt
    if "jalan" in p or "walk" in p or "berjalan" in p:
        dir_ = "left" if "kiri" in p else "right"
        add("walk", 3.0, dir_)

    if "lari" in p or "run" in p or "berlari" in p:
        dir_ = "left" if "kiri" in p else "right"
        add("run", 3.0, dir_, speed=1.5)

    if "lompat" in p or "jump" in p or "melompat" in p:
        reps = 2 if ("dua" in p or "2" in p or "berkali" in p) else 1
        for _ in range(reps):
            add("jump", 1.2)
            add("idle", 0.3)

    if "dance" in p or "menari" in p or "tari" in p or "goyang" in p:
        add("dance", 4.0)

    if "wave" in p or "lambaikan" in p or "halo" in p or "dadah" in p:
        add("wave", 2.0)

    if "pikir" in p or "think" in p or "berpikir" in p:
        add("think", 3.0)

    if "pukul" in p or "punch" in p or "tinju" in p or "fight" in p:
        reps = 3 if "berkali" in p else 2
        for _ in range(reps):
            add("punch", 0.6)
            add("idle", 0.2)

    # Tambah idle di akhir
    add("idle", 1.0)

    # Sesuaikan total durasi
    actual_dur = t
    if actual_dur < duration and len(timeline) > 0:
        last = timeline[-1]
        last["duration"] += duration - actual_dur

    return {
        "duration": max(actual_dur, duration),
        "characters": [
            {
                "id": "hero",
                "color": "#1a1a1a",
                "position": [0, 0, 0],
                "timeline": timeline,
            }
        ],
        "camera": {
            "mode": "follow",
            "shots": [
                {"t": 0.0, "type": "wide", "pos": [0, 2.2, 5.5]},
            ],
        },
        "environment": {
            "background": "#f0f2f5",
            "floor_color": "#e8e8e8",
            "lighting": "day",
        },
    }


# ── WebM → MP4 Converter ──────────────────────────────────────────────────────

def _convert_webm_to_mp4(
    webm_path: Path,
    output_path: Path,
    add_music: bool = False,
    job_id: str = "",
) -> bool:
    """Konversi WebM dari browser → MP4 H.264 menggunakan FFmpeg."""
    try:
        cmd = [
            "ffmpeg",
            "-i", str(webm_path),
            "-c:v", "libx264",
            "-preset", "fast",
            "-crf", "20",
            "-pix_fmt", "yuv420p",   # Kompatibel dengan semua player
            "-movflags", "+faststart",  # Streaming-ready
            "-c:a", "aac",
            "-b:a", "128k",
            str(output_path),
            "-y",
        ]

        if add_music:
            # Tambah background music loop dengan volume rendah
            # (Gunakan procedural audio dari FFmpeg)
            cmd = [
                "ffmpeg",
                "-i", str(webm_path),
                "-f", "lavfi",
                "-i", "sine=frequency=440:beep_factor=4:duration=60",
                "-filter_complex",
                "[0:v]scale=1920:1080:force_original_aspect_ratio=decrease,"
                "pad=1920:1080:(ow-iw)/2:(oh-ih)/2[v];"
                "[0:a][1:a]amix=inputs=2:duration=first:weights=1 0.1[a]",
                "-map", "[v]",
                "-map", "[a]",
                "-c:v", "libx264", "-preset", "fast", "-crf", "20",
                "-pix_fmt", "yuv420p",
                "-movflags", "+faststart",
                "-c:a", "aac", "-b:a", "128k",
                str(output_path), "-y",
            ]

        if job_id:
            _jobs[job_id]["status"] = "converting"
            _jobs[job_id]["progress"] = 0.4

        result = subprocess.run(cmd, capture_output=True, timeout=180)

        if result.returncode != 0:
            logger.error("[Stickman] FFmpeg error: %s", result.stderr.decode()[-300:])
            return False

        if job_id:
            _jobs[job_id]["status"] = "completed"
            _jobs[job_id]["progress"] = 1.0
            _jobs[job_id]["output_path"] = str(output_path)

        logger.info("[Stickman] Converted: %s → %s", webm_path.name, output_path.name)
        return True

    except subprocess.TimeoutExpired:
        logger.error("[Stickman] FFmpeg timeout for %s", webm_path.name)
        return False
    except Exception as e:
        logger.error("[Stickman] Conversion error: %s", e)
        return False


# ── API Routes ────────────────────────────────────────────────────────────────

@router.post("/script")
async def generate_script(request: StickmanScriptRequest):
    """Generate stickman animation script dari text prompt.

    Mengembalikan JSON yang bisa langsung diparsing oleh frontend.
    """
    if not request.prompt.strip():
        raise HTTPException(status_code=400, detail="Prompt tidak boleh kosong")

    script = _generate_script_from_prompt(request.prompt, request.duration)

    return {
        "status": "ok",
        "prompt": request.prompt,
        "duration": script["duration"],
        "clip_count": len(script["characters"][0]["timeline"]),
        "script": script,
    }


@router.post("/upload")
async def upload_recording(
    file: UploadFile = File(...),
    prompt: str = Form(""),
    duration_s: float = Form(0),
    add_music: bool = Form(False),
):
    """Terima WebM recording dari browser, konversi ke MP4.

    Upload file WebM yang direkam CanvasRecorder.
    Background thread melakukan FFmpeg conversion.
    """
    job_id = f"stk_{uuid.uuid4().hex[:10]}"

    # Simpan WebM upload
    ext = "webm" if "webm" in (file.content_type or "") else "mp4"
    webm_path = UPLOAD_DIR / f"{job_id}.{ext}"
    mp4_path  = OUTPUT_DIR / f"{job_id}_final.mp4"

    content = await file.read()
    with open(webm_path, "wb") as f:
        f.write(content)

    logger.info("[Stickman] Received upload: %s (%.1f MB)", file.filename, len(content) / 1024 / 1024)

    # Init job
    _jobs[job_id] = {
        "job_id": job_id,
        "status": "queued",
        "progress": 0.1,
        "prompt": prompt,
        "duration_s": duration_s,
        "webm_path": str(webm_path),
        "output_path": None,
        "created_at": time.time(),
        "error": None,
    }

    # Convert di background thread
    def _run():
        try:
            _jobs[job_id]["status"] = "converting"
            _jobs[job_id]["progress"] = 0.3
            success = _convert_webm_to_mp4(webm_path, mp4_path, add_music, job_id)
            if not success:
                _jobs[job_id]["status"] = "error"
                _jobs[job_id]["error"] = "FFmpeg conversion failed"
        except Exception as e:
            _jobs[job_id]["status"] = "error"
            _jobs[job_id]["error"] = str(e)
            logger.exception("[Stickman] Job %s failed", job_id)

    thread = threading.Thread(target=_run, daemon=True)
    thread.start()

    return {
        "job_id": job_id,
        "status": "converting",
        "message": "Konversi dimulai. Cek status di /api/stickman/{job_id}",
        "download_url": f"/api/stickman/{job_id}/download",
    }


@router.get("/{job_id}")
async def get_job_status(job_id: str):
    """Cek status konversi stickman video."""
    job = _jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Job {job_id} not found")
    return job


@router.get("/{job_id}/download")
async def download_video(job_id: str):
    """Download MP4 stickman yang sudah dirender."""
    job = _jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Job {job_id} not found")

    if job["status"] != "completed":
        raise HTTPException(
            status_code=202,
            detail=f"Video belum siap. Status: {job['status']} ({int(job['progress']*100)}%)",
        )

    output_path = job.get("output_path")
    if not output_path or not Path(output_path).exists():
        raise HTTPException(status_code=404, detail="File tidak ditemukan")

    return FileResponse(
        path=output_path,
        media_type="video/mp4",
        filename=f"stickman_{job_id}.mp4",
    )
