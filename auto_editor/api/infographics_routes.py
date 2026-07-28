"""Infographics Video API endpoints.

POST /api/infographics/script — AI-driven infographic JSON structure design.
POST /api/infographics/upload — Receives WebM canvas recording, converts to MP4.
"""

from __future__ import annotations

from pathlib import Path
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

router = APIRouter(prefix="/api/infographics")

# Outputs folders
OUTPUT_DIR = Path("./output/infographics")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

UPLOAD_DIR = Path("./output/infographics/uploads")
UPLOAD_DIR.mkdir(parents=True, exist_ok=True)

_jobs: dict[str, dict] = {}


class InfographicScriptRequest(BaseModel):
    prompt: str


def _design_infographics_script(prompt: str) -> dict:
    """Rancang struktur infografis berdasarkan prompt (Rule-based & LLM ready)."""
    p = prompt.lower()

    # Default settings
    duration = 10.0
    elements = []
    camera_keyframes = [
        {"t": 0.0, "position": [0, 4, 8.5], "lookAt": [0, 1.2, 0]},
    ]

    # Rule-based parsing
    if "laba" in p or "sales" in p or "penjualan" in p or "untung" in p:
        elements.append({
            "id": "sales_bar",
            "type": "bar_chart",
            "position": [-3.0, 0, 0],
            "start": 1.5,
            "duration": 3.5,
            "data": [
                {"label": "Q1", "value": 45, "color": "#6366f1"},
                {"label": "Q2", "value": 75, "color": "#10b981"},
                {"label": "Q3", "value": 95, "color": "#f59e0b"},
            ]
        })
        camera_keyframes.append({"t": 4.0, "position": [-3, 3, 5.5], "lookAt": [-3, 1.2, 0]})

    if "distribusi" in p or "lingkaran" in p or "pie" in p or "bagi" in p:
        elements.append({
            "id": "market_pie",
            "type": "pie_chart",
            "position": [3.0, 0.5, 0],
            "start": 5.5,
            "duration": 4.0,
            "data": [
                {"label": "Produk A", "value": 55, "color": "#6366f1"},
                {"label": "Produk B", "value": 30, "color": "#10b981"},
                {"label": "Lainnya", "value": 15, "color": "#f59e0b"},
            ]
        })
        camera_keyframes.append({"t": 8.0, "position": [3, 3, 5.5], "lookAt": [3, 1.2, 0]})
        duration = 12.0

    if "target" in p or "cincin" in p or "ring" in p or "progress" in p:
        elements.append({
            "id": "progress_target",
            "type": "progress_ring",
            "position": [0, 0.8, 1],
            "start": 2.0,
            "duration": 3.0,
            "value": 85,
            "color": "#10b981",
        })

    # Fallback default elements if prompt has nothing recognized
    if not elements:
        elements = [
            {
                "id": "default_bar",
                "type": "bar_chart",
                "position": [-3.0, 0, 0],
                "start": 1.0,
                "duration": 4.0,
                "data": [
                    {"label": "A", "value": 50, "color": "#6366f1"},
                    {"label": "B", "value": 80, "color": "#10b981"},
                ]
            },
            {
                "id": "default_ring",
                "type": "progress_ring",
                "position": [3.0, 0.8, 0],
                "start": 5.0,
                "duration": 4.0,
                "value": 75,
                "color": "#f59e0b",
            }
        ]
        camera_keyframes.append({"t": 5.0, "position": [3, 3, 5.5], "lookAt": [3, 1.2, 0]})

    camera_keyframes.append({"t": duration, "position": [0, 5, 8.5], "lookAt": [0, 1.2, 0]})

    return {
        "duration": duration,
        "theme": {
            "background": "#f0f2f5",
            "primary": "#6366f1",
            "secondary": "#10b981",
            "accent": "#f59e0b",
            "grid_color": "#d0d0d8"
        },
        "camera": {
            "keyframes": camera_keyframes
        },
        "elements": elements
    }


def _convert_webm_to_mp4(webm_path: Path, output_path: Path, add_music: bool = False, job_id: str = "") -> bool:
    try:
        # standard optimized output H.264
        cmd = [
            "ffmpeg", "-i", str(webm_path),
            "-c:v", "libx264", "-preset", "fast", "-crf", "20",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart",
            str(output_path), "-y"
        ]

        if add_music:
            cmd = [
                "ffmpeg", "-i", str(webm_path),
                "-f", "lavfi", "-i", "sine=frequency=330:beep_factor=3:duration=60",
                "-filter_complex",
                "[0:v]scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2[v];[0:a][1:a]amix=inputs=2:duration=first:weights=1 0.1[a]",
                "-map", "[v]", "-map", "[a]",
                "-c:v", "libx264", "-preset", "fast", "-crf", "20",
                "-pix_fmt", "yuv420p", "-movflags", "+faststart",
                "-c:a", "aac", "-b:a", "128k",
                str(output_path), "-y"
            ]

        if job_id:
            _jobs[job_id]["status"] = "converting"

        result = subprocess.run(cmd, capture_output=True, timeout=180)
        if result.returncode != 0:
            return False

        if job_id:
            _jobs[job_id]["status"] = "completed"
            _jobs[job_id]["output_path"] = str(output_path)

        return True
    except Exception as e:
        logger.error("[Infographics] conversion fail: %s", e)
        return False


# ── Routes ────────────────────────────────────────────────────────────────────

@router.post("/script")
async def generate_infographics_script(request: InfographicScriptRequest):
    """Rancang timeline grafik infografis 3D dari text prompt."""
    script = _design_infographics_script(request.prompt)
    return {
        "status": "ok",
        "script": script,
    }


@router.post("/upload")
async def upload_infographics_recording(
    file: UploadFile = File(...),
    prompt: str = Form(""),
    duration_s: float = Form(0),
    add_music: bool = Form(False),
):
    """Terima WebM canvas recording, render menjadi MP4 di background thread."""
    job_id = f"info_{uuid.uuid4().hex[:10]}"
    ext = "webm" if "webm" in (file.content_type or "") else "mp4"

    webm_path = UPLOAD_DIR / f"{job_id}.{ext}"
    mp4_path = OUTPUT_DIR / f"{job_id}_final.mp4"

    content = await file.read()
    with open(webm_path, "wb") as f:
        f.write(content)

    _jobs[job_id] = {
        "job_id": job_id,
        "status": "queued",
        "progress": 0.1,
        "output_path": None,
        "created_at": time.time(),
    }

    def _run():
        success = _convert_webm_to_mp4(webm_path, mp4_path, add_music, job_id)
        if not success:
            _jobs[job_id]["status"] = "error"

    threading.Thread(target=_run, daemon=True).start()

    return {
        "job_id": job_id,
        "status": "converting",
        "download_url": f"/api/infographics/{job_id}/download",
    }


@router.get("/{job_id}")
async def get_job_status(job_id: str):
    job = _jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail="Job not found")
    return job


@router.get("/{job_id}/download")
async def download_video(job_id: str):
    job = _jobs.get(job_id)
    if not job or job["status"] != "completed":
        raise HTTPException(status_code=404, detail="File not ready or job missing")

    return FileResponse(
        path=job["output_path"],
        media_type="video/mp4",
        filename=f"infographics_{job_id}.mp4",
    )
