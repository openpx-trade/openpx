#!/usr/bin/env python3
"""Generate `schema/openpx.openapi.yaml` from the Rust Exchange trait + JSON Schema.

Sources of truth:
  - engine/core/src/exchange/traits.rs   -> method signatures + doc comments
  - schema/openpx.schema.json            -> all unified type definitions (Market,
                                            Order, FetchMarketsParams, ...)

Output: a self-contained OpenAPI 3.1.0 spec that Mintlify can auto-render into
the API tab. Models OpenPX trait methods as `POST /{operation_id}` operations
under tags that match the existing API navigation grouping. All component
schemas come from openpx.schema.json (single source of truth); tuple/list
return wrappers (e.g. `(Vec<Market>, Option<String>)`) are synthesized as
`*Page` schemas.

Run via `just openapi`. CI's sdk-sync job regenerates and diffs.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parent.parent
TRAITS_RS = ROOT / "engine" / "core" / "src" / "exchange" / "traits.rs"
JSON_SCHEMA = ROOT / "schema" / "openpx.schema.json"
CARGO_TOML = ROOT / "Cargo.toml"
# Mintlify auto-discovers OpenAPI specs inside docs/. Putting it there lets us
# reference it from docs.json by name (no `..` traversal) and lets Mintlify
# auto-generate one page per operation.
OUTPUT = ROOT / "docs" / "openpx.openapi.yaml"

# Per-method tag + HTTP verb. Verb is purely cosmetic — Mintlify renders it as
# a badge — but we pick semantically: reads are GET, writes are POST. Tag
# placement mirrors the docs/api/<tag-slug>/ filesystem layout.
METHOD_META: dict[str, tuple[str, str]] = {
    "fetch_markets": ("Markets", "GET"),
    "fetch_market_lineage": ("Markets", "GET"),
    "fetch_market_tags": ("Tags", "GET"),
    "fetch_orderbook": ("Orderbook", "GET"),
    "fetch_orderbooks_batch": ("Orderbook", "GET"),
    "fetch_midpoint": ("Pricing", "GET"),
    "fetch_midpoints_batch": ("Pricing", "GET"),
    "fetch_spread": ("Pricing", "GET"),
    "fetch_last_trade_price": ("Pricing", "GET"),
    "fetch_open_interest": ("Pricing", "GET"),
    "fetch_trades": ("Trades", "GET"),
    "create_order": ("Orders", "POST"),
    "create_orders_batch": ("Orders", "POST"),
    "cancel_order": ("Orders", "POST"),
    "cancel_all_orders": ("Orders", "POST"),
    "fetch_order": ("Orders", "GET"),
    "fetch_open_orders": ("Orders", "GET"),
    "fetch_positions": ("Positions", "GET"),
    "fetch_balance": ("Balance", "GET"),
    "refresh_balance": ("Balance", "POST"),
    "fetch_fills": ("Fills", "GET"),
    "fetch_server_time": ("Server", "GET"),
}

METHOD_TAGS: dict[str, str] = {k: v[0] for k, v in METHOD_META.items()}


def tag_slug(tag: str) -> str:
    return tag.lower().replace(" ", "-")

# Methods we deliberately skip — `id`, `name`, `describe`, `manifest` are not
# trade-side primitives and don't model cleanly as REST operations.
SKIP_METHODS = {"id", "name", "describe", "manifest"}


def read_workspace_version() -> str:
    text = CARGO_TOML.read_text()
    m = re.search(
        r"\[workspace\.package\][^\[]*?^version\s*=\s*\"([^\"]+)\"",
        text,
        re.MULTILINE | re.DOTALL,
    )
    return m.group(1) if m else "0.0.0"


# ---------------------------------------------------------------------------
# Rust trait parser
# ---------------------------------------------------------------------------

# Match each method signature in traits.rs. The trait is well-formatted; we
# capture preceding doc-comment block, optional `async`, name, parenthesized
# args (multi-line), and the return type up to the first `;` or `{`.
METHOD_RE = re.compile(
    r"((?:^[ \t]*///[^\n]*\n)*)"        # doc comments (group 1)
    r"^[ \t]*(async\s+)?fn\s+"           # async marker (group 2)
    r"([a-zA-Z_][a-zA-Z0-9_]*)"          # name (group 3)
    r"\s*\((.*?)\)"                      # args (group 4)
    r"\s*(?:->\s*(.*?))?"                # return type (group 5)
    r"\s*[;{]",                          # terminator
    re.MULTILINE | re.DOTALL,
)


def extract_doc(block: str) -> str:
    """Strip /// markers from a doc-comment block; collapse to one line where possible."""
    lines: list[str] = []
    for line in block.splitlines():
        line = line.strip()
        if line.startswith("///"):
            lines.append(line[3:].strip())
    return "\n".join(lines).strip()


