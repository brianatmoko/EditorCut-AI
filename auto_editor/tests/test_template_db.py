"""Tests for TemplateDB — YAML template management."""

import sys
import tempfile
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.template_db import TemplateDB


TEMPLATES_DIR = str(Path(__file__).parent.parent / "config" / "templates")


def test_list_templates():
    db = TemplateDB(TEMPLATES_DIR)
    templates = db.list_all()
    assert len(templates) >= 3


def test_get_template():
    db = TemplateDB(TEMPLATES_DIR)
    t = db.get("cinematic")
    assert t is not None
    assert "tracks" in t


def test_find_similar():
    db = TemplateDB(TEMPLATES_DIR)
    result = db.find_similar("tiktok product video review")
    assert result is not None


def test_apply_template():
    db = TemplateDB(TEMPLATES_DIR)
    elements = db.apply("cinematic", {"TITLE": "Test Video"})
    assert len(elements) > 0
    for el in elements:
        assert hasattr(el, 'position')
        assert hasattr(el, 'timeline')


def test_save_and_delete():
    with tempfile.TemporaryDirectory() as tmpdir:
        db = TemplateDB(tmpdir)
        db.save("test_template", {
            "name": "test_template",
            "tracks": [{"id": "test", "type": "video"}]
        })
        assert db.get("test_template") is not None
        db.delete("test_template")
        assert db.get("test_template") is None


def test_nonexistent_template():
    db = TemplateDB(TEMPLATES_DIR)
    assert db.get("nonexistent_template") is None
