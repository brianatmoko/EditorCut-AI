"""Unified Auto-Video Studio API endpoints — 2D Vector Engine.

POST /api/studio/ai-chat  — Chat with AI Director, returns a 2D animation script.
POST /api/studio/ai-story — AI multi-scene storyboard generator.
POST /api/studio/script   — AI-driven unified 2D script structure design.
POST /api/studio/upload   — Receives WebM canvas recording, converts to MP4.
"""

from __future__ import annotations

from pathlib import Path
import json
import re
import uuid
import time
import logging
import subprocess
import threading
from typing import Optional

from fastapi import APIRouter, HTTPException, UploadFile, File, Form
from fastapi.responses import FileResponse
from pydantic import BaseModel

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/api/studio")

# Outputs folders
OUTPUT_DIR = Path("./output/studio")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

UPLOAD_DIR = Path("./output/studio/uploads")
UPLOAD_DIR.mkdir(parents=True, exist_ok=True)

_jobs: dict[str, dict] = {}

# ── 2D Canvas System Prompt ────────────────────────────────────────────────────

_2D_SYSTEM_PROMPT = """You are a 2D Vector Animation Director AI for OpenCut Studio.

OpenCut Studio uses a **Two.js 2D Vector Engine** — all scenes are rendered in a 2D canvas.
Charts, characters, and all elements are positioned in 2D canvas pixel-space coordinates, NOT 3D world space.

== 2D CANVAS LAYOUT ==
The canvas is ~800×600 pixels (viewport). The scene center is at (0,0) in relative units.
Use these relative canvas position values for element placement (they will be scaled to pixels):
  - Left side area: x = -3 to -1, y = 0 to 1
  - Center: x = 0, y = 0 to 1
  - Right side: x = 1 to 3, y = 0 to 1

== ENVIRONMENTS (theme) ==
- "city" — urban city skyline, bus stops, neon billboards, vector cars (day/night/sunset)
- "cyberpunk" — neon night city with glowing skyscrapers
- "room" — cozy indoor room with sofa, TV, bookshelf, potted plants
- "school" — classroom setting
- "space" — outer space with planets and stars
- "desert" — golden desert landscape
- "forest" — green woodland with mountains
- "ocean" — ocean environment
- "arctic" — snowy frozen landscape
- "volcano" — volcanic terrain
- "studio" — professional studio with parked cars and barrier gate

== TIME OF DAY ==
"noon", "sunset", "night"

== CHARACTER TYPES ==
"stickman" | "soldier" | "robot" | "michelle"

== CHARACTER ACTIONS ==
"idle", "walk", "run", "jump", "dance", "wave", "think", "punch"

== INFOGRAPHIC ELEMENT TYPES ==
"bar_chart" — vertical bars with labels and values
"pie_chart" — segmented pie/donut chart
"progress_ring" — animated circular progress ring

== CAMERA MODES ==
"cinematic" — AI auto-directs camera pan and zoom
"follow" — camera follows the character
"free" — fixed wide view

Return ONLY valid JSON — no explanation, no markdown code fences."""

_AI_CHAT_SYSTEM = _2D_SYSTEM_PROMPT + """

For the /ai-chat endpoint, return ONLY this JSON shape:
{
  "reply": "Short friendly reply to the user in the same language they wrote.",
  "script": {
    "duration": 12.0,
    "characterType": "robot",
    "environment": { "theme": "city", "timeOfDay": "noon" },
    "cameraMode": "cinematic",
    "cameraTarget": "none",
    "characters": [
      {
        "id": "hero",
        "timeline": [
          { "t": 0.0, "action": "idle", "duration": 1.5 },
          { "t": 1.5, "action": "walk", "duration": 4.0, "direction": "right" },
          { "t": 5.5, "action": "wave", "duration": 2.0 },
          { "t": 7.5, "action": "idle", "duration": 4.5 }
        ]
      }
    ],
    "elements": [
      {
        "id": "sales_bar",
        "type": "bar_chart",
        "position": [-2.5, 0.5, 0],
        "start": 2.0,
        "duration": 8.0,
        "data": [
          { "label": "Q1", "value": 70, "color": "#6366f1" },
          { "label": "Q2", "value": 90, "color": "#10b981" },
          { "label": "Q3", "value": 110, "color": "#f59e0b" }
        ]
      }
    ]
  }
}"""

