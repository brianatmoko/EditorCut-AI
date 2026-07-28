"""WebSocket handler for real-time job progress."""

from __future__ import annotations
from fastapi import WebSocket, WebSocketDisconnect
from typing import Optional
import json

from .routes import active_connections, active_jobs


async def job_websocket(websocket: WebSocket, job_id: str):
    """WebSocket endpoint for real-time job progress.

    Usage:
        ws = new WebSocket("ws://localhost:8765/ws/job/{job_id}")
        ws.onmessage = (event) => console.log(event.data)
    """
    await websocket.accept()

    if job_id not in active_connections:
        active_connections[job_id] = []
    active_connections[job_id].append(websocket)

    try:
        if job_id in active_jobs:
            await websocket.send_json({
                "job_id": job_id,
                **active_jobs[job_id],
                "timestamp": __import__('time').time(),
            })

        while True:
            await websocket.receive_text()

    except WebSocketDisconnect:
        if job_id in active_connections:
            active_connections[job_id].remove(websocket)
            if not active_connections[job_id]:
                del active_connections[job_id]
