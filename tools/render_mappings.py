#!/usr/bin/env python3
"""Render Databento-style mapping tables from schema/mappings/*.yaml.

For each mapping file, produces a Mintlify-compatible MDX page at
docs/schemas/mappings/<type>.mdx with a per-field crosswalk table:

    | Unified field | Type | <exchange> source | … | Notes |

Each per-exchange source cell is one of three types — `direct`, `synthetic`,
or `omitted` — driven by the `type:` key on every source entry.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from mapping_lib import (
    CHANNEL_SECTIONS,
    load_schema_definitions,
    load_yaml,
    mapping_kind,
    resolve_ref,
)

ROOT = Path(__file__).resolve().parent.parent
SCHEMA = ROOT / "schema" / "openpx.schema.json"
MAPPINGS_DIR = ROOT / "schema" / "mappings"
OUTPUT_DIR = ROOT / "docs" / "schemas" / "mappings"


def short_ref(ref: str) -> str:
    """Trim '#/components/schemas/X/properties/' for compact display."""
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
    return s.replace("|", "\\|")


def _code_span(s: str) -> str:
    s = _md_escape_pipe(s)
    if "`" in s:
        return f"`` {s} ``"
    return f"`{s}`"


def _safe_prose(s: str) -> str:
    """Escape MDX-hazardous characters in free-text prose."""
    return (
        s.replace("|", "\\|")
        .replace("\n", " ")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("<", "\\<")
        .replace(">", "\\>")
    )


def render_source_cell(src: dict[str, Any], spec: dict[str, Any]) -> str:
    t = src.get("type")
    if t == "omitted":
        return "_omitted_"
    if t == "synthetic":
        return "_synthetic_"
    if t == "direct":
        ref = src.get("ref", "")
        if not ref:
            return "_direct_ ❌ (no ref)"
        if ref.startswith("#/"):
            label = short_ref(ref)
            resolved = resolve_ref(spec, ref)
            cell = _code_span(label)
            if resolved is None:
                cell += " ❌"
            fb = src.get("fallback_ref")
            if fb:
                cell += f", fallback {_code_span(short_ref(fb))}"
            return cell
        # Bare-name ref — spec gap (live response field not in OpenAPI).
        return f"{_code_span(ref)} _(spec gap)_"
    return "—"


def render_table(mapping: dict[str, Any], unified: dict[str, Any]) -> str:
    type_name = mapping["unified_type"]
    upstream = mapping.get("upstream_specs", {})
    exchanges = list(upstream.keys())
    specs: dict[str, dict[str, Any]] = {}
    for ex, rel in upstream.items():
        p = ROOT / rel
        specs[ex] = load_yaml(p) if p.is_file() else {}

    unified_props = unified.get("properties", {})

    counts = {ex: {"direct": 0, "synthetic": 0, "omitted": 0} for ex in exchanges}
    for f in mapping.get("fields", []):
        srcs = f.get("sources", {}) or {}
        for ex in exchanges:
            t = (srcs.get(ex) or {}).get("type")
            if t in counts[ex]:
                counts[ex][t] += 1

    lines: list[str] = []
    lines.append("---")
    lines.append(f'title: "{type_name} mapping"')
    lines.append(
        f'description: "How each upstream exchange schema maps to OpenPX {type_name}."'
    )
    lines.append("---")
    lines.append("")
    raw_desc = (unified.get("description") or "").splitlines()
    desc = raw_desc[0].strip() if raw_desc else ""
    if desc:
        lines.append(f"_{desc}_")
        lines.append("")
    lines.append("Every field is one of three entry types — **direct** (taken from upstream), **synthetic** (computed by OpenPX), or **omitted** (not exposed upstream).")
    lines.append("")
    lines.append("## Coverage")
    lines.append("")
    lines.append("| Exchange | Direct | Synthetic | Omitted |")
    lines.append("|---|---|---|---|")
    for ex in exchanges:
        c = counts[ex]
        lines.append(f"| {ex} | {c['direct']} | {c['synthetic']} | {c['omitted']} |")
    lines.append("")
    lines.append("## Field crosswalk")
    lines.append("")
    header = ["Unified field", "Type"] + [f"{ex} source" for ex in exchanges] + ["Notes"]
    lines.append("| " + " | ".join(header) + " |")
    lines.append("|" + "|".join(["---"] * len(header)) + "|")

    for f in mapping.get("fields", []):
        name = f.get("name")
        if not name or name not in unified_props:
            continue
        u_type = field_type_label(unified_props[name])
        srcs = f.get("sources", {}) or {}
        cells = [f"`{name}`", u_type]
        notes: list[str] = []
        for ex in exchanges:
            src = srcs.get(ex) or {}
            cells.append(render_source_cell(src, specs[ex]))
            n = src.get("note")
            if n:
                notes.append(f"**{ex}:** {n.strip()}")
        cells.append(_safe_prose(" ".join(notes)))
        lines.append("| " + " | ".join(cells) + " |")

    lines.append("")
    lines.append("## Source specs")
    lines.append("")
    for ex, rel in upstream.items():
        lines.append(f"- **{ex}** &middot; [`{rel}`](https://github.com/openpx-trade/openpx/blob/main/{rel})")
    lines.append("")
    lines.append("> Tables are auto-generated from `schema/mappings/`. CI fails if any `direct` ref no longer resolves in the cached upstream spec; the daily upstream-refresh PR surfaces drift here.")
    lines.append("")
    return "\n".join(lines)


# --------------------------------------------------------------------------
# Channel-mapping renderer (mapping_kind: channel)
# --------------------------------------------------------------------------
# Channel mappings document a WebSocket channel rather than a REST model:
# how the unified subscribe payload, the unified receive variants, and the
# unified session-event variants map to each exchange's upstream message.
# The output is a Mintlify MDX page with one table per section.

# Pretty title labels per section. Keys mirror mapping_lib.CHANNEL_SECTIONS.
SECTION_TITLES = {
    "subscribe_payload": "Subscribe payload",
    "receive_messages": "Receive messages",
    "session_events": "Session events",
}


def _channel_count(mapping: dict[str, Any], exchange: str) -> dict[str, int]:
    counts = {"direct": 0, "synthetic": 0, "omitted": 0}
    for section in CHANNEL_SECTIONS:
        for entry in mapping.get(section, []) or []:
            src = (entry.get("sources") or {}).get(exchange) or {}
            t = src.get("type")
            if t in counts:
                counts[t] += 1
    return counts


def render_channel_section(
    mapping: dict[str, Any],
    section: str,
    specs: dict[str, dict[str, Any]],
) -> list[str]:
    entries = mapping.get(section, []) or []
    if not entries:
        return []
    exchanges = list(mapping.get("upstream_specs", {}).keys())
    name_header = "Variant" if section != "subscribe_payload" else "Field"
    header = [name_header] + [f"{ex} source" for ex in exchanges] + ["Notes"]
    out: list[str] = []
    out.append(f"## {SECTION_TITLES[section]}")
    out.append("")
    out.append("| " + " | ".join(header) + " |")
    out.append("|" + "|".join(["---"] * len(header)) + "|")
    for entry in entries:
        name = entry.get("name") or entry.get("variant") or "?"
        srcs = entry.get("sources") or {}
        cells = [f"`{name}`"]
        notes: list[str] = []
        for ex in exchanges:
            src = srcs.get(ex) or {}
            cells.append(render_source_cell(src, specs.get(ex, {})))
            n = src.get("note")
            if n:
                notes.append(f"**{ex}:** {n.strip()}")
        cells.append(_safe_prose(" ".join(notes)))
        out.append("| " + " | ".join(cells) + " |")
    out.append("")
    return out


def render_channel(mapping: dict[str, Any]) -> str:
    channel = mapping["unified_channel"]
    upstream = mapping.get("upstream_specs", {})
    exchanges = list(upstream.keys())
    specs: dict[str, dict[str, Any]] = {}
    for ex, rel in upstream.items():
        p = ROOT / rel
        specs[ex] = load_yaml(p) if p.is_file() else {}

    title = f"{channel.title()} stream mapping"
    lines: list[str] = []
    lines.append("---")
    lines.append(f'title: "{title}"')
    lines.append(
        f'description: "How each upstream WebSocket channel maps to the unified OpenPX `{channel}` stream."'
    )
    lines.append("---")
    lines.append("")
    lines.append(
        f"Subscribe inputs and receive frames for the unified **{channel}** WebSocket channel,"
    )
    lines.append(
        "crosswalked against the upstream AsyncAPI specs cached under `schema/upstream/`."
    )
    lines.append("")
    lines.append("Every source entry is one of three types — **direct** (taken from upstream), **synthetic** (constructed by OpenPX), or **omitted** (not exposed upstream).")
    lines.append("")
    lines.append("## Coverage")
    lines.append("")
    lines.append("| Exchange | Direct | Synthetic | Omitted |")
    lines.append("|---|---|---|---|")
    for ex in exchanges:
        c = _channel_count(mapping, ex)
        lines.append(f"| {ex} | {c['direct']} | {c['synthetic']} | {c['omitted']} |")
    lines.append("")
    for section in CHANNEL_SECTIONS:
        lines.extend(render_channel_section(mapping, section, specs))
    lines.append("## Source specs")
    lines.append("")
    for ex, rel in upstream.items():
        lines.append(
            f"- **{ex}** &middot; [`{rel}`](https://github.com/openpx-trade/openpx/blob/main/{rel})"
        )
    lines.append("")
    lines.append(
        "> Tables are auto-generated from `schema/mappings/`. CI fails if any `direct` ref no longer resolves in the cached upstream AsyncAPI spec; the daily upstream-refresh PR surfaces drift here."
    )
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
        kind = mapping_kind(mapping)
        out_path = OUTPUT_DIR / (p.stem + ".mdx")

        if kind == "channel":
            text = render_channel(mapping)
        else:
            type_name = mapping.get("unified_type")
            if type_name not in defs:
                print(f"skip {p.name}: openpx schema has no `{type_name}`", file=sys.stderr)
                continue
            text = render_table(mapping, defs[type_name])

        out_path.write_text(text)
        rel = out_path.relative_to(ROOT)
        print(f"wrote {rel} ({len(text):,} bytes)")
        written += 1
    print(f"\n{written} mapping page(s) rendered")
    return 0


if __name__ == "__main__":
    sys.exit(main())