def parse_args(args_blob: str) -> list[tuple[str, str]]:
    """Parse a Rust args list into [(name, type)]. Skips `&self` / `&mut self`."""
    args_blob = args_blob.strip()
    if not args_blob:
        return []
    # Split on commas at depth 0 (don't split inside generics).
    parts: list[str] = []
    depth = 0
    cur: list[str] = []
    for ch in args_blob:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if cur:
        parts.append("".join(cur).strip())

    out: list[tuple[str, str]] = []
    for p in parts:
        p = p.strip()
        if not p or p in ("&self", "&mut self", "self"):
            continue
        if ":" not in p:
            continue
        name, ty = p.split(":", 1)
        out.append((name.strip(), ty.strip()))
    return out


def parse_traits(text: str) -> list[dict[str, Any]]:
    """Return the list of trait methods that should become OpenAPI operations."""
    methods: list[dict[str, Any]] = []
    for m in METHOD_RE.finditer(text):
        name = m.group(3)
        if name in SKIP_METHODS:
            continue
        if name not in METHOD_META:
            continue
        doc = extract_doc(m.group(1) or "")
        args = parse_args(m.group(4) or "")
        ret = (m.group(5) or "").strip().rstrip(";").strip()
        tag, verb = METHOD_META[name]
        methods.append(
            {
                "name": name,
                "doc": doc,
                "args": args,
                "ret": ret,
                "tag": tag,
                "verb": verb,
            }
        )
    return methods


# ---------------------------------------------------------------------------
# Rust type → OpenAPI schema
# ---------------------------------------------------------------------------

PRIMITIVE_TYPES: dict[str, dict[str, Any]] = {
    "String": {"type": "string"},
    "&str": {"type": "string"},
    "str": {"type": "string"},
    "bool": {"type": "boolean"},
    "f64": {"type": "number", "format": "double"},
    "f32": {"type": "number", "format": "float"},
    "i64": {"type": "integer", "format": "int64"},
    "i32": {"type": "integer", "format": "int32"},
    "u64": {"type": "integer", "format": "int64", "minimum": 0},
    "u32": {"type": "integer", "format": "int32", "minimum": 0},
    "usize": {"type": "integer", "minimum": 0},
    "()": {"type": "object", "description": "Empty success — no return body."},
    "Value": {"description": "Arbitrary JSON value (raw upstream response)."},
    "DateTime<Utc>": {"type": "string", "format": "date-time"},
}


def strip_ref(t: str) -> str:
    """Drop borrows and `mut`."""
    return t.replace("&mut ", "&").lstrip("&").strip()


def split_generic(t: str) -> tuple[str, list[str]]:
    """`HashMap<String, f64>` -> (`HashMap`, [`String`, `f64`])."""
    t = t.strip()
    if "<" not in t:
        return t, []
    head, rest = t.split("<", 1)
    if not rest.endswith(">"):
        return t, []
    inner = rest[:-1]
    args: list[str] = []
    depth = 0
    cur: list[str] = []
    for ch in inner:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            args.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if cur:
        args.append("".join(cur).strip())
    return head.strip(), args


def split_tuple(t: str) -> list[str]:
    """`(Vec<Market>, Option<String>)` -> [`Vec<Market>`, `Option<String>`]."""
    t = t.strip()
    if not (t.startswith("(") and t.endswith(")")):
        return []
    inner = t[1:-1]
    parts: list[str] = []
    depth = 0
    cur: list[str] = []
    for ch in inner:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if cur:
        parts.append("".join(cur).strip())
    return parts