_AI_STORY_SYSTEM = _2D_SYSTEM_PROMPT + """

For the /ai-story endpoint, generate a 3-scene storyboard.
Return ONLY valid JSON matching this exact schema:
{
  "title": "Story Title",
  "scenes": [
    {
      "id": 1,
      "title": "Scene 1: Opening",
      "duration": 9.0,
      "narration": "Narration text spoken aloud.",
      "characterType": "robot",
      "environment": { "theme": "city", "timeOfDay": "noon" },
      "cameraTarget": "none",
      "cameraKeyframes": [
        { "t": 0.0, "position": [0, 4, 9], "lookAt": [0, 1.5, 0] },
        { "t": 9.0, "position": [0, 3, 6], "lookAt": [0, 1.2, 0] }
      ],
      "actions": [
        { "t": 0.0, "action": "idle", "duration": 1.5 },
        { "t": 1.5, "action": "walk", "duration": 5.0, "direction": "right" },
        { "t": 6.5, "action": "wave", "duration": 2.5 }
      ],
      "elements": [
        {
          "id": "intro_ring",
          "type": "progress_ring",
          "position": [0, 0.8, 0],
          "start": 1.0,
          "duration": 7.0,
          "value": 85,
          "color": "#6366f1"
        }
      ]
    }
  ]
}"""


class UnifiedScriptRequest(BaseModel):
    prompt: str
    duration: float = 12.0


class StoryRequest(BaseModel):
    prompt: str


class ChatRequest(BaseModel):
    prompt: str


# ── MOKOClient (AI Gateway) ────────────────────────────────────────────────────

from moko_bridge.moko_client import MOKOClient

_moko_client: Optional[MOKOClient] = None


def get_moko_client() -> MOKOClient:
    global _moko_client
    if _moko_client is None:
        _moko_client = MOKOClient()
    return _moko_client


# ── Smart Fallback Script Builder (rule-based, 2D-native) ──────────────────────

def _detect_theme(p: str) -> str:
    if any(w in p for w in ["ruang", "kamar", "room", "dalam"]):
        return "room"
    if any(w in p for w in ["sekolah", "kelas", "school"]):
        return "school"
    if any(w in p for w in ["angkasa", "planet", "space", "luar angkasa"]):
        return "space"
    if any(w in p for w in ["gurun", "desert", "pasir"]):
        return "desert"
    if any(w in p for w in ["hutan", "forest", "pohon"]):
        return "forest"
    if any(w in p for w in ["laut", "ocean", "bawah laut"]):
        return "ocean"
    if any(w in p for w in ["cyberpunk", "neon", "cyber", "futuristik"]):
        return "cyberpunk"
    if any(w in p for w in ["salju", "arctic", "kutub", "es"]):
        return "arctic"
    if any(w in p for w in ["lahar", "volcano", "gunung api", "api"]):
        return "volcano"
    if any(w in p for w in ["studio", "syuting", "parkir", "parking"]):
        return "studio"
    return "city"


def _detect_char(p: str) -> str:
    if any(w in p for w in ["soldier", "tentara", "militer"]):
        return "soldier"
    if any(w in p for w in ["michelle", "wanita", "penari", "dancer"]):
        return "michelle"
    if any(w in p for w in ["stickman"]):
        return "stickman"
    return "robot"


def _detect_time(p: str) -> str:
    if any(w in p for w in ["malam", "night", "gelap"]):
        return "night"
    if any(w in p for w in ["sore", "sunset", "petang"]):
        return "sunset"
    return "noon"


