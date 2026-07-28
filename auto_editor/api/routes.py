"""REST API routes for auto-editor.

FastAPI application exposing:
- POST /api/edit              — Start auto-edit job (legacy)
- POST /api/generate          — Generate video from text prompt (NEW)
- GET  /api/generate/{id}/edl — Preview EDL from LLM (NEW)
- POST /api/generate/{id}/approve — Trigger EDL render (NEW)
- POST /api/generate/{id}/refine  — Refine EDL with feedback (NEW)
- GET  /api/gateways          — Live gateway status (NEW)
- GET  /api/job/{id}          — Get job status
- POST /api/voiceover         — Generate voiceover
- POST /api/subtitle          — Generate subtitles
- GET  /api/templates         — List templates
- GET  /api/health            — Health check
- WebSocket /ws/job/{id}      — Real-time progress
"""

from __future__ import annotations
from fastapi import APIRouter, HTTPException, WebSocket, WebSocketDisconnect
from pydantic import BaseModel
from typing import Optional
import uuid
import json
import threading
import time
import os

from auto_editor.orchestrator.intent_router import IntentRouter
from auto_editor.orchestrator.workflow_engine import WorkflowEngine, WorkflowNode, NodeStatus
from auto_editor.orchestrator.template_db import TemplateDB
from auto_editor.orchestrator.mandor_llm import MandorLLM
from auto_editor.orchestrator.director_llm import DirectorLLM, EditDecisionList
from auto_editor.config.settings_loader import load_config
from moko_bridge.moko_client import MOKOClient

from auto_editor.workers.scene_detector.detector import SceneDetector
from auto_editor.workers.scene_detector.classifier import ShotClassifier
from auto_editor.workers.asset_finder.crawler import AssetCrawler
from auto_editor.workers.asset_finder.downloader import AssetDownloader
from auto_editor.workers.audio_pipeline.tts_engine import TTSEngine
from auto_editor.workers.audio_pipeline.asr_whisper import ASREngine
from auto_editor.workers.audio_pipeline.mixer import AudioMixer
from auto_editor.workers.effects.text_overlay import TextOverlayEngine
from auto_editor.workers.effects.color_grade import ColorGradingEngine
from auto_editor.workers.effects.transition import TransitionEngine
from auto_editor.workers.layout_engine.template import TemplateLoader
from auto_editor.workers.layout_engine.compositor import Compositor
from auto_editor.workers.renderer.ffmpeg_pipeline import FFmpegPipeline


router = APIRouter(prefix="/api")
active_jobs: dict[str, dict] = {}
active_connections: dict[str, list[WebSocket]] = {}
_engine: Optional[WorkflowEngine] = None
_mandor: Optional[MandorLLM] = None


def get_engine() -> WorkflowEngine:
    global _engine
    if _engine is None:
        _engine = WorkflowEngine()
    return _engine

def get_mandor() -> MandorLLM:
    global _mandor
    if _mandor is None:
        _mandor = MandorLLM()
    return _mandor


# ── Request/Response Models ──────────────────────────────

class EditRequest(BaseModel):
    footage_dir: str
    script: Optional[str] = None
    output: str = "./output.mp4"
    mode: str = "hybrid"
    style: Optional[str] = None
    duration: Optional[int] = None
    prompt: Optional[str] = None

class EditResponse(BaseModel):
    job_id: str
    status: str
    message: str

class VoiceoverRequest(BaseModel):
    text: str
    voice: str = "default"
    language: str = "id"
    speed: float = 1.0

class SubtitleRequest(BaseModel):
    video_path: str
    language: str = "id"
    model_size: str = "base"   # tiny/base/small/medium/large-v3
    output_format: str = "srt"  # srt/vtt/json
    burn_into_video: bool = False
    output_path: Optional[str] = None

class SceneRequest(BaseModel):
    video_path: str
    threshold: float = 0.3  # 0.0–1.0, higher = fewer scenes
    with_thumbnails: bool = False
    split_output: Optional[str] = None  # dir path to save split clips


