"""Tests for REST API routes."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from fastapi.testclient import TestClient
from auto_editor.api.server import app

client = TestClient(app)


def test_health():
    r = client.get("/api/health")
    assert r.status_code == 200
    data = r.json()
    assert data["status"] == "ok"


def test_intent_analysis():
    r = client.get("/api/intent", params={"query": "buat video cinematic 30 detik"})
    assert r.status_code == 200
    data = r.json()
    assert data["intent"] == "auto_edit"
    assert data["plan"]["duration"] == 30


def test_list_templates():
    r = client.get("/api/templates")
    assert r.status_code == 200
    data = r.json()
    assert data["count"] >= 10


def test_get_template():
    r = client.get("/api/templates/cinematic")
    assert r.status_code == 200
    data = r.json()
    assert data["name"] == "cinematic"
    assert "tracks" in data


def test_start_edit():
    r = client.post("/api/edit", json={
        "footage_dir": "./",
        "prompt": "buat video",
        "output": "./test_output.mp4"
    })
    assert r.status_code == 200
    data = r.json()
    assert "job_id" in data
    assert data["status"] == "queued"


def test_get_job():
    r = client.post("/api/edit", json={"footage_dir": "./", "prompt": "test"})
    job_id = r.json()["job_id"]

    r = client.get(f"/api/job/{job_id}")
    assert r.status_code == 200
    assert r.json()["job_id"] == job_id


def test_get_nonexistent_job():
    r = client.get("/api/job/nonexistent_job")
    assert r.status_code == 404


def test_voiceover_empty():
    r = client.post("/api/voiceover", json={"text": ""})
    assert r.status_code == 400


def test_root():
    r = client.get("/")
    assert r.status_code == 200
    assert "name" in r.json()
