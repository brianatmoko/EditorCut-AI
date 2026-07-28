"""Layout template manager — load, save, search, apply templates."""

from __future__ import annotations
from pathlib import Path
from typing import Optional, Any
import yaml
import re

from ..models import CoordinateElement, Position, Size, Timeline, Transform, TextStyle

_DEFAULT_TEMPLATES_DIR = str(Path(__file__).parent.parent / "config" / "templates")


class TemplateDB:
    """Manage layout templates stored as YAML files."""

    def __init__(self, templates_dir: str = _DEFAULT_TEMPLATES_DIR):
        self._templates_dir = Path(templates_dir)
        self._cache: dict[str, dict] = {}
        self._load_all()

    def _load_all(self) -> None:
        if not self._templates_dir.exists():
            self._templates_dir.mkdir(parents=True, exist_ok=True)
            return
        for yaml_file in self._templates_dir.glob("*.yaml"):
            try:
                with open(yaml_file) as f:
                    data = yaml.safe_load(f)
                    if data and "name" in data:
                        self._cache[data["name"]] = data
            except (yaml.YAMLError, IOError) as e:
                print(f"Warning: Failed to load template '{yaml_file}': {e}")

    def list_all(self) -> list[dict]:
        return [
            {"name": t.get("name"), "description": t.get("description", ""),
             "style": t.get("style", "custom"), "aspect_ratio": t.get("aspect_ratio", "16:9")}
            for t in self._cache.values()
        ]

    def get(self, name: str) -> Optional[dict]:
        return self._cache.get(name)

    def find_similar(self, query: str) -> Optional[dict]:
        query_lower = query.lower()
        query_keywords = set(re.findall(r'\w+', query_lower))
        best_score = 0
        best_template = None
        for name, template in self._cache.items():
            searchable = f"{name} {template.get('description', '')} "
            searchable += f"{template.get('style', '')} "
            searchable += " ".join(template.get("tags", []))
            tmpl_keywords = set(re.findall(r'\w+', searchable.lower()))
            overlap = query_keywords & tmpl_keywords
            if not overlap:
                continue
            score = sum(
                3 if kw in name.lower() else
                2 if kw in template.get('description', '').lower() else
                2 if kw in template.get('style', '').lower() else
                1
                for kw in overlap
            )
            if score > best_score:
                best_score = score
                best_template = template
        return best_template

    def apply(self, name: str, variables: dict[str, str]) -> list[CoordinateElement]:
        template = self.get(name)
        if not template:
            raise KeyError(f"Template '{name}' not found")
        elements = []
        for track in template.get("tracks", []):
            track_str = yaml.dump(track)
            for var_name, var_value in variables.items():
                track_str = track_str.replace(f"{{{var_name}}}", str(var_value))
            resolved = yaml.safe_load(track_str)
            element = self._track_to_element(resolved)
            elements.append(element)
        return elements

    def save(self, name: str, data: dict) -> None:
        filepath = self._templates_dir / f"{name}.yaml"
        with open(filepath, "w") as f:
            yaml.dump(data, f, default_flow_style=False, allow_unicode=True)
        self._cache[name] = data

    def delete(self, name: str) -> bool:
        filepath = self._templates_dir / f"{name}.yaml"
        if filepath.exists():
            filepath.unlink()
            self._cache.pop(name, None)
            return True
        return False

    def _track_to_element(self, track: dict) -> CoordinateElement:
        pos = track.get("position", {})
        sz = track.get("size", {})
        tml = track.get("timeline", {})
        trf = track.get("transform", {})
        style = track.get("style", {})
        element = CoordinateElement(
            id=track.get("id", "untitled"),
            type=track.get("type", "video"),
            position=Position(x=pos.get("x", 0.5), y=pos.get("y", 0.5), z=pos.get("z", 0)),
            size=Size(width=sz.get("width", 0.5), height=sz.get("height", 0.5), unit=sz.get("unit", "normalized")),
            timeline=Timeline(start=tml.get("start", 0.0), end=tml.get("end", 10.0)),
            transform=Transform(rotation=trf.get("rotation", 0.0), scale=trf.get("scale", 1.0),
                                opacity=trf.get("opacity", 1.0), anchor=trf.get("anchor", "center")),
        )
        if element.type == "text" and style:
            element.text_style = TextStyle(
                text=style.get("text", ""), font_family=style.get("font_family", "Inter"),
                font_size=style.get("font_size", 48), font_weight=style.get("font_weight", 400),
                color=style.get("color", "#FFFFFF"), text_align=style.get("text_align", "center"),
            )
        return element

    def reload(self) -> None:
        self._cache.clear()
        self._load_all()