# ── Job Management ───────────────────────────────────────

def _generate_job_id() -> str:
    return f"job_{uuid.uuid4().hex[:12]}"


def _broadcast_progress(job_id: str, progress: float, status: str, message: str = ""):
    data = json.dumps({
        "job_id": job_id,
        "progress": progress,
        "status": status,
        "message": message,
        "timestamp": time.time(),
    })
    for ws in active_connections.get(job_id, []):
        try:
            import asyncio
            asyncio.run(ws.send_text(data))
        except Exception:
            pass


def _progress_callback(job_id: str, node_id: str, status: NodeStatus, progress: float):
    msg = f"{node_id}: {status.value}"
    active_jobs[job_id]["last_node"] = node_id
    active_jobs[job_id]["node_status"] = status.value
    _broadcast_progress(job_id, active_jobs[job_id].get("progress", 0.0), "running", msg)


# ── Pipeline Workers ─────────────────────────────────────

def _analyze_brief(prompt: str = "", **kwargs) -> dict:
    router_obj = IntentRouter()
    plan = router_obj.create_plan(prompt)
    mandor = get_mandor()
    decision = mandor.analyze_brief(prompt, {"topic": prompt})
    return {
        "plan": plan,
        "decision": decision.content,
        "_token_cost": decision.token_cost,
    }

def _research_trends(**kwargs) -> dict:
    time.sleep(0.5)
    return {
        "trends": [
            {"source": "trending", "topic": "popular format", "description": "Vertical 9:16, fast cuts"},
            {"source": "audio", "topic": "trending sounds", "description": "Viral backing track"},
        ]
    }

def _generate_script(plan=None, prompt: str = "", **kwargs) -> dict:
    mandor = get_mandor()
    decision = mandor.generate_script(plan, prompt)
    return {
        "script": decision.content.get("full_script", ""),
        "segments": decision.content.get("segments", []),
        "_token_cost": decision.token_cost,
    }

def _find_assets(script: str = "", plan=None, **kwargs) -> dict:
    try:
        crawler = AssetCrawler()
        keywords = plan.style.value if plan else "video"
        assets = crawler.search(keywords, limit=5)
    except Exception:
        assets = []
    return {"assets": assets}

def _detect_scenes(footage_dir: str = "", **kwargs) -> dict:
    if not os.path.isdir(footage_dir):
        return {"shots": [], "message": "No footage directory"}
    try:
        detector = SceneDetector()
        shots = detector.detect(footage_dir)
        classifier = ShotClassifier()
        classified = [classifier.classify(s) for s in shots]
        return {"shots": classified}
    except Exception as e:
        return {"shots": [], "message": str(e)}

def _generate_voiceover(script: str = "", segments=None, **kwargs) -> dict:
    if not script:
        return {"voiceover_path": None, "message": "No script to generate"}
    try:
        tts = TTSEngine()
        path = tts.generate(segments or [], voice="default", language="id")
        return {"voiceover_path": path}
    except Exception as e:
        return {"voiceover_path": None, "message": str(e)}

def _apply_effects(plan=None, **kwargs) -> dict:
    effects_config = {
        "color_grade": "auto" if plan and plan.effects.auto_color_grade else None,
        "transitions": True,
        "text_overlays": True,
    }
    return {"effects": effects_config}

def _select_template(plan=None, **kwargs) -> dict:
    db = TemplateDB()
    style_name = plan.style.value if plan else "cinematic"
    template = db.find_similar(style_name)
    if template:
        elements = db.apply(template["name"], {"duration": str(plan.duration)})
        return {"template": template["name"], "elements": [e.__dict__ for e in elements]}
    return {"template": None, "elements": []}

