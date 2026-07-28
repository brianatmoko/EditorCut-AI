"""FastAPI application server for auto-editor API.

Usage:
    python -m auto_editor.api.server
    # or: uvicorn auto_editor.api.server:app --port 8765
"""

from __future__ import annotations
import os
import sys
from pathlib import Path
from contextlib import asynccontextmanager

# Prepend local project bin to PATH to locate local static FFmpeg/FFprobe
_ROOT_DIR = Path(__file__).resolve().parent.parent.parent
_LOCAL_BIN = _ROOT_DIR / "bin"
if _LOCAL_BIN.exists():
    os.environ["PATH"] = str(_LOCAL_BIN) + os.pathsep + os.environ.get("PATH", "")

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
import uvicorn
import json
import logging

from .routes import router
from .websocket import job_websocket
from .stickman_routes import router as stickman_router
from .infographics_routes import router as infographics_router
from .studio_routes import router as studio_router
from .ai_chat_routes import router as ai_chat_router


logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan: startup → yield → shutdown."""
    try:
        from moko_bridge.moko_client import MOKOClient
        moko = MOKOClient()
        health = moko.check_health()
        available = health.get("available", False)
        if available:
            providers = [p["name"] for p in health.get("providers", []) if p.get("available")]
            logger.info("[API] Free AI gateways available: %s", providers)
        else:
            offline = [p["name"] for p in health.get("providers", [])]
            logger.warning(
                "[API] No free AI gateways available (%s). AI features will use fallback responses.",
                offline if offline else "none configured",
            )
    except Exception as exc:
        logger.warning("[API] Could not check AI gateway health: %s", exc)
    yield
    # Shutdown — nothing to clean up currently


app = FastAPI(
    title="OpenCut AI Auto-Editor API",
    version="0.1.0",
    description="Token-efficient AI video editing automation API.",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(router)
app.include_router(stickman_router)
app.include_router(infographics_router)
app.include_router(studio_router)
app.include_router(ai_chat_router)




@app.websocket("/ws")
async def websocket_general(websocket: WebSocket):
    """Generic WebSocket for the AutoEditorBridge client."""
    await websocket.accept()
    try:
        while True:
            raw = await websocket.receive_text()
            try:
                msg = json.loads(raw)
            except json.JSONDecodeError:
                continue
            msg_id = msg.get("id", "")
            msg_type = msg.get("type", "")
            payload = msg.get("payload", {})

            if msg_type == "start_job":
                from .routes import start_edit, EditRequest
                intent = payload.get("intent", "auto_edit")
                req = EditRequest(
                    footage_dir=payload.get("footage_dir", "./auto_editor/test_footage"),
                    prompt=payload.get("params", {}).get("prompt", ""),
                    output=f"./output/{msg_id}.mp4",
                )
                job_resp = await start_edit(req)
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"job_id": job_resp.job_id, "status": job_resp.status},
                })
            elif msg_type == "job_status":
                from .routes import active_jobs
                job_data = active_jobs.get(payload.get("jobId", ""))
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": job_data or {"status": "unknown"},
                })
            elif msg_type == "analyze_timeline":
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"analysis": "timeline analysis placeholder"},
                })
            elif msg_type == "detect_scenes":
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"shots": []},
                })
            elif msg_type == "generate_voiceover":
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"voiceover_path": None, "note": "Voiceover generation via MOKO TTS"},
                })
            elif msg_type == "transcribe":
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"segments": []},
                })
            elif msg_type == "search_assets":
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"assets": []},
                })
            else:
                await websocket.send_json({
                    "id": msg_id, "type": "response",
                    "payload": {"error": f"Unknown message type: {msg_type}"},
                })
    except WebSocketDisconnect:
        pass

@app.websocket("/ws/job/{job_id}")
async def websocket_endpoint(websocket: WebSocket, job_id: str):
    await job_websocket(websocket, job_id)


@app.get("/")
async def root():
    from moko_bridge.moko_client import MOKOClient
    moko = MOKOClient()
    health = moko.check_health()
    return {
        "name": "OpenCut AI Auto-Editor API",
        "version": "0.1.0",
        "docs": "/docs",
        "health": "/api/health",
        "ai": {
            "mode": "api_gateways",
            "available": health.get("available", False),
            "providers": health.get("providers", []),
        },
    }






def main():
    """Run the API server."""
    uvicorn.run(
        "auto_editor.api.server:app",
        host="0.0.0.0",
        port=8765,
        reload=True,
        log_level="info",
    )


if __name__ == "__main__":
    main()
