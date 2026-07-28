"""Tests for WorkflowEngine — DAG execution with parallel support."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from auto_editor.orchestrator.workflow_engine import WorkflowEngine, WorkflowNode


def test_simple_workflow():
    engine = WorkflowEngine()
    results = {}

    def node_a(**kw):
        results["a"] = True
        return {"from_a": "done"}

    engine.register("test", [WorkflowNode(id="a", handler=node_a)])
    result = engine.run("test", {})
    assert result.success
    assert results["a"]


def test_sequential_workflow():
    engine = WorkflowEngine()
    order = []

    def node_a(**kw):
        order.append("a")
        return {"val": 1}

    def node_b(**kw):
        order.append("b")
        assert kw.get("val") == 1
        return {"val": 2}

    engine.register("seq", [
        WorkflowNode(id="a", handler=node_a),
        WorkflowNode(id="b", handler=node_b, deps=["a"]),
    ])
    result = engine.run("seq", {})
    assert result.success
    assert order == ["a", "b"]


def test_parallel_workflow():
    engine = WorkflowEngine()

    def node_a(**kw):
        return {"from_a": True}

    def node_b(**kw):
        return {"from_b": True}

    def node_c(**kw):
        return {"merged": {**kw}}

    engine.register("parallel", [
        WorkflowNode(id="a", handler=node_a),
        WorkflowNode(id="b", handler=node_b),
        WorkflowNode(id="c", handler=node_c, deps=["a", "b"]),
    ])
    result = engine.run("parallel", {})
    assert result.success
    assert result.processing_time > 0


def test_node_failure():
    engine = WorkflowEngine()

    def failing_node(**kw):
        raise RuntimeError("Simulated failure")

    engine.register("fail", [WorkflowNode(id="a", handler=failing_node, retry_count=0)])
    result = engine.run("fail", {})
    assert not result.success
    assert len(result.errors) > 0


def test_invalid_dependency():
    engine = WorkflowEngine()
    try:
        engine.register("bad", [
            WorkflowNode(id="a", handler=lambda **_: {}),
            WorkflowNode(id="b", handler=lambda **_: {}, deps=["nonexistent"]),
        ])
        assert False, "Should have raised ValueError"
    except ValueError:
        pass


def test_circular_dependency():
    engine = WorkflowEngine()
    try:
        engine.register("circular", [
            WorkflowNode(id="a", handler=lambda **_: {}, deps=["b"]),
            WorkflowNode(id="b", handler=lambda **_: {}, deps=["a"]),
        ])
        assert False, "Should have raised ValueError"
    except ValueError:
        pass


def test_progress_callback():
    engine = WorkflowEngine()
    updates = []

    def callback(node_id, status, progress):
        updates.append((node_id, status))

    engine.on_progress(callback)
    engine.register("progress", [
        WorkflowNode(id="a", handler=lambda **_: {}),
    ])
    engine.run("progress", {})
    assert len(updates) > 0