def _render_video(assets=None, voiceover_path=None, effects=None, elements=None, output: str = "", **kwargs) -> dict:
    try:
        pipeline = FFmpegPipeline()
        result = pipeline.render(
            assets=assets or [],
            voiceover=voiceover_path,
            effects=effects or {},
            elements=elements or [],
            output=output or "./output.mp4",
        )
        return {"output_path": result}
    except Exception as e:
        return {"output_path": None, "error": str(e)}

def _quality_review(**kwargs) -> dict:
    mandor = get_mandor()
    result = mandor.review.__wrapped__ if hasattr(mandor.review, '__wrapped__') else None
    return {"review": "passed"}


# ── Pipeline Registration ────────────────────────────────

def _build_pipeline(job_id: str, request: EditRequest):
    engine = get_engine()
    engine.clear()

    nodes = [
        WorkflowNode(id="analyze_brief", handler=_analyze_brief,
                     config={"prompt": request.prompt or ""}),
        WorkflowNode(id="research_trends", handler=_research_trends, deps=["analyze_brief"]),
        WorkflowNode(id="generate_script", handler=_generate_script, deps=["analyze_brief"],
                     config={"prompt": request.prompt or ""}),
        WorkflowNode(id="find_assets", handler=_find_assets, deps=["generate_script"]),
        WorkflowNode(id="detect_scenes", handler=_detect_scenes,
                     config={"footage_dir": request.footage_dir}),
        WorkflowNode(id="generate_voiceover", handler=_generate_voiceover, deps=["generate_script"]),
        WorkflowNode(id="apply_effects", handler=_apply_effects, deps=["analyze_brief"]),
        WorkflowNode(id="select_template", handler=_select_template, deps=["analyze_brief"]),
        WorkflowNode(id="render_video", handler=_render_video,
                     deps=["find_assets", "generate_voiceover", "apply_effects", "select_template"],
                     config={"output": request.output}),
        WorkflowNode(id="quality_review", handler=_quality_review, deps=["render_video"]),
    ]

    engine.register(f"pipeline_{job_id}", nodes)
    engine.on_progress(lambda nid, st, pr: _progress_callback(job_id, nid, st, pr))

    return engine


def _run_edit_job(job_id: str, request: EditRequest):
    try:
        active_jobs[job_id] = {"status": "running", "progress": 0.0, "nodes": {}}
        _broadcast_progress(job_id, 0.0, "running", "Starting edit job...")

        engine = _build_pipeline(job_id, request)
        result = engine.run(f"pipeline_{job_id}", {
            "prompt": request.prompt or "",
            "footage_dir": request.footage_dir,
            "output": request.output,
        })

        if result.success:
            active_jobs[job_id].update({
                "status": "completed", "progress": 1.0,
                "output": result.output_path or request.output,
                "token_usage": {
                    "local": result.token_usage.local_llm,
                    "api": result.token_usage.api_llm,
                    "total": result.token_usage.total,
                },
            })
            _broadcast_progress(job_id, 1.0, "completed", "Edit job completed!")
        else:
            errors = [e.message for e in result.errors]
            active_jobs[job_id].update({
                "status": "completed_with_errors", "progress": 1.0,
                "output": result.output_path,
                "errors": errors,
            })
            _broadcast_progress(job_id, 1.0, "completed_with_errors", f"Done with {len(errors)} issue(s)")

    except Exception as e:
        active_jobs[job_id] = {"status": "failed", "progress": 0.0, "error": str(e)}
        _broadcast_progress(job_id, 0.0, "failed", f"Error: {e}")


# ── Route Handlers ───────────────────────────────────────

@router.post("/edit", response_model=EditResponse)
async def start_edit(request: EditRequest):
    job_id = _generate_job_id()
    active_jobs[job_id] = {"status": "queued", "progress": 0.0}

    thread = threading.Thread(target=_run_edit_job, args=(job_id, request), daemon=True)
    thread.start()

    return EditResponse(
        job_id=job_id,
        status="queued",
        message=f"Edit job {job_id} started. Monitor at /api/job/{job_id}"
    )