def _build_stickman_timeline(p: str) -> list:
    timeline = [{"t": 0.0, "action": "idle", "duration": 1.0}]
    t = 1.0

    def add(action, dur, direction=None, speed=1.0):
        nonlocal t
        clip = {"t": round(t, 2), "action": action, "duration": round(dur, 2)}
        if direction:
            clip["direction"] = direction
        if speed != 1.0:
            clip["speed"] = speed
        timeline.append(clip)
        t += dur

    has_walk = any(w in p for w in ["jalan", "walk", "berjalan", "pergi"])
    has_run = any(w in p for w in ["lari", "run", "berlari"])
    has_jump = any(w in p for w in ["lompat", "jump", "melompat"])
    has_dance = any(w in p for w in ["dance", "menari", "tari", "goyang"])
    has_wave = any(w in p for w in ["wave", "lambaikan", "halo", "dadah", "sapa"])
    has_think = any(w in p for w in ["pikir", "think", "berpikir", "analisis"])
    has_punch = any(w in p for w in ["pukul", "punch", "tinju", "fight", "lawan"])

    if has_run:
        add("run", 3.5, "right", 1.4)
    if has_walk:
        add("walk", 3.0, "right")
    if has_think:
        add("think", 3.0)
    if has_jump:
        add("jump", 1.2)
        add("idle", 0.3)
    if has_dance:
        add("dance", 4.5)
    if has_wave:
        add("wave", 2.0)
    if has_punch:
        add("punch", 0.6)
        add("idle", 0.2)
        add("punch", 0.6)

    if not (has_walk or has_run or has_jump or has_dance or has_wave or has_think or has_punch):
        add("walk", 3.0, "right")
        add("think", 2.0)
        add("wave", 2.0)

    add("idle", 1.5)
    return timeline


def _build_elements(p: str) -> list:
    elements = []
    has_bar = any(w in p for w in ["laba", "sales", "penjualan", "untung", "bar", "batang", "data", "grafik"])
    has_pie = any(w in p for w in ["distribusi", "lingkaran", "pie", "bagi", "share", "porsi"])
    has_ring = any(w in p for w in ["target", "cincin", "ring", "progress", "persen", "capaian"])

    if has_bar or (not has_pie and not has_ring):
        elements.append({
            "id": "sales_bar",
            "type": "bar_chart",
            "position": [-2.8, 0.5, 0],
            "start": 1.5,
            "duration": 6.0,
            "data": [
                {"label": "Q1", "value": 65, "color": "#6366f1"},
                {"label": "Q2", "value": 88, "color": "#10b981"},
                {"label": "Q3", "value": 110, "color": "#f59e0b"},
            ]
        })
    if has_pie:
        elements.append({
            "id": "market_pie",
            "type": "pie_chart",
            "position": [2.8, 0.5, 0],
            "start": 4.0,
            "duration": 6.0,
            "data": [
                {"label": "A", "value": 60, "color": "#6366f1"},
                {"label": "B", "value": 40, "color": "#10b981"},
            ]
        })
    if has_ring:
        elements.append({
            "id": "progress_target",
            "type": "progress_ring",
            "position": [0, 1.2, 0],
            "start": 2.0,
            "duration": 5.0,
            "value": 88,
            "color": "#10b981",
        })
    return elements


def _build_fallback_chat_response(prompt: str) -> dict:
    p = prompt.lower()
    theme = _detect_theme(p)
    char = _detect_char(p)
    tod = _detect_time(p)
    timeline = _build_stickman_timeline(p)
    elements = _build_elements(p)
    duration = max(10.0, sum(c["duration"] for c in timeline) + 2.0)

    friendly_replies = {
        "walk": "Baik! Karakter akan berjalan melintasi panggung dengan latar yang keren.",
        "dance": "Siap! Karakter akan menari dengan semangat di panggung 2D.",
        "default": f"Animasi telah dibuat! Karakter {char} akan tampil di latar {theme}.",
    }
    p_lower = prompt.lower()
    reply = friendly_replies.get(
        "dance" if "tari" in p_lower or "dance" in p_lower else
        "walk" if "jalan" in p_lower or "walk" in p_lower else "default",
        friendly_replies["default"]
    )

    return {
        "reply": reply,
        "script": {
            "duration": duration,
            "characterType": char,
            "environment": {"theme": theme, "timeOfDay": tod},
            "cameraMode": "cinematic",
            "cameraTarget": "none",
            "characters": [{"id": "hero", "timeline": timeline}],
            "elements": elements,
        }
    }