def rust_to_schema(rust: str, schema_defs: set[str]) -> dict[str, Any]:
    """Translate a Rust type to an OpenAPI schema fragment.

    Resolves Result<T, E> -> T (errors are handled by the response 'default').
    Tuple types are rejected — caller must synthesize a wrapper schema.
    """
    rust = strip_ref(rust)
    head, args = split_generic(rust)

    if head == "Result":
        return rust_to_schema(args[0], schema_defs)
    if head == "Option":
        inner = rust_to_schema(args[0], schema_defs)
        # OpenAPI 3.1: x | null
        if "$ref" in inner:
            return {"oneOf": [inner, {"type": "null"}]}
        if "type" in inner and inner["type"] != "null":
            t = inner["type"]
            new = dict(inner)
            new["type"] = [t, "null"] if isinstance(t, str) else list(t) + ["null"]
            return new
        return {"oneOf": [inner, {"type": "null"}]}
    if head == "Vec":
        return {"type": "array", "items": rust_to_schema(args[0], schema_defs)}
    if head == "HashMap":
        if len(args) == 2:
            return {
                "type": "object",
                "additionalProperties": rust_to_schema(args[1], schema_defs),
            }
        return {"type": "object", "additionalProperties": True}

    if rust in PRIMITIVE_TYPES:
        return dict(PRIMITIVE_TYPES[rust])

    if rust.startswith("(") and rust != "()":
        raise ValueError(f"tuple type {rust!r} requires a wrapper schema")

    bare = rust.replace(" ", "")
    if bare in schema_defs:
        return {"$ref": f"#/components/schemas/{bare}"}

    return {"description": f"Unmapped Rust type: `{rust}`"}


# ---------------------------------------------------------------------------
# Wrapper schemas for tuple returns
# ---------------------------------------------------------------------------

def page_wrapper_name(item_rust: str) -> str:
    """Vec<Market> -> MarketsPage; Vec<MarketTrade> -> MarketTradesPage."""
    _, args = split_generic(item_rust)
    if not args:
        return f"{item_rust}Page"
    inner = args[0].strip()
    plural = inner if inner.endswith("s") else inner + "s"
    return f"{plural}Page"


def synthesize_response_wrapper(
    method_name: str, ret: str, schema_defs: set[str]
) -> tuple[str, dict[str, Any]] | None:
    """If the return is a tuple like `(Vec<X>, Option<String>)`, build a *Page
    schema. Returns (component_name, schema) or None."""
    parts = split_tuple(strip_ref(ret).removeprefix("Result<").rstrip(">").split(",")[0])
    # Re-parse properly: the input is the full Result<...>; we want the inner.
    inner_ret = strip_ref(ret)
    head, args = split_generic(inner_ret)
    if head == "Result":
        inner_ret = args[0]
    parts = split_tuple(inner_ret)
    if not parts or len(parts) != 2:
        return None
    items_part, cursor_part = parts
    items_head, items_args = split_generic(items_part)
    if items_head != "Vec" or not items_args:
        return None

    name = page_wrapper_name(items_part)
    item_schema = rust_to_schema(items_args[0], schema_defs)
    items_field_name = "items"
    inner_name = items_args[0].strip()
    if inner_name == "Market":
        items_field_name = "markets"
    elif inner_name == "Event":
        items_field_name = "events"
    elif inner_name == "Series":
        items_field_name = "series"
    elif inner_name == "MarketTrade":
        items_field_name = "trades"
    elif inner_name == "Order":
        items_field_name = "orders"

    schema = {
        "type": "object",
        "description": (
            f"One page of `{inner_name}` results. `cursor` is `null` on the "
            "last page; otherwise pass it back in the next request to continue."
        ),
        "required": [items_field_name],
        "properties": {
            items_field_name: {"type": "array", "items": item_schema},
            "cursor": {"type": ["string", "null"]},
        },
    }
    return name, schema


# ---------------------------------------------------------------------------
# Operation builder
# ---------------------------------------------------------------------------

def build_request_body(
    method: dict[str, Any], schema_defs: set[str]
) -> dict[str, Any] | None:
    """Build a requestBody schema. If the method has exactly one struct arg,
    use that struct's schema directly (no extra wrapping)."""
    args = method["args"]
    if not args:
        return None

    if len(args) == 1:
        _, ty = args[0]
        ty_stripped = strip_ref(ty)
        head, inner = split_generic(ty_stripped)
        if head == "Option" and inner:
            ty_stripped = inner[0]
        bare = ty_stripped.replace(" ", "")
        if bare in schema_defs:
            return {
                "required": True,
                "content": {
                    "application/json": {
                        "schema": {"$ref": f"#/components/schemas/{bare}"}
                    }
                },
            }

    properties: dict[str, dict[str, Any]] = {}
    required: list[str] = []
    for name, ty in args:
        ty_stripped = strip_ref(ty)
        head, _ = split_generic(ty_stripped)
        schema = rust_to_schema(ty_stripped, schema_defs)
        properties[name] = schema
        if head != "Option":
            required.append(name)
    body_schema: dict[str, Any] = {"type": "object", "properties": properties}
    if required:
        body_schema["required"] = required
    return {
        "required": True,
        "content": {"application/json": {"schema": body_schema}},
    }