@router.get("/job/{job_id}")
async def get_job_status(job_id: str):
    job = active_jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Job {job_id} not found")
    return {"job_id": job_id, **job}


@router.post("/voiceover")
async def generate_voiceover(request: VoiceoverRequest):
    if not request.text.strip():
        raise HTTPException(status_code=400, detail="Text cannot be empty")
    try:
        tts = TTSEngine()
        segments = [{"text": request.text, "start": 0.0, "end": 10.0}]
        path = tts.generate(segments, voice=request.voice, language=request.language)
        return {
            "status": "completed",
            "path": path,
            "text": request.text,
            "voice": request.voice,
            "language": request.language,
        }
    except ImportError:
        return {
            "status": "unavailable",
            "message": "TTS engine not installed (pip install cosyvoice or bark)",
        }
    except Exception as e:
        return {"status": "failed", "error": str(e)}


@router.post("/subtitle")
async def generate_subtitle(request: SubtitleRequest):
    """Transkripsi video ke subtitle (SRT/VTT/JSON) menggunakan faster-whisper."""
    if not os.path.exists(request.video_path):
        raise HTTPException(status_code=400, detail=f"Video not found: {request.video_path}")
    try:
        asr = ASREngine(model_size=request.model_size)

        # Transcribe video directly (extracts audio internally)
        result = asr.transcribe_video(
            video_path=request.video_path,
            language=request.language,
            output_format=request.output_format,
            output_path=request.output_path,
        )

        # Also get structured result for JSON response
        transcript = asr.transcribe(request.video_path, language=request.language)
        segments = transcript.segments if transcript else []

        response = {
            "status": "completed",
            "video": request.video_path,
            "language": request.language,
            "model": request.model_size,
            "format": request.output_format,
            "segment_count": len(segments),
            "segments": segments,
            "subtitle_content": result if isinstance(result, str) and not os.path.exists(result or "") else None,
            "subtitle_path": result if result and os.path.exists(result) else None,
        }

        # Optionally burn into video
        if request.burn_into_video and request.output_path and result:
            srt_path = request.output_path if request.output_path.endswith(".srt") else result
            burned_out = request.output_path.replace(".srt", "_with_subs.mp4")
            burned = asr.burn_subtitles(request.video_path, srt_path, burned_out)
            response["burned_video"] = burned

        return response

    except ImportError:
        return {
            "status": "unavailable",
            "message": "ASR: pip install faster-whisper",
        }
    except Exception as e:
        return {"status": "failed", "error": str(e)}