def _generate_chat_via_llm(prompt: str) -> dict:
    """Use LLM to generate 2D animation script from chat prompt."""
    client = get_moko_client()
    res = client.llm_generate(
        prompt=f"User request (in their language): {prompt}\n\nGenerate a 2D animation script for this.",
        system_prompt=_AI_CHAT_SYSTEM,
        max_tokens=2000,
        temperature=0.7,
    )

    if res and res.get("content"):
        try:
            raw_text = res["content"]
            match = re.search(r'\{.*\}', raw_text, re.DOTALL)
            if match:
                parsed = json.loads(match.group(0))
                if "script" in parsed and "reply" in parsed:
                    return parsed
        except Exception as e:
            logger.warning("[Studio] Failed to parse LLM chat JSON: %s", e)

    return _build_fallback_chat_response(prompt)


def _generate_storyboard_via_llm(prompt: str) -> dict:
    """Generate a multi-scene 2D storyboard sequence using LLM or smart fallback."""
    client = get_moko_client()

    res = client.llm_generate(
        prompt=f"Create a 3-scene 2D vector animation storyboard about: {prompt}",
        system_prompt=_AI_STORY_SYSTEM,
        max_tokens=2500,
        temperature=0.7,
    )

    if res and res.get("content"):
        try:
            raw_text = res["content"]
            match = re.search(r'\{.*\}', raw_text, re.DOTALL)
            if match:
                parsed = json.loads(match.group(0))
                if "scenes" in parsed and len(parsed["scenes"]) > 0:
                    return parsed
        except Exception as e:
            logger.warning("[Studio] Failed to parse LLM storyboard JSON: %s", e)

    # Fallback multi-scene storyboard
    p = prompt.lower()
    theme = _detect_theme(p)
    char = _detect_char(p)

    return {
        "title": prompt.title(),
        "scenes": [
            {
                "id": 1,
                "title": "Adegan 1: Pembuka",
                "duration": 9.0,
                "narration": f"Selamat datang di animasi 2D tentang {prompt}.",
                "characterType": char,
                "environment": {"theme": theme, "timeOfDay": "noon"},
                "cameraTarget": "none",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [0, 5, 12], "lookAt": [0, 1.2, 0]},
                    {"t": 9.0, "position": [0, 3, 7], "lookAt": [0, 1.0, 0]}
                ],
                "actions": [
                    {"t": 0.0, "action": "idle", "duration": 1.5},
                    {"t": 1.5, "action": "walk", "duration": 4.5, "direction": "right"},
                    {"t": 6.0, "action": "wave", "duration": 3.0}
                ],
                "elements": [
                    {
                        "id": "intro_ring",
                        "type": "progress_ring",
                        "position": [0, 1.2, 0],
                        "start": 1.0,
                        "duration": 7.0,
                        "value": 85,
                        "color": "#6366f1"
                    }
                ]
            },
            {
                "id": 2,
                "title": "Adegan 2: Data & Analisis",
                "duration": 10.0,
                "narration": "Berikut adalah data performa dan pertumbuhan utama.",
                "characterType": char,
                "environment": {"theme": theme, "timeOfDay": "sunset"},
                "cameraTarget": "hero",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [-3, 4, 8], "lookAt": [-3, 1.2, 0]},
                    {"t": 10.0, "position": [3, 4, 8], "lookAt": [3, 1.2, 0]}
                ],
                "actions": [
                    {"t": 0.0, "action": "think", "duration": 4.0},
                    {"t": 4.0, "action": "dance", "duration": 6.0}
                ],
                "elements": [
                    {
                        "id": "perf_bar",
                        "type": "bar_chart",
                        "position": [-2.5, 0.5, 0],
                        "start": 1.0,
                        "duration": 8.0,
                        "data": [
                            {"label": "Pasar A", "value": 60, "color": "#10b981"},
                            {"label": "Pasar B", "value": 90, "color": "#f59e0b"},
                            {"label": "Pasar C", "value": 110, "color": "#6366f1"}
                        ]
                    }
                ]
            },
            {
                "id": 3,
                "title": "Adegan 3: Penutup Sinematik",
                "duration": 8.0,
                "narration": "Terima kasih telah menyaksikan presentasi animasi 2D vektor ini.",
                "characterType": char,
                "environment": {"theme": theme, "timeOfDay": "night"},
                "cameraTarget": "none",
                "cameraKeyframes": [
                    {"t": 0.0, "position": [0, 4, 6], "lookAt": [0, 1.2, 0]},
                    {"t": 8.0, "position": [0, 6, 15], "lookAt": [0, 1.0, 0]}
                ],
                "actions": [
                    {"t": 0.0, "action": "wave", "duration": 3.0},
                    {"t": 3.0, "action": "idle", "duration": 5.0}
                ],
                "elements": [
                    {
                        "id": "pie_share",
                        "type": "pie_chart",
                        "position": [2.5, 0.8, 0],
                        "start": 1.0,
                        "duration": 6.0,
                        "data": [
                            {"label": "Target", "value": 75, "color": "#10b981"},
                            {"label": "Sisa", "value": 25, "color": "#ef4444"}
                        ]
                    }
                ]
            }
        ]
    }


