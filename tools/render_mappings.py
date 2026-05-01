#!/usr/bin/env python3
"""Render Databento-style mapping tables from schema/mappings/*.yaml.

For each mapping file, produces a Mintlify-compatible MDX page at
docs/api/mappings/<type>.mdx with a per-field crosswalk table:

| Unified field | Type | Kalshi source | Polymarket source | Transform | Notes |

Every cell is derived: type/description from the openpx schema, source paths
from the mapping YAML, source types from the cached upstream specs.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from mapping_lib import load_schema_definitions, load_yaml, resolve_ref

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / "schema" / "openpx.schema.json"
MAPPINGS_DIR = ROOT / "schema" / "mappings"
OUTPUT_DIR = ROOT / "docs" / "api" / "mappings"


def short_ref(ref: str) -> str:
    """Trim '#/components/schemas/X/properties/' for a compact display."""
    prefix = "#/components/schemas/"
    if ref.startswith(prefix):
        rest = ref[len(prefix):]
        return rest.replace("/properties/", ".")
    return ref


def field_type_label(node: dict[str, Any]) -> str:
    t = node.get("type")
    if isinstance(t, list):
        non_null = [x for x in t if x != "null"]
        nullable = "null" in t
        label = "|".join(non_null) if non_null else "null"
        if nullable:
            label = f"{label}?"
    elif isinstance(t, str):
        label = t
    elif "$ref" in node:
        label = node["$ref"].split("/")[-1]
    elif "allOf" in node:
        for sub in node["allOf"]:
            if isinstance(sub, dict) and "$ref" in sub:
                return sub["$ref"].split("/")[-1]
        label = "object"
    else:
        label = "?"
    fmt = node.get("format")
    if fmt:
        label = f"{label} ({fmt})"
    return label


def _md_escape_pipe(s: str) -> str:
    """Pipes break markdown tables; escape them."""
    return s.replace("|", "\\|")


def _code_span(s: str) -> str:
    """Wrap text in backticks. Code-spans in MDX skip JSX parsing, so curly
    braces like `{exchange}:{id}` are safe inside. If the text already contains
    backticks, use double-backticks per CommonMark."""
    s = _md_escape_pipe(s)
    if "`" in s:
        return f"`` {s} ``"
    return f"`{s}`"


def _format_synthetic(s: str) -> str:
    """Render a `synthetic:` value for a table cell. JSX-hazardous content
    (curly braces) goes inside a code span so MDX won't parse it; pure prose
    is left as markdown so embedded backticks render as code-spans."""
    if "{" in s or "}" in s:
        return _code_span(s)
    return _md_escape_pipe(s).replace("\n", " ")


def render_source_cell(
    src: dict[str, Any] | None, spec: dict[str, Any], spec_rel: str
) -> str:
    if not src:
        return "—"
    if src.get("omitted"):
        return "_not exposed_"
    if "synthetic" in src:
        synth = src["synthetic"]
        if isinstance(synth, str):
            return f"_computed_ — {_format_synthetic(synth)}"
        return "_computed_"
    if "ref_unspecced" in src:
        # Direct read from a live-response field the OpenAPI spec doesn't
        # document on this schema. Surface the field name and call out the
        # gap so consumers know not to look for it via the spec.
        field = str(src["ref_unspecced"])
        return f"{_code_span(field)} — _spec gap_"
    ref = src.get("ref")
    if not ref:
        return "_invalid_"
    resolved = resolve_ref(spec, ref)
    label = short_ref(ref)
    if resolved is None:
        return f"{_code_span(label)} ❌"
    src_type = field_type_label(resolved)
    cell = f"{_code_span(label)} ({src_type})"
    fb = src.get("fallback_ref")
    if fb:
        cell += f", fallback {_code_span(short_ref(fb))}"
    return cell


def render_transform(src: dict[str, Any] | None) -> str:
    if not src or src.get("omitted"):
        return ""
    if "synthetic" in src:
        return "synthetic"
    return src.get("transform", "direct")


def _safe_prose(s: str) -> str:
    """Escape MDX-hazardous characters in free-text prose. Braces would otherwise
    open JSX expressions; angle brackets would open JSX tags (e.g. `Vec<X>` is
    parsed as an element); pipes break markdown tables; newlines collapse rows."""
    return (
        s.replace("|", "\\|")
        .replace("\n", " ")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("<", "\\<")
        .replace(">", "\\>")
    )


def render_notes(field: dict[str, Any], srcs: dict[str, dict[str, Any]]) -> str:
    notes: list[str] = []
    if "notes" in field:
        notes.append(field["notes"].strip())
    for ex in srcs:
        n = srcs.get(ex, {}).get("notes")
        if n:
            notes.append(f"**{ex}:** {n.strip()}")
    return _safe_prose(" ".join(notes))


def render_table(mapping: dict[str, Any], unified: dict[str, Any]) -> str:
    type_name = mapping["unified_type"]
    upstream = mapping.get("upstream_specs", {})
    exchanges = list(upstream.keys())
    specs: dict[str, dict[str, Any]] = {}
    for ex, rel in upstream.items():
        p = ROOT / rel
        if p.is_file():
            specs[ex] = load_yaml(p)
        else:
            specs[ex] = {}

    unified_props = unified.get("properties", {})

    counts = {ex: {"sourced": 0, "synthetic": 0, "omitted": 0} for ex in exchanges}
    for f in mapping.get("fields", []):
        srcs = f.get("sources", {}) or {}
        is_global_synthetic = "synthetic" in f and not srcs
        for ex in exchanges:
            if is_global_synthetic:
                counts[ex]["synthetic"] += 1
                continue
            s = srcs.get(ex, {}) or {}
            if s.get("omitted"):
                counts[ex]["omitted"] += 1
            elif "synthetic" in s:
                counts[ex]["synthetic"] += 1
            elif s.get("ref") or "ref_unspecced" in s:
                counts[ex]["sourced"] += 1

    lines: list[str] = []
    lines.append("---")
    lines.append(f'title: "{type_name} mapping"')
    lines.append(
        f'description: "How each upstream exchange schema maps to OpenPX {type_name}."'
    )
    lines.append("---")
    lines.append("")
    desc = (unified.get("description") or "").splitlines()[0].strip()
    if desc:
        lines.append(f"_{desc}_")
        lines.append("")
    lines.append("## Coverage")
    lines.append("")
    lines.append("| Exchange | Sourced | Synthetic | Omitted |")
    lines.append("|---|---|---|---|")
    for ex in exchanges:
        c = counts[ex]
        lines.append(f"| {ex} | {c['sourced']} | {c['synthetic']} | {c['omitted']} |")
    lines.append("")
    lines.append("**Sourced** — value copied or transformed from a documented upstream field.  ")
    lines.append("**Synthetic** — computed by OpenPX, not present upstream.  ")
    lines.append("**Omitted** — upstream does not expose this concept.  ")
    lines.append("")
    lines.append("## Field crosswalk")
    lines.append("")
    header = ["Unified field", "Type"] + [f"{ex} source" for ex in exchanges] + [
        "Transform",
        "Notes",
    ]
    lines.append("| " + " | ".join(header) + " |")
    lines.append("|" + "|".join(["---"] * len(header)) + "|")

    for f in mapping.get("fields", []):
        name = f.get("name")
        if not name or name not in unified_props:
            continue
        u_type = field_type_label(unified_props[name])
        srcs = f.get("sources", {}) or {}
        is_global_synthetic = "synthetic" in f and not srcs
        cells = [f"`{name}`", u_type]
        for ex in exchanges:
            if is_global_synthetic:
                synth = f["synthetic"]
                cells.append(
                    f"_computed_ — {_format_synthetic(synth)}"
                    if isinstance(synth, str)
                    else "_computed_"
                )
            else:
                cells.append(render_source_cell(srcs.get(ex), specs[ex], upstream[ex]))
        if is_global_synthetic:
            cells.append("synthetic")
        else:
            xforms = [render_transform(srcs.get(ex)) for ex in exchanges]
            seen = [x for x in xforms if x]
            cells.append(seen[0] if len(set(seen)) == 1 else " / ".join(xforms))
        cells.append(render_notes(f, srcs))
        lines.append("| " + " | ".join(cells) + " |")

    lines.append("")
    lines.append("## Source specs")
    lines.append("")
    for ex, rel in upstream.items():
        lines.append(f"- **{ex}** &middot; [`{rel}`](https://github.com/openpx-trade/openpx/blob/main/{rel})")
    lines.append("")
    lines.append("> Tables are auto-generated from `schema/mappings/`. CI fails on unresolved $refs and on type mismatches for `transform: direct`. Drift in upstream specs surfaces here on the daily refresh PR.")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    defs = load_schema_definitions(SCHEMA)
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    files = sorted(MAPPINGS_DIR.glob("*.yaml"))
    if not files:
        sys.exit(f"no mapping files in {MAPPINGS_DIR}")

    written = 0
    for p in files:
        mapping = load_yaml(p)
        type_name = mapping.get("unified_type")
        if type_name not in defs:
            print(f"skip {p.name}: openpx schema has no `{type_name}`", file=sys.stderr)
            continue
        out_name = p.stem + ".mdx"
        out_path = OUTPUT_DIR / out_name
        text = render_table(mapping, defs[type_name])
        out_path.write_text(text)
        rel = out_path.relative_to(ROOT)
        print(f"wrote {rel} ({len(text):,} bytes)")
        written += 1
    print(f"\n{written} mapping page(s) rendered")
    return 0


if __name__ == "__main__":
    sys.exit(main())
