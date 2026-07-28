"""Tests for IntentRouter — rule-based command classification."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.intent_router import IntentRouter
from auto_editor.models import EditingIntent


def test_classify_auto_edit():
    router = IntentRouter()
    tests = [
        ("buat video cinematic", EditingIntent.AUTO_EDIT),
        ("bikin video produk 30 detik", EditingIntent.AUTO_EDIT),
        ("buatkan video promosi", EditingIntent.AUTO_EDIT),
        ("create a video tutorial", EditingIntent.AUTO_EDIT),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected, f"Failed: '{query}' -> {intent}, expected {expected}"


def test_classify_voiceover():
    router = IntentRouter()
    tests = [
        ("tambah voiceover", EditingIntent.ADD_VOICEOVER),
        ("buat narasi", EditingIntent.ADD_VOICEOVER),
        ("tambahkan dubbing", EditingIntent.ADD_VOICEOVER),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_subtitle():
    router = IntentRouter()
    tests = [
        ("buat subtitle", EditingIntent.ADD_SUBTITLE),
        ("tambah teks", EditingIntent.ADD_SUBTITLE),
        ("generate caption", EditingIntent.ADD_SUBTITLE),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_render():
    router = IntentRouter()
    tests = [
        ("render semua", EditingIntent.BATCH_RENDER),
        ("export video", EditingIntent.RENDER),
        ("simpan hasil", EditingIntent.RENDER),
    ]
    for query, expected in tests:
        intent, _ = router.classify(query)
        assert intent == expected


def test_classify_unknown():
    router = IntentRouter()
    tests = [
        "apa kabar",
        "siapa nama kamu",
        "hello world",
        "testing 123",
    ]
    for query in tests:
        intent, _ = router.classify(query)
        assert intent == EditingIntent.UNKNOWN, f"Failed: '{query}' -> {intent}"


def test_extract_duration():
    router = IntentRouter()
    assert router.extract_duration("30 detik") == 30
    assert router.extract_duration("2 menit") == 120
    assert router.extract_duration("5 minute video") == 300
    assert router.extract_duration("no duration here") is None


def test_extract_style():
    router = IntentRouter()
    assert router.extract_style("cinematic video") == "cinematic"
    assert router.extract_style("vlog style") == "vlog"
    assert router.extract_style("tutorial content") == "tutorial"
    assert router.extract_style("no style") is None


def test_create_plan():
    router = IntentRouter()
    plan = router.create_plan("buat video tiktok 30 detik produk kopi")
    assert plan.duration == 30
    assert plan.aspect_ratio.value == "9:16"
    assert plan.intent == EditingIntent.AUTO_EDIT
