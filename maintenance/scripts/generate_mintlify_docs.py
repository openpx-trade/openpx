#!/usr/bin/env python3
"""Generate Mintlify-flavored reference MDX pages from schema/openpx.schema.json.

The schema is auto-built from Rust types annotated with `#[derive(schemars::JsonSchema)]`,
which captures `///` doc comments as `description` fields. So this script's
output is 1-1 with the Rust source and stays in lockstep automatically.

Output:
    docs/reference/types.mdx — single comprehensive types reference

Usage:
    python3 maintenance/scripts/generate_mintlify_docs.py

Run via `just docs`. CI's check-sync job verifies the output matches what would
be regenerated.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent.parent
SCHEMA_PATH = ROOT / "schema" / "openpx.schema.json"
OUT_PATH = ROOT / "docs" / "reference" / "types.mdx"


def render_type_ref(t: Any) -> str:
    if isinstance(t, list):
        non_null = [x for x in t if x != "null"]
        suffix = "?" if "null" in t else ""
        return (non_null[0] if non_null else "any") + suffix
    if isinstance(t, dict):
        if "$ref" in t:
            return f"[`{t['$ref'].rsplit('/', 1)[-1]}`](#{t['$ref'].rsplit('/', 1)[-1].lower()})"
        if t.get("type") == "array":
            inner = render_type_ref(t.get("items", {}))
            return f"`{inner}`[]" if "`" not in inner else f"{inner}[]"
        if "enum" in t:
            return " \\| ".join(f"`{v!r}`" for v in t["enum"])
        if "anyOf" in t:
            return " \\| ".join(render_type_ref(o) for o in t["anyOf"])
        return f"`{t.get('type', 'any')}`"
    return f"`{t}`"


def render_field_row(name: str, schema: dict[str, Any], required: bool) -> str:
    type_str = render_type_ref(schema)
    if "enum" in schema:
        type_str = render_type_ref(schema)
    desc = schema.get("description", "").replace("\n", " ").strip()
    flag = "" if required else " *(optional)*"
    return f"| `{name}` | {type_str}{flag} | {desc} |"


def render_object(name: str, schema: dict[str, Any]) -> str:
    desc = schema.get("description", "").strip()
    props = schema.get("properties", {})
    required = set(schema.get("required", []))

    out = [f"## `{name}`", ""]
    if desc:
        out += [desc, ""]
    if props:
        out += ["| Field | Type | Description |", "|---|---|---|"]
        for fname, fschema in props.items():
            out.append(render_field_row(fname, fschema, fname in required))
        out.append("")
    return "\n".join(out)


def render_enum(name: str, schema: dict[str, Any]) -> str:
    desc = schema.get("description", "").strip()
    out = [f"## `{name}`", ""]
    if desc:
        out += [desc, ""]
    if "enum" in schema:
        out += ["**Variants:**", ""]
        for v in schema["enum"]:
            out.append(f"- `{v!r}`")
        out.append("")
    elif "oneOf" in schema:
        out += ["**Variants:**", ""]
        for variant in schema["oneOf"]:
            v_desc = variant.get("description", "").strip()
            if "enum" in variant:
                for v in variant["enum"]:
                    line = f"- `{v!r}`"
                    if v_desc:
                        line += f" — {v_desc}"
                    out.append(line)
            elif "properties" in variant:
                for k in variant["properties"]:
                    out.append(f"- `{k}`")
        out.append("")
    return "\n".join(out)


def render_definition(name: str, schema: dict[str, Any]) -> str:
    if "enum" in schema or "oneOf" in schema:
        return render_enum(name, schema)
    return render_object(name, schema)


def main() -> int:
    schema = json.loads(SCHEMA_PATH.read_text())
    defs: dict[str, dict[str, Any]] = schema.get("definitions", {})

    parts = [
        "---",
        'title: "Types"',
        'description: "Auto-generated reference for every type exported by OpenPX. Source: schema/openpx.schema.json (built from Rust #[derive(schemars::JsonSchema)] annotations)."',
        "---",
        "",
        "<Note>",
        "This page is auto-generated from `schema/openpx.schema.json` by `just docs`.",
        "Edit the corresponding Rust types and doc-comments in `engine/core/src/`",
        "and regenerate — do not hand-edit this page.",
        "</Note>",
        "",
    ]

    for name in sorted(defs):
        parts.append(render_definition(name, defs[name]))
        parts.append("")

    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUT_PATH.write_text("\n".join(parts))
    print(f"wrote {OUT_PATH.relative_to(ROOT)} ({len(defs)} types)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
