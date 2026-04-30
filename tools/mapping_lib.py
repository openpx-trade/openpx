"""Shared primitives for the mapping-system tools.

Both `validate_mappings.py` (the gate) and `render_mappings.py` (the renderer)
operate on the same files — `schema/mappings/*.yaml`,
`schema/openpx.schema.json`, and `schema/upstream/*` — so they share the file
loading and JSON-pointer ref-resolution logic.

The semantic surfaces (type compatibility for validation, type-label display
for rendering) stay in their respective tools because they have different
concerns: validate cares about whether `transform=direct` is sound, render
cares about how to display the type to a human.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import yaml


def load_yaml(path: Path) -> Any:
    return yaml.safe_load(path.read_text())


def load_schema_definitions(schema_path: Path) -> dict[str, Any]:
    """Return the `definitions` (or `$defs`) map from an openpx JSON Schema."""
    schema = json.loads(schema_path.read_text())
    return schema.get("definitions") or schema.get("$defs") or {}


def resolve_ref(spec: dict[str, Any], ref: str) -> dict[str, Any] | None:
    """Resolve a JSON-pointer-style ref like
    `#/components/schemas/Market/properties/ticker` against `spec`. Returns
    None if any segment is missing."""
    if not ref.startswith("#/"):
        return None
    cur: Any = spec
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        else:
            return None
    return cur if isinstance(cur, dict) else None


def normalize_type(t: Any) -> set[str]:
    """Return the set of non-null types for a JSON-Schema/OpenAPI `type` field
    (which can be a string or a list)."""
    if isinstance(t, list):
        return {x for x in t if x and x != "null"}
    if isinstance(t, str):
        return {t} if t != "null" else set()
    return set()