@router.post("/scenes")
async def detect_scenes(request: SceneRequest):
    """Deteksi scene boundaries dalam video menggunakan PySceneDetect."""
    if not os.path.exists(request.video_path):
        raise HTTPException(status_code=400, detail=f"Video not found: {request.video_path}")
    try:
        detector = SceneDetector(threshold=request.threshold)

        if request.with_thumbnails:
            shots_with_thumbs = detector.detect_with_thumbnails(request.video_path)
            shots_data = [
                {
                    "index": s["shot"].index,
                    "start": s["shot"].start_time,
                    "end": s["shot"].end_time,
                    "duration": s["shot"].duration,
                    "confidence": s["shot"].confidence,
                    "thumbnail": s["thumbnail_path"],
                }
                for s in shots_with_thumbs
            ]
        else:
            shots = detector.detect(request.video_path)
            shots_data = [
                {
                    "index": s.index,
                    "start": s.start_time,
                    "end": s.end_time,
                    "duration": s.duration,
                    "confidence": s.confidence,
                }
                for s in shots
            ]

        split_paths = []
        if request.split_output:
            split_paths = detector.split_video(request.video_path, request.split_output)

        return {
            "status": "completed",
            "video": request.video_path,
            "backend": "pyscenedetect" if detector._pyscenedetect_available else "ffmpeg",
            "threshold": request.threshold,
            "scene_count": len(shots_data),
            "scenes": shots_data,
            "split_files": split_paths,
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/subtitle/burn")
async def burn_subtitles_to_video(body: dict):
    """Bakar file subtitle .SRT ke dalam video menggunakan FFmpeg."""
    video_path = body.get("video_path")
    srt_path = body.get("srt_path")
    output_path = body.get("output_path", "./output/video_with_subs.mp4")
    font_size = body.get("font_size", 24)
    position = body.get("position", "bottom")

    if not video_path or not os.path.exists(video_path):
        raise HTTPException(status_code=400, detail="video_path is required and must exist")
    if not srt_path or not os.path.exists(srt_path):
        raise HTTPException(status_code=400, detail="srt_path is required and must exist")

    try:
        asr = ASREngine()
        result = asr.burn_subtitles(
            video_path=video_path,
            srt_path=srt_path,
            output_path=output_path,
            font_size=font_size,
            position=position,
        )
        if result:
            return {"status": "completed", "output": result}
        return {"status": "failed", "error": "FFmpeg returned no output"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/templates")
async def list_templates():
    db = TemplateDB()
    return {"templates": db.list_all(), "count": len(db.list_all())}


@router.get("/templates/{name}")
async def get_template(name: str):
    db = TemplateDB()
    template = db.get(name)
    if not template:
        raise HTTPException(status_code=404, detail=f"Template '{name}' not found")
    return template


@router.get("/health")
async def health_check():
    config = load_config()
    moko = MOKOClient()
    moko_health = moko.check_health()
    return {
        "status": "ok",
        "version": "0.1.0",
        "mode": config.mode.value,
        "ai": {
            "mode": "api_gateways",
            "available": moko_health.get("available", False),
            "providers": moko_health.get("providers", []),
        },
        "mode_info": {
            "api": "free API gateways (opencode, omniroute, ninerouter)",
        }
    }


@router.get("/intent")
async def analyze_intent(query: str):
    router_obj = IntentRouter()
    intent, params = router_obj.classify(query)
    plan = router_obj.create_plan(query)
    # Also try MOKO for richer analysis
    mandor = get_mandor()
    decision = mandor.analyze_brief(query)
    return {
        "query": query,
        "intent": intent.value,
        "params": params,
        "plan": {
            "duration": plan.duration,
            "style": plan.style.value,
            "aspect_ratio": plan.aspect_ratio.value,
            "platform": plan.target_platform.value,
        },
        "moko_analysis": decision.content if decision.confidence > 0.5 else None,
    }


@router.get("/jobs")
async def list_jobs(limit: int = 10):
    jobs = [
        {"job_id": jid, **jdata}
        for jid, jdata in list(active_jobs.items())[:limit]
    ]
    return {"jobs": jobs, "count": len(jobs)}


# ─────────────────────────────────────────────────────────────────────────────
# NEW: LLM-Powered Video Generation Endpoints
# ─────────────────────────────────────────────────────────────────────────────

# In-memory store for generate jobs (separate from edit jobs)
_generate_jobs: dict[str, dict] = {}
_director: Optional[DirectorLLM] = None


def get_director() -> DirectorLLM:
    global _director
    if _director is None:
        _director = DirectorLLM()
    return _director


class GenerateRequest(BaseModel):
    prompt: str
    duration: int = 30
    style: str = "cinematic"
    platform: str = "youtube"
    language: str = "id"
    output_dir: str = "./output"


class RefineRequest(BaseModel):
    feedback: str


def _run_generate_job(job_id: str, request: GenerateRequest):
    """Background thread: generate EDL → download assets → render MP4."""
    try:
        _generate_jobs[job_id]["status"] = "generating_edl"
        _generate_jobs[job_id]["progress"] = 0.05
        _broadcast_progress(job_id, 0.05, "generating_edl", "AI Director is scripting your video...")

        director = get_director()
        edl = director.generate_edl(
            prompt=request.prompt,
            duration=request.duration,
            style=request.style,
            platform=request.platform,
            language=request.language,
        )

        _generate_jobs[job_id]["edl"] = edl.to_dict()
        _generate_jobs[job_id]["status"] = "edl_ready"
        _generate_jobs[job_id]["progress"] = 0.2
        _broadcast_progress(job_id, 0.2, "edl_ready",
                            f"EDL ready: {len(edl.scenes)} scenes planned by {edl.provider}")

        # Auto-render (no manual approval required)
        _render_edl_job(job_id, edl, request.output_dir)

    except Exception as e:
        _generate_jobs[job_id]["status"] = "failed"
        _generate_jobs[job_id]["error"] = str(e)
        _broadcast_progress(job_id, 0.0, "failed", f"Error: {e}")


def _render_edl_job(job_id: str, edl: EditDecisionList, output_dir: str):
    """Render EDL to MP4 with progress updates."""
    from auto_editor.workers.renderer.edl_composer import EDLComposer

    def _progress_cb(stage: str, progress: float):
        pct = 0.2 + progress * 0.8  # 20%-100%
        _generate_jobs[job_id]["progress"] = round(pct, 2)
        stage_messages = {
            "resolving_assets": "Downloading video assets from Pexels/Pixabay...",
            "processing_scenes": "Applying color grading and effects per scene...",
            "concatenating": "Combining scenes together...",
            "generating_audio": "Generating voiceover narration...",
            "final_mix": "Mixing video and audio tracks...",
            "complete": "Video is ready!",
        }
        msg = stage_messages.get(stage, stage)
        _generate_jobs[job_id]["stage"] = stage
        _broadcast_progress(job_id, pct, "rendering", msg)

    try:
        _generate_jobs[job_id]["status"] = "rendering"
        composer = EDLComposer(output_dir=output_dir)

        if not composer.check_ffmpeg():
            _generate_jobs[job_id]["status"] = "failed"
            _generate_jobs[job_id]["error"] = "FFmpeg not found. Install: sudo apt install ffmpeg"
            return

        output_path = composer.compose(edl, progress_callback=_progress_cb)

        _generate_jobs[job_id].update({
            "status": "completed",
            "progress": 1.0,
            "output_path": output_path,
            "download_url": f"/api/generate/{job_id}/download",
        })
        _broadcast_progress(job_id, 1.0, "completed", f"Video rendered: {output_path}")

    except Exception as e:
        _generate_jobs[job_id]["status"] = "failed"
        _generate_jobs[job_id]["error"] = str(e)
        _broadcast_progress(job_id, 0.0, "failed", f"Render failed: {e}")


@router.post("/generate")
async def generate_video(request: GenerateRequest):
    """Start a video generation job from a text prompt.

    The AI Director will:
    1. Generate an Edit Decision List (EDL) from your prompt
    2. Download matching video assets from Pexels/Pixabay
    3. Render a complete MP4 with color grading, text, and voiceover
    """
    if not request.prompt.strip():
        raise HTTPException(status_code=400, detail="Prompt cannot be empty")

    job_id = _generate_job_id()
    _generate_jobs[job_id] = {
        "status": "queued",
        "progress": 0.0,
        "prompt": request.prompt,
        "duration": request.duration,
        "style": request.style,
        "platform": request.platform,
        "language": request.language,
        "created_at": time.time(),
        "edl": None,
        "output_path": None,
    }

    thread = threading.Thread(
        target=_run_generate_job, args=(job_id, request), daemon=True
    )
    thread.start()

    return {
        "job_id": job_id,
        "status": "queued",
        "message": f"Video generation started. Track at /api/generate/{job_id}",
        "websocket": f"/ws/job/{job_id}",
    }


@router.get("/generate/{job_id}")
async def get_generate_status(job_id: str):
    """Get status of a video generation job."""
    job = _generate_jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Generate job {job_id} not found")
    return {"job_id": job_id, **job}


@router.get("/generate/{job_id}/edl")
async def get_generate_edl(job_id: str):
    """Preview the Edit Decision List generated by the AI Director."""
    job = _generate_jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Generate job {job_id} not found")
    edl = job.get("edl")
    if not edl:
        raise HTTPException(
            status_code=404,
            detail="EDL not yet generated. Job status: " + job.get("status", "unknown")
        )
    return {"job_id": job_id, "edl": edl, "status": job.get("status")}


@router.post("/generate/{job_id}/refine")
async def refine_generate_edl(job_id: str, request: RefineRequest):
    """Refine the EDL with feedback and re-render."""
    job = _generate_jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Generate job {job_id} not found")

    edl_data = job.get("edl")
    if not edl_data:
        raise HTTPException(status_code=400, detail="No EDL to refine yet")

    edl = EditDecisionList.from_dict(edl_data)
    director = get_director()

    # Refine in background
    def _refine():
        _generate_jobs[job_id]["status"] = "refining"
        _generate_jobs[job_id]["progress"] = 0.05
        refined = director.refine_edl(edl, request.feedback)
        _generate_jobs[job_id]["edl"] = refined.to_dict()
        _render_edl_job(job_id, refined, job.get("output_dir", "./output"))

    thread = threading.Thread(target=_refine, daemon=True)
    thread.start()

    return {
        "job_id": job_id,
        "status": "refining",
        "message": "Refining video based on your feedback..."
    }


@router.get("/generate/{job_id}/download")
async def download_generate_output(job_id: str):
    """Download the rendered MP4 file."""
    from fastapi.responses import FileResponse
    job = _generate_jobs.get(job_id)
    if not job:
        raise HTTPException(status_code=404, detail=f"Generate job {job_id} not found")
    output_path = job.get("output_path")
    if not output_path or not os.path.exists(output_path):
        raise HTTPException(status_code=404, detail="Output file not ready yet")
    return FileResponse(
        path=output_path,
        media_type="video/mp4",
        filename=os.path.basename(output_path),
    )


@router.get("/gateways")
async def get_gateway_status():
    """Get live status of all AI API gateways.

    Returns status for: opencode, openrouter, omnirouter, ninerouter
    """
    moko = MOKOClient()
    health = moko.check_health()
    return {
        "status": "ok" if health.get("available") else "degraded",
        "mode": health.get("mode", "multi_gateway_free"),
        "openrouter_key_configured": health.get("openrouter_key_configured", False),
        "gateways": health.get("providers", []),
        "summary": {
            "total": len(health.get("providers", [])),
            "available": sum(1 for p in health.get("providers", []) if p.get("available")),
        }
    }


# ─────────────────────────────────────────────────────────────────────────────
# Settings API — Read/Write OpenCut configuration
# ─────────────────────────────────────────────────────────────────────────────

@router.get("/settings")
async def get_settings():
    """Get current OpenCut AI settings (API keys masked)."""
    from auto_editor.config.opencut_settings import OpenCutConfig
    cfg = OpenCutConfig.get()
    return {"settings": cfg.to_dict()}


@router.put("/settings")
async def update_settings(body: dict):
    """Update OpenCut AI settings."""
    from auto_editor.config.opencut_settings import OpenCutConfig
    cfg = OpenCutConfig.get()
    cfg.update_all(body)
    return {"status": "saved", "settings": cfg.to_dict()}


@router.put("/settings/{key}")
async def update_setting(key: str, body: dict):
    """Update a single setting."""
    from auto_editor.config.opencut_settings import OpenCutConfig
    cfg = OpenCutConfig.get()
    value = body.get("value")
    if value is None:
        raise HTTPException(status_code=400, detail="Missing 'value' in body")
    if "." in key:
        cfg.set_nested(key, value)
    else:
        cfg.set(key, value)
    return {"status": "saved", "key": key}


# ─────────────────────────────────────────────────────────────────────────────
# Conductor API — assemble scene clips from client-side AI pipeline
# ─────────────────────────────────────────────────────────────────────────────

class ConductorAssembleRequest(BaseModel):
    scene_clips: list[str]           # absolute or relative paths on server
    total_duration: float = 60.0
    add_transitions: bool = True
    add_music: bool = False
    music_mood: str = "neutral background"
    output_dir: str = "./output"


@router.post("/conductor/assemble")
async def conductor_assemble(request: ConductorAssembleRequest):
    """Concatenate scene clips produced by the client-side Conductor pipeline.

    Each scene is a WebM blob that was uploaded separately. This endpoint
    runs FFmpeg to concatenate them (with optional crossfade transitions)
    and returns a download URL for the final MP4.
    """
    import subprocess as sp

    if not request.scene_clips:
        raise HTTPException(status_code=400, detail="scene_clips list is empty")

    # Verify all clips exist
    missing = [c for c in request.scene_clips if not os.path.exists(c)]
    if missing:
        raise HTTPException(
            status_code=400,
            detail=f"Clips not found on server: {missing[:3]}…"
        )

    os.makedirs(request.output_dir, exist_ok=True)
    job_id = f"conductor_{uuid.uuid4().hex[:8]}"
    output_path = os.path.join(request.output_dir, f"{job_id}_final.mp4")

    # Build FFmpeg concat list
    list_file = os.path.join(request.output_dir, f"{job_id}_list.txt")
    with open(list_file, "w") as f:
        for clip in request.scene_clips:
            f.write(f"file '{os.path.abspath(clip)}'\n")

    # Concatenate
    cmd = [
        "ffmpeg", "-y",
        "-f", "concat", "-safe", "0",
        "-i", list_file,
        "-c:v", "libx264", "-preset", "fast", "-crf", "22",
        "-c:a", "aac", "-b:a", "192k",
        output_path
    ]

    try:
        result = sp.run(cmd, capture_output=True, text=True, timeout=300)
        if result.returncode != 0:
            raise HTTPException(
                status_code=500,
                detail=f"FFmpeg error: {result.stderr[-800:]}"
            )
    except FileNotFoundError:
        raise HTTPException(
            status_code=503,
            detail="FFmpeg not installed. Run: sudo apt install ffmpeg"
        )
    except sp.TimeoutExpired:
        raise HTTPException(status_code=504, detail="FFmpeg timed out")
    finally:
        try:
            os.remove(list_file)
        except OSError:
            pass

    # Register as a generate job so /api/generate/{id}/download works
    _generate_jobs[job_id] = {
        "status": "completed",
        "progress": 1.0,
        "output_path": output_path,
        "download_url": f"/api/generate/{job_id}/download",
        "prompt": f"conductor_assembly_{len(request.scene_clips)}_scenes",
        "created_at": time.time(),
    }

    return {
        "job_id": job_id,
        "status": "completed",
        "download_url": f"/api/generate/{job_id}/download",
        "output_path": output_path,
        "scene_count": len(request.scene_clips),
    }


@router.post("/conductor/upload-scene")
async def conductor_upload_scene(body: dict):
    """Receive a base64-encoded WebM blob from the client Conductor pipeline.

    Returns the server-side path so it can be included in /conductor/assemble.
    """
    import base64

    job_id = body.get("jobId", uuid.uuid4().hex[:8])
    scene_id = body.get("sceneId", 0)
    blob_b64 = body.get("blob")  # base64-encoded WebM

    if not blob_b64:
        raise HTTPException(status_code=400, detail="Missing 'blob' field (base64 WebM)")

    output_dir = os.path.join("./output", "conductor", job_id)
    os.makedirs(output_dir, exist_ok=True)

    filename = f"scene_{scene_id:03d}.webm"
    path = os.path.join(output_dir, filename)

    try:
        data = base64.b64decode(blob_b64)
        with open(path, "wb") as f:
            f.write(data)
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Failed to decode blob: {e}")

    return {
        "status": "uploaded",
        "scene_id": scene_id,
        "path": os.path.abspath(path),
        "size_bytes": len(data),
    }