def build_query_parameters(
    method: dict[str, Any],
    schema_defs: set[str],
    raw_defs: dict[str, Any],
) -> list[dict[str, Any]]:
    """Build OpenAPI `parameters` (in: query) for GET methods.

    A single struct arg (e.g. `params: &FetchMarketsParams`) is expanded so
    each struct field becomes its own query parameter — the natural REST
    representation of "fetch markets matching these filters". Primitive args
    map 1:1 to a query parameter."""
    args = method["args"]
    if not args:
        return []

    if len(args) == 1:
        _, ty = args[0]
        ty_stripped = strip_ref(ty)
        head, inner = split_generic(ty_stripped)
        if head == "Option" and inner:
            ty_stripped = inner[0]
        bare = ty_stripped.replace(" ", "")
        if bare in schema_defs:
            node = raw_defs.get(bare, {})
            props = node.get("properties") or {}
            required = set(node.get("required") or [])
            if props:
                params_list: list[dict[str, Any]] = []
                for prop_name, prop in props.items():
                    prop_copy = dict(prop)
                    desc = prop_copy.pop("description", None)
                    schema = normalize_jsonschema_for_openapi(prop_copy)
                    param: dict[str, Any] = {
                        "name": prop_name,
                        "in": "query",
                        "required": prop_name in required,
                        "schema": schema,
                    }
                    if desc:
                        param["description"] = desc
                    params_list.append(param)
                return params_list

    params_list = []
    for arg_name, ty in args:
        ty_stripped = strip_ref(ty)
        head, inner = split_generic(ty_stripped)
        is_optional = head == "Option"
        if is_optional and inner:
            ty_stripped = inner[0]
        schema = rust_to_schema(ty_stripped, schema_defs)
        params_list.append({
            "name": arg_name,
            "in": "query",
            "required": not is_optional,
            "schema": schema,
        })
    return params_list


def build_response_schema(
    method: dict[str, Any], schema_defs: set[str], wrappers: dict[str, dict[str, Any]]
) -> dict[str, Any]:
    """Translate the method's return type into a 200-response schema."""
    ret = method["ret"] or ""
    if not ret:
        return {"type": "object"}

    wrap = synthesize_response_wrapper(method["name"], ret, schema_defs)
    if wrap is not None:
        name, schema = wrap
        wrappers.setdefault(name, schema)
        return {"$ref": f"#/components/schemas/{name}"}

    inner_ret = strip_ref(ret)
    head, args = split_generic(inner_ret)
    if head == "Result" and args:
        return rust_to_schema(args[0], schema_defs)
    return rust_to_schema(inner_ret, schema_defs)


def build_operation(
    method: dict[str, Any],
    schema_defs: set[str],
    wrappers: dict[str, dict[str, Any]],
    raw_defs: dict[str, Any],
) -> dict[str, Any]:
    """Compose an OpenAPI operation. Key ordering follows the canonical layout
    (tags → summary → description → operationId → parameters → requestBody →
    responses) — Mintlify's renderer skips the parameters panel when keys are
    out of canonical order."""
    name = method["name"]
    doc = method["doc"]
    summary = name.replace("_", " ").capitalize()
    description = doc or ""
    op: dict[str, Any] = {
        "tags": [method["tag"]],
        "summary": summary,
    }
    if description:
        op["description"] = description
    op["operationId"] = name

    if method["verb"] == "GET":
        params = build_query_parameters(method, schema_defs, raw_defs)
        if params:
            op["parameters"] = params
    else:
        body = build_request_body(method, schema_defs)
        if body:
            op["requestBody"] = body

    op["responses"] = {
        "200": {
            "description": "Success.",
            "content": {
                "application/json": {
                    "schema": build_response_schema(method, schema_defs, wrappers)
                }
            },
        },
        "default": {
            "description": "Unified error response.",
            "content": {
                "application/json": {
                    "schema": {"$ref": "#/components/schemas/OpenPxError"}
                }
            },
        },
    }
    return op


