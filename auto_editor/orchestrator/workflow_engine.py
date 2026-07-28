"""DAG-based workflow engine for video editing pipelines."""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any, Callable, Optional
from enum import Enum
import time
import threading

from ..models import WorkflowResult, EditError, TokenUsage


class NodeStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class WorkflowNode:
    """Single node in the workflow DAG."""
    id: str
    handler: Callable[..., Any]
    deps: list[str] = field(default_factory=list)
    config: dict = field(default_factory=dict)
    retry_count: int = 2
    timeout: int = 300
    status: NodeStatus = NodeStatus.PENDING
    result: Any = None
    error: Optional[str] = None

    def is_ready(self, completed: set[str]) -> bool:
        return all(dep in completed for dep in self.deps)


class WorkflowEngine:
    """Execute DAG-based workflows with parallel support."""

    def __init__(self):
        self._workflows: dict[str, list[WorkflowNode]] = {}
        self._progress_callbacks: list[Callable] = []

    def register(self, name: str, nodes: list[WorkflowNode]) -> None:
        self._validate_workflow(name, nodes)
        self._workflows[name] = nodes

    def _validate_workflow(self, name: str, nodes: list[WorkflowNode]) -> None:
        node_ids = {n.id for n in nodes}
        if len(node_ids) != len(nodes):
            ids = [n.id for n in nodes]
            duplicates = {id_ for id_ in ids if ids.count(id_) > 1}
            raise ValueError(f"Duplicate node IDs in '{name}': {duplicates}")
        all_deps = set()
        for n in nodes:
            all_deps.update(n.deps)
        missing = all_deps - node_ids
        if missing:
            raise ValueError(f"Missing dependencies in '{name}': {missing}")
        visited = set()
        path = []
        def dfs(node_id: str) -> None:
            if node_id in path:
                cycle_start = path[path.index(node_id):]
                raise ValueError(f"Circular dependency in '{name}': {' → '.join(cycle_start + [node_id])}")
            if node_id in visited:
                return
            visited.add(node_id)
            path.append(node_id)
            node = next(n for n in nodes if n.id == node_id)
            for dep in node.deps:
                dfs(dep)
            path.pop()
        for n in nodes:
            if n.id not in visited:
                dfs(n.id)

    def get_workflow(self, name: str) -> list[WorkflowNode]:
        if name not in self._workflows:
            raise KeyError(f"Workflow '{name}' not registered")
        return self._workflows[name]

    def list_workflows(self) -> list[str]:
        return list(self._workflows.keys())

    def on_progress(self, callback: Callable[[str, NodeStatus, float], None]) -> None:
        self._progress_callbacks.append(callback)

    def run(self, name: str, input_data: dict) -> WorkflowResult:
        if name not in self._workflows:
            raise KeyError(f"Workflow '{name}' not registered")
        nodes = [WorkflowNode(**{**n.__dict__, 'status': NodeStatus.PENDING})
                 for n in self._workflows[name]]
        completed: set[str] = set()
        results: dict[str, Any] = {}
        errors: list[EditError] = []
        start_time = time.time()
        token_usage = TokenUsage()

        while len(completed) < len(nodes):
            ready = [
                n for n in nodes
                if n.status == NodeStatus.PENDING and n.is_ready(completed)
            ]
            if not ready and len(completed) < len(nodes):
                pending = [n for n in nodes if n.status == NodeStatus.PENDING]
                stalled = [n for n in pending if not n.is_ready(completed)]
                if stalled:
                    errors.append(EditError(
                        node_id=stalled[0].id, error_type="stalled_dependency",
                        message=f"Node '{stalled[0].id}' has unmet dependencies", recoverable=False
                    ))
                    for s in stalled:
                        s.status = NodeStatus.SKIPPED
                        completed.add(s.id)
                    continue
                break

            threads = []
            for node in ready:
                node.status = NodeStatus.RUNNING
                self._notify_progress(node.id, NodeStatus.RUNNING, 0.0)
                thread = threading.Thread(
                    target=self._execute_node, args=(node, input_data, results, errors, token_usage)
                )
                thread.start()
                threads.append((node, thread))

            for node, thread in threads:
                thread.join(timeout=node.timeout)
                if thread.is_alive():
                    node.status = NodeStatus.FAILED
                    errors.append(EditError(
                        node_id=node.id, error_type="timeout",
                        message=f"Node '{node.id}' timed out after {node.timeout}s",
                        recoverable=True, recovery_action="retry"
                    ))
                completed.add(node.id)
                self._notify_progress(node.id, node.status, 1.0 if node.status == NodeStatus.SUCCESS else 0.0)

        end_time = time.time()
        success = all(n.status == NodeStatus.SUCCESS for n in nodes)
        quality_score = 1.0 if success else max(0.0, 1.0 - len(errors) * 0.2)

        return WorkflowResult(
            success=success, output_path=results.get("output_path"),
            token_usage=token_usage, errors=errors,
            quality_score=quality_score, processing_time=end_time - start_time
        )

    def _execute_node(self, node: WorkflowNode, input_data: dict, results: dict, errors: list, token_usage: TokenUsage) -> None:
        for attempt in range(node.retry_count + 1):
            try:
                node_input = {**input_data}
                for dep_id in node.deps:
                    if dep_id in results:
                        dep_result = results[dep_id]
                        if isinstance(dep_result, dict):
                            node_input.update(dep_result)
                result = node.handler(**node_input)
                if isinstance(result, dict) and "_token_cost" in result:
                    token_usage.add_local(result["_token_cost"])
                results[node.id] = result
                node.status = NodeStatus.SUCCESS
                node.result = result
                return
            except Exception as e:
                if attempt < node.retry_count:
                    time.sleep(2 ** attempt)
                else:
                    node.status = NodeStatus.FAILED
                    node.error = str(e)
                    errors.append(EditError(
                        node_id=node.id, error_type="execution_error",
                        message=str(e), recoverable=True, recovery_action="retry or skip"
                    ))

    def _notify_progress(self, node_id: str, status: NodeStatus, progress: float) -> None:
        for cb in self._progress_callbacks:
            try:
                cb(node_id, status, progress)
            except Exception:
                pass

    def clear(self) -> None:
        self._workflows.clear()
        self._progress_callbacks.clear()