def _convert_webm_to_mp4(webm_path: Path, output_path: Path, add_music: bool = False, job_id: str = "") -> bool:
    """Konversi video WebM hasil canvas recorder ke MP4 menggunakan FFmpeg."""
    try:
        cmd = [
            "ffmpeg", "-i", str(webm_path),
            "-c:v", "libx264", "-preset", "fast", "-crf", "20",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart",
            str(output_path), "-y"
        ]

        if add_music:
            cmd = [
                "ffmpeg", "-i", str(webm_path),
                "-f", "lavfi", "-i", "sine=frequency=350:beep_factor=3:duration=60",
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
        logger.error("[Studio] conversion fail: %s", e)
        return False


# ── Routes ─────────────────────────────────────────────────────────────────────

@router.post("/ai-chat")
async def studio_ai_chat(request: ChatRequest):
    """AI Director chat — returns a 2D animation script from a user prompt."""
    result = _generate_chat_via_llm(request.prompt)
    return {
        "status": "ok",
        "reply": result.get("reply", "Animasi telah dibuat!"),
        "script": result.get("script", {}),
    }


@router.post("/ai-story")
async def generate_ai_storyboard(request: StoryRequest):
    """Generate multi-scene 2D storyboard JSON for Studio Sequencer."""
    storyboard = _generate_storyboard_via_llm(request.prompt)
    return {
        "status": "ok",
        "storyboard": storyboard,
    }


@router.post("/script")
async def generate_studio_script(request: UnifiedScriptRequest):
    """Rancang timeline gabungan 2D (karakter + charts) dari prompt."""
    p = request.prompt.lower()
    theme = _detect_theme(p)
    char = _detect_char(p)
    tod = _detect_time(p)
    timeline = _build_stickman_timeline(p)
    elements = _build_elements(p)
    duration = max(request.duration, sum(c["duration"] for c in timeline) + 2.0)

    return {
        "status": "ok",
        "script": {
            "duration": duration,
            "characterType": char,
            "environment": {"theme": theme, "timeOfDay": tod},
            "cameraMode": "cinematic",
            "cameraTarget": "none",
            "characters": [{"id": "hero", "timeline": timeline}],
            "elements": elements,
        }
    }


@router.post("/upload")
async def upload_studio_recording(
    file: UploadFile = File(...),
    prompt: str = Form(""),
    duration_s: float = Form(0),
    add_music: bool = Form(False),
):
    """Terima WebM recording, jalankan konversi ke MP4 di background thread."""
    job_id = f"std_{uuid.uuid4().hex[:10]}"
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
        "download_url": f"/api/studio/{job_id}/download",
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
        filename=f"studio_{job_id}.mp4",
    )