# ---------------------------------------------------------------------------
# OpenAPI document assembly
# ---------------------------------------------------------------------------

ERROR_SCHEMA: dict[str, Any] = {
    "type": "object",
    "description": (
        "Unified error type returned when an exchange call fails. The `kind` "
        "field is the canonical category; `message` carries the upstream "
        "explanation. Per-exchange variants are mapped into this shape before "
        "they cross the trait boundary."
    ),
    "required": ["kind", "message"],
    "properties": {
        "kind": {
            "type": "string",
            "enum": [
                "Authentication",
                "InsufficientFunds",
                "MarketNotFound",
                "OrderNotFound",
                "RateLimited",
                "NetworkError",
                "Api",
                "Config",
                "NotSupported",
            ],
        },
        "message": {"type": "string"},
    },
}


def normalize_jsonschema_for_openapi(node: Any) -> Any:
    """JSON Schema dialects differ from OpenAPI 3.1 in two ways relevant here:
    - `$ref` paths use `#/definitions/X`; OpenAPI uses `#/components/schemas/X`.
    - `type: ['string', 'null']` is OpenAPI 3.1-compatible already; pass through.
    """
    if isinstance(node, dict):
        out: dict[str, Any] = {}
        for k, v in node.items():
            if k == "$ref" and isinstance(v, str) and v.startswith("#/definitions/"):
                out[k] = v.replace("#/definitions/", "#/components/schemas/", 1)
            else:
                out[k] = normalize_jsonschema_for_openapi(v)
        return out
    if isinstance(node, list):
        return [normalize_jsonschema_for_openapi(x) for x in node]
    return node


def main() -> int:
    schema = json.loads(JSON_SCHEMA.read_text())
    raw_defs: dict[str, Any] = schema.get("definitions") or schema.get("$defs") or {}
    schema_def_names = set(raw_defs.keys())

    components: dict[str, Any] = {
        name: normalize_jsonschema_for_openapi(body)
        for name, body in raw_defs.items()
    }
    components["OpenPxError"] = ERROR_SCHEMA

    methods = parse_traits(TRAITS_RS.read_text())
    if not methods:
        sys.exit(f"no operations parsed from {TRAITS_RS}")

    wrappers: dict[str, dict[str, Any]] = {}
    paths: dict[str, dict[str, Any]] = {}
    for m in methods:
        op = build_operation(m, schema_def_names, wrappers, raw_defs)
        path = f"/v1/{tag_slug(m['tag'])}/{m['name']}"
        verb = m["verb"].lower()
        paths.setdefault(path, {})[verb] = op

    components.update(wrappers)

    tags_seen: list[str] = []
    seen: set[str] = set()
    for m in methods:
        if m["tag"] not in seen:
            tags_seen.append(m["tag"])
            seen.add(m["tag"])

    spec: dict[str, Any] = {
        "openapi": "3.1.0",
        "info": {
            "title": "OpenPX Unified API",
            "description": (
                "Unified API surface across prediction-market exchanges. One "
                "`Exchange` trait, one set of models, one shape — implemented "
                "in Rust with auto-generated Python and TypeScript SDKs.\n\n"
                "This OpenAPI spec is **auto-generated** from "
                "`engine/core/src/exchange/traits.rs` and "
                "`schema/openpx.schema.json`. Edit the trait or the schema, "
                "then regenerate via `just openapi`. The spec models trait "
                "methods as REST operations for documentation rendering only "
                "— OpenPX is an in-process Rust library, not a REST service."
            ),
            "version": read_workspace_version(),
            "license": {"name": "MIT"},
        },
        "servers": [
            {
                "url": "https://github.com/openpx-trade/openpx",
                "description": (
                    "Documentation surface only. OpenPX is an in-process "
                    "Rust library; there is no hosted REST endpoint."
                ),
            }
        ],
        "tags": [{"name": t} for t in tags_seen],
        "paths": paths,
        "components": {"schemas": components},
    }

    OUTPUT.write_text(yaml.safe_dump(spec, sort_keys=False, width=120))
    rel = OUTPUT.relative_to(ROOT)
    print(
        f"wrote {rel} ({OUTPUT.stat().st_size:,} bytes, "
        f"{len(methods)} operations, {len(components)} schemas)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
