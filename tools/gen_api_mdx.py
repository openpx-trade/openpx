#!/usr/bin/env python3
"""Generate `docs/api/<tag>/<method>.mdx` for every Exchange-trait method.

Replaces the previous attempt to lean on Mintlify's tab-level OpenAPI
auto-render — that produced a different visual style than the project's prior
hand-crafted `<ParamField>` / `<ResponseField>` / `<RequestExample>` MDX. This
script re-creates that exact visual but auto-derives every cell from the
single source of truth.

Sources:
  - engine/core/src/exchange/traits.rs   -> trait method signatures + doc
                                            comments + per-arg `let _ = ...`
                                            patterns are ignored.
  - schema/openpx.schema.json            -> all type definitions, with
                                            descriptions, used to expand
                                            request and response shapes.
  - docs/openpx.openapi.yaml             -> read for verb + path metadata so
                                            the `openapi:` frontmatter badge
                                            resolves cleanly.

Per-method output (one MDX file each):
  ---
  title: "<method>"
  sidebarTitle: "<Humanized method>"
  openapi: <VERB> <path>
  playground: simple
  description: "<first sentence of doc comment>"
  ---

  ## Parameters
    <ParamField ...>...</ParamField>
    ...

  ## Returns
    <ResponseField ...><Expandable>...</Expandable></ResponseField>
    ...

  <RequestExample>
    ```rust Rust
    ...
    ```
    ```python Python
    ...
    ```
    ```typescript TypeScript
    ...
    ```
  </RequestExample>

  <ResponseExample>
    ```json 200
    { ... }
    ```
  </ResponseExample>
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
OPENAPI_YAML = ROOT / "docs" / "openpx.openapi.yaml"
OUTPUT_DIR = ROOT / "docs" / "api"

# Per-method tag + HTTP verb. Verb is cosmetic — Mintlify renders it as a
# badge — but we pick semantically: reads are GET, writes are POST.
METHOD_META: dict[str, tuple[str, str]] = {
    "fetch_markets": ("Markets", "GET"),
    "fetch_market": ("Markets", "GET"),
    "fetch_events": ("Events", "GET"),
    "fetch_event": ("Events", "GET"),
    "fetch_series": ("Series", "GET"),
    "fetch_series_one": ("Series", "GET"),
    "fetch_market_tags": ("Tags", "GET"),
    "fetch_orderbook": ("Orderbook", "GET"),
    "fetch_orderbooks_batch": ("Orderbook", "GET"),
    "fetch_orderbook_history": ("Orderbook History", "GET"),
    "fetch_midpoint": ("Pricing", "GET"),
    "fetch_midpoints_batch": ("Pricing", "GET"),
    "fetch_spread": ("Pricing", "GET"),
    "fetch_last_trade_price": ("Pricing", "GET"),
    "fetch_open_interest": ("Pricing", "GET"),
    "fetch_trades": ("Trades", "GET"),
    "fetch_user_trades": ("Trades", "GET"),
    "fetch_price_history": ("Price History", "GET"),
    "create_order": ("Orders", "POST"),
    "create_orders_batch": ("Orders", "POST"),
    "cancel_order": ("Orders", "POST"),
    "cancel_all_orders": ("Orders", "POST"),
    "fetch_order": ("Orders", "GET"),
    "fetch_open_orders": ("Orders", "GET"),
    "fetch_positions": ("Positions", "GET"),
    "fetch_balance": ("Balance", "GET"),
    "fetch_balance_raw": ("Balance", "GET"),
    "refresh_balance": ("Balance", "POST"),
    "fetch_fills": ("Fills", "GET"),
    "fetch_user_activity": ("Fills", "GET"),
    "fetch_server_time": ("Server", "GET"),
}

PRIMITIVE_DISPLAY: dict[str, str] = {
    "String": "string",
    "&str": "string",
    "str": "string",
    "bool": "boolean",
    "f64": "f64",
    "f32": "f32",
    "i64": "i64",
    "i32": "i32",
    "u64": "u64",
    "u32": "u32",
    "usize": "usize",
    "()": "void",
    "Value": "object",
    "DateTime<Utc>": "DateTime",
}


# ---------------------------------------------------------------------------
# Trait parser (same shape as gen_openapi.py)
# ---------------------------------------------------------------------------

METHOD_RE = re.compile(
    r"((?:^[ \t]*///[^\n]*\n)*)"
    r"^[ \t]*(async\s+)?fn\s+"
    r"([a-zA-Z_][a-zA-Z0-9_]*)"
    r"\s*\((.*?)\)"
    r"\s*(?:->\s*(.*?))?"
    r"\s*[;{]",
    re.MULTILINE | re.DOTALL,
)


def extract_doc(block: str) -> str:
    lines: list[str] = []
    for line in block.splitlines():
        line = line.strip()
        if line.startswith("///"):
            lines.append(line[3:].strip())
    return "\n".join(lines).strip()


def parse_args(args_blob: str) -> list[tuple[str, str]]:
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


def parse_traits() -> list[dict[str, Any]]:
    text = TRAITS_RS.read_text()
    methods: list[dict[str, Any]] = []
    for m in METHOD_RE.finditer(text):
        name = m.group(3)
        if name not in METHOD_META:
            continue
        doc = extract_doc(m.group(1) or "")
        args = parse_args(m.group(4) or "")
        ret = (m.group(5) or "").strip().rstrip(";").strip()
        methods.append({"name": name, "doc": doc, "args": args, "ret": ret})
    return methods


# ---------------------------------------------------------------------------
# Rust type helpers
# ---------------------------------------------------------------------------

def strip_ref(t: str) -> str:
    return t.replace("&mut ", "&").lstrip("&").strip()


def split_generic(t: str) -> tuple[str, list[str]]:
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


def display_type_for_rust(rust: str, schema_defs: set[str]) -> str:
    """Return the human-friendly type label, e.g. `Market[]`, `string?`, `f64`."""
    rust = strip_ref(rust)
    head, args = split_generic(rust)
    if head == "Result" and args:
        return display_type_for_rust(args[0], schema_defs)
    if head == "Option" and args:
        inner = display_type_for_rust(args[0], schema_defs)
        return f"{inner}?" if not inner.endswith("?") else inner
    if head == "Vec" and args:
        return f"{display_type_for_rust(args[0], schema_defs)}[]"
    if head == "HashMap" and len(args) == 2:
        return f"{{{display_type_for_rust(args[0], schema_defs)}: {display_type_for_rust(args[1], schema_defs)}}}"
    if rust in PRIMITIVE_DISPLAY:
        return PRIMITIVE_DISPLAY[rust]
    bare = rust.replace(" ", "")
    if bare in schema_defs:
        return bare
    return rust


def rust_unwrap(rust: str) -> tuple[str, bool]:
    """Strip Result<...> and Option<...> wrappers. Returns (inner_rust, is_optional)."""
    rust = strip_ref(rust)
    head, args = split_generic(rust)
    if head == "Result" and args:
        rust = args[0]
        head, args = split_generic(rust)
    is_optional = False
    if head == "Option" and args:
        rust = args[0]
        is_optional = True
    return rust, is_optional


# ---------------------------------------------------------------------------
# JSON Schema → MDX type display
# ---------------------------------------------------------------------------

def display_type_for_jsonschema(node: dict[str, Any]) -> str:
    """Best-effort type label for a JSON Schema property node.

    Schemars emits Rust enums + types with optional fields as `anyOf` of a
    `$ref` and a `null`. We surface those as `<TypeName>?` so users see the
    enum/struct name rather than a generic 'object?'."""
    if "$ref" in node:
        return node["$ref"].split("/")[-1]
    for combinator in ("allOf", "anyOf", "oneOf"):
        if combinator in node:
            ref_name = None
            has_null = False
            for sub in node[combinator]:
                if not isinstance(sub, dict):
                    continue
                if "$ref" in sub:
                    ref_name = sub["$ref"].split("/")[-1]
                elif sub.get("type") == "null":
                    has_null = True
            if ref_name:
                return f"{ref_name}?" if has_null else ref_name
    t = node.get("type")
    if isinstance(t, list):
        non_null = [x for x in t if x and x != "null"]
        nullable = "null" in t
        if not non_null:
            return "null"
        base = non_null[0]
        if base == "array":
            items = node.get("items", {})
            inner = display_type_for_jsonschema(items)
            return f"{inner}[]" + ("?" if nullable else "")
        return _fmt_primitive(base, node) + ("?" if nullable else "")
    if isinstance(t, str):
        if t == "array":
            items = node.get("items", {})
            return f"{display_type_for_jsonschema(items)}[]"
        return _fmt_primitive(t, node)
    return "object"


def _fmt_primitive(t: str, node: dict[str, Any]) -> str:
    if t == "number":
        fmt = node.get("format")
        if fmt == "double":
            return "f64"
        if fmt == "float":
            return "f32"
        return "number"
    if t == "integer":
        fmt = node.get("format")
        if fmt == "int64":
            return "i64"
        if fmt == "int32":
            return "i32"
        return "integer"
    if t == "string":
        if node.get("format") == "date-time":
            return "DateTime"
        return "string"
    return t


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------

INDENT = "  "


def mdx_escape(s: str) -> str:
    """Make freeform text safe inside JSX-y MDX content. Pipes/braces are the
    common JSX-expression hazards in MDX2."""
    return (s or "").replace("{", "\\{").replace("}", "\\}").strip()


def humanize(name: str) -> str:
    return name.replace("_", " ").capitalize()


def slug(name: str) -> str:
    return name.replace("_", "-")


def tag_dir(tag: str) -> str:
    return tag.lower().replace(" ", "-")


def render_response_field(
    name: str,
    type_label: str,
    description: str,
    indent: str = "",
) -> str:
    desc = mdx_escape(description)
    if desc:
        return f"{indent}<ResponseField name=\"{name}\" type=\"{type_label}\">{desc}</ResponseField>\n"
    return f"{indent}<ResponseField name=\"{name}\" type=\"{type_label}\" />\n"


def render_param_field(
    name: str,
    type_label: str,
    required: bool,
    description: str,
    indent: str = "",
) -> str:
    desc = mdx_escape(description)
    attrs = f"path=\"{name}\" type=\"{type_label}\""
    if required:
        attrs += " required"
    if desc:
        return f"{indent}<ParamField {attrs}>\n{indent}{INDENT}{desc}\n{indent}</ParamField>\n\n"
    return f"{indent}<ParamField {attrs} />\n\n"


def expand_object(
    type_name: str,
    schema_defs: dict[str, Any],
    indent: str,
    visited: set[str],
) -> str:
    """Walk an object schema's properties and emit nested ResponseFields under
    an Expandable. Cycles are guarded by `visited` — recursive types render the
    second occurrence as a non-expanded reference label."""
    if type_name in visited:
        return ""
    visited = visited | {type_name}

    node = schema_defs.get(type_name)
    if not node or not isinstance(node, dict):
        return ""
    props = node.get("properties") or {}
    required = set(node.get("required") or [])
    if not props:
        return ""

    inner = ""
    inner_indent = indent + INDENT
    for prop_name in sorted(props.keys()):
        prop = props[prop_name]
        type_label = display_type_for_jsonschema(prop)
        desc = prop.get("description", "")
        if prop_name not in required:
            if not type_label.endswith("?"):
                type_label += "?"
        inner += render_response_field(prop_name, type_label, desc, inner_indent)

    return inner


def render_response_field_with_expansion(
    name: str,
    type_label: str,
    description: str,
    expand_type_name: str | None,
    schema_defs: dict[str, Any],
    indent: str = "",
    visited: set[str] | None = None,
) -> str:
    """Like render_response_field, but if `expand_type_name` is given and is
    a defined object schema, nest <Expandable> with the object's fields."""
    visited = visited or set()
    if expand_type_name is None or expand_type_name in visited:
        return render_response_field(name, type_label, description, indent)
    expansion = expand_object(expand_type_name, schema_defs, indent + INDENT, visited)
    if not expansion:
        return render_response_field(name, type_label, description, indent)

    desc = mdx_escape(description)
    expand_title = expand_type_name
    open_tag = f"{indent}<ResponseField name=\"{name}\" type=\"{type_label}\">\n"
    open_tag += f"{indent}{INDENT}{desc}\n\n" if desc else "\n"
    open_tag += f"{indent}{INDENT}<Expandable title=\"{expand_title}\">\n"
    close_tag = f"{indent}{INDENT}</Expandable>\n{indent}</ResponseField>\n"
    return open_tag + expansion + close_tag


def returns_section(method: dict[str, Any], schema_defs: dict[str, Any]) -> str:
    """Render the ## Returns section from the trait method's return type."""
    ret = method["ret"]
    if not ret:
        return ""

    inner_ret = strip_ref(ret)
    head, args = split_generic(inner_ret)
    if head == "Result" and args:
        inner_ret = args[0]

    out = "## Returns\n\n"

    # Tuple shape (e.g. `(Vec<Market>, Option<String>)` for paginated lists)
    parts = split_tuple(inner_ret)
    if parts and len(parts) == 2:
        items_part, cursor_part = parts
        items_head, items_args = split_generic(items_part)
        items_field = "items"
        item_type = items_args[0] if items_args else items_part
        if items_head == "Vec":
            inner = items_args[0].strip() if items_args else items_part
            if inner == "Market":
                items_field = "markets"
            elif inner == "Event":
                items_field = "events"
            elif inner == "Series":
                items_field = "series"
            elif inner in ("MarketTrade", "UserTrade"):
                items_field = "trades"
            elif inner == "OrderbookSnapshot":
                items_field = "snapshots"
            elif inner == "Order":
                items_field = "orders"
            type_label = f"{inner}[]"
            out += render_response_field_with_expansion(
                items_field,
                type_label,
                f"One page of `{inner}` results.",
                inner if inner in schema_defs else None,
                schema_defs,
            )
        out += render_response_field(
            "cursor",
            "string?",
            "Opaque pagination cursor. `null` on the last page.",
        )
        return out + "\n"

    # HashMap<String, V>
    head_h, args_h = split_generic(inner_ret)
    if head_h == "HashMap" and len(args_h) == 2:
        v_type = display_type_for_rust(args_h[1], set(schema_defs.keys()))
        out += render_response_field(
            "result",
            f"{{string: {v_type}}}",
            "Map keyed by the input identifier.",
        )
        return out + "\n"

    # Vec<T>
    if head_h == "Vec" and args_h:
        inner = args_h[0].strip()
        type_label = f"{inner}[]"
        if inner in schema_defs:
            out += render_response_field_with_expansion(
                "result",
                type_label,
                f"List of `{inner}` results.",
                inner,
                schema_defs,
            )
        else:
            out += render_response_field(
                "result", type_label, f"List of `{inner}` results."
            )
        return out + "\n"

    # Single primitive or known type
    bare = inner_ret.replace(" ", "").rstrip("?")
    if bare in schema_defs:
        out += render_response_field_with_expansion(
            "result",
            bare,
            f"`{bare}` payload.",
            bare,
            schema_defs,
        )
    else:
        type_label = display_type_for_rust(inner_ret, set(schema_defs.keys()))
        if inner_ret == "()":
            return ""  # void return — no Returns section
        out += render_response_field("result", type_label, "")
    return out + "\n"


def parameters_section(method: dict[str, Any], schema_defs: dict[str, Any]) -> str:
    """Render the ## Parameters section from the trait method's args.

    For a single struct arg (`params: &FetchMarketsParams`), expand the
    struct's fields into individual ParamFields. Otherwise emit one
    ParamField per arg.
    """
    args = method["args"]
    if not args:
        return ""

    # If exactly one arg of a custom struct type, expand its fields.
    if len(args) == 1:
        _, ty = args[0]
        ty_unwrapped, _ = rust_unwrap(ty)
        bare = ty_unwrapped.replace(" ", "")
        if bare in schema_defs:
            node = schema_defs[bare]
            props = node.get("properties") or {}
            if props:
                out = "## Parameters\n\n"
                required = set(node.get("required") or [])
                for prop_name in sorted(props.keys()):
                    prop = props[prop_name]
                    type_label = display_type_for_jsonschema(prop)
                    is_required = prop_name in required
                    if not is_required and not type_label.endswith("?"):
                        type_label += "?"
                    desc = prop.get("description", "")
                    out += render_param_field(
                        prop_name, type_label, is_required, desc
                    )
                return out

    # Multiple args, or non-struct single arg: emit one ParamField each.
    out = "## Parameters\n\n"
    for name, ty in args:
        ty_unwrapped, is_opt = rust_unwrap(ty)
        type_label = display_type_for_rust(ty_unwrapped, set(schema_defs.keys()))
        if is_opt and not type_label.endswith("?"):
            type_label += "?"
        out += render_param_field(name, type_label, not is_opt, "")
    return out


# ---------------------------------------------------------------------------
# Examples
# ---------------------------------------------------------------------------

def python_method_name(name: str) -> str:
    return name


def ts_method_name(name: str) -> str:
    parts = name.split("_")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def render_request_example(method: dict[str, Any]) -> str:
    name = method["name"]
    args = method["args"]
    arg_names = [a[0] for a in args]

    # Rust: build a no-frills `let result = ex.<method>(<args>).await?;` line.
    # Real example values are domain-specific; we emit placeholders.
    rust_args: list[str] = []
    for arg_name, ty in args:
        ty_u, _ = rust_unwrap(ty)
        if "&str" in ty or ty_u in ("String",):
            rust_args.append(f'"{arg_name}-example"')
        elif "Vec<" in ty:
            rust_args.append("vec![]")
        elif ty_u in ("f64", "f32"):
            rust_args.append("0.5")
        elif ty_u in ("i64", "i32", "u64", "u32", "usize"):
            rust_args.append("0")
        elif ty_u == "bool":
            rust_args.append("false")
        elif ty.startswith("&"):
            rust_args.append(f"&{arg_name}")
        else:
            rust_args.append(f"{arg_name}")

    rust_call = f"let result = ex.{name}({', '.join(rust_args)}).await?;"

    py_args = ", ".join(arg_names)
    py_call = f"result = ex.{python_method_name(name)}({py_args})"

    ts_args = ", ".join(arg_names)
    ts_call = f"const result = await ex.{ts_method_name(name)}({ts_args});"

    return (
        "<RequestExample>\n\n"
        f"```rust Rust\n{rust_call}\n```\n\n"
        f"```python Python\n{py_call}\n```\n\n"
        f"```typescript TypeScript\n{ts_call}\n```\n\n"
        "</RequestExample>\n"
    )


def render_response_example(method: dict[str, Any], schema_defs: dict[str, Any]) -> str:
    """Emit a JSON 200 example. We sample one representative shape — full
    realism would need fixtures; this surfaces the schema convincingly enough
    for docs."""
    ret = method["ret"]
    if not ret:
        return ""
    inner_ret = strip_ref(ret)
    head, args = split_generic(inner_ret)
    if head == "Result" and args:
        inner_ret = args[0]

    sample = sample_value_for_rust(inner_ret, schema_defs, depth=0)
    if sample is None:
        return ""
    body = json.dumps(sample, indent=2)
    return (
        "<ResponseExample>\n\n"
        f"```json 200\n{body}\n```\n\n"
        "</ResponseExample>\n"
    )


def sample_value_for_rust(
    rust: str, schema_defs: dict[str, Any], depth: int
) -> Any:
    """Build a placeholder JSON value for a Rust type. Truncates recursion to
    avoid blowing up on cyclic types."""
    if depth > 3:
        return None
    rust = strip_ref(rust)
    head, args = split_generic(rust)
    if head == "Result" and args:
        return sample_value_for_rust(args[0], schema_defs, depth)
    if head == "Option" and args:
        return sample_value_for_rust(args[0], schema_defs, depth)
    if head == "Vec" and args:
        v = sample_value_for_rust(args[0], schema_defs, depth + 1)
        return [v] if v is not None else []
    if head == "HashMap" and len(args) == 2:
        return {"<key>": sample_value_for_rust(args[1], schema_defs, depth + 1)}
    if rust.startswith("("):
        parts = split_tuple(rust)
        if len(parts) == 2:
            items_head, items_args = split_generic(parts[0])
            if items_head == "Vec" and items_args:
                inner = items_args[0]
                items_field = _items_field_for(inner)
                return {
                    items_field: sample_value_for_rust(parts[0], schema_defs, depth + 1),
                    "cursor": None,
                }
        return None
    if rust in PRIMITIVE_DISPLAY:
        return _sample_primitive(rust)
    if rust in schema_defs:
        return sample_object(rust, schema_defs, depth + 1)
    return None


def _items_field_for(rust: str) -> str:
    return {
        "Market": "markets",
        "Event": "events",
        "Series": "series",
        "MarketTrade": "trades",
        "UserTrade": "trades",
        "OrderbookSnapshot": "snapshots",
        "Order": "orders",
    }.get(rust, "items")


def _sample_primitive(rust: str) -> Any:
    if rust in ("String", "&str", "str"):
        return "example"
    if rust == "bool":
        return True
    if rust in ("f64", "f32"):
        return 0.5
    if rust in ("i64", "i32", "u64", "u32", "usize"):
        return 0
    if rust == "DateTime<Utc>":
        return "2026-01-01T00:00:00Z"
    if rust == "()":
        return None
    if rust == "Value":
        return {}
    return None


def sample_object(type_name: str, schema_defs: dict[str, Any], depth: int) -> Any:
    if depth > 3:
        return {}
    node = schema_defs.get(type_name)
    if not isinstance(node, dict):
        return {}
    if "enum" in node:
        return node["enum"][0] if node["enum"] else None
    props = node.get("properties") or {}
    if not props:
        return {}
    out: dict[str, Any] = {}
    for k, v in props.items():
        out[k] = _sample_jsonschema_value(v, schema_defs, depth + 1)
    return out


def _sample_jsonschema_value(node: dict[str, Any], schema_defs: dict[str, Any], depth: int) -> Any:
    if depth > 4:
        return None
    if "$ref" in node:
        ref_name = node["$ref"].split("/")[-1]
        return sample_object(ref_name, schema_defs, depth + 1)
    for combinator in ("allOf", "anyOf", "oneOf"):
        if combinator in node:
            for sub in node[combinator]:
                if isinstance(sub, dict) and "$ref" in sub:
                    return sample_object(sub["$ref"].split("/")[-1], schema_defs, depth + 1)
    t = node.get("type")
    if isinstance(t, list):
        non_null = [x for x in t if x and x != "null"]
        if not non_null:
            return None
        t = non_null[0]
    if t == "string":
        if node.get("format") == "date-time":
            return "2026-01-01T00:00:00Z"
        return node.get("examples", ["example"])[0] if node.get("examples") else "example"
    if t == "integer":
        return 0
    if t == "number":
        return 0.5
    if t == "boolean":
        return True
    if t == "array":
        items = node.get("items", {})
        return [_sample_jsonschema_value(items, schema_defs, depth + 1)]
    if t == "object":
        return {}
    if "enum" in node:
        return node["enum"][0]
    return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def render_method_mdx(method: dict[str, Any], schema_defs: dict[str, Any]) -> str:
    name = method["name"]
    tag, verb = METHOD_META[name]
    doc = method["doc"]
    description_line = doc.split("\n", 1)[0] if doc else humanize(name)
    body_doc_lines = doc.split("\n", 1)[1].strip() if "\n" in doc else ""

    fm_path = f"/v1/{tag_dir(tag)}/{name}"

    # YAML frontmatter values: JSON-encode strings to safely escape quotes,
    # backslashes, and other characters. JSON strings are valid YAML scalars.
    out = "---\n"
    out += f"title: {json.dumps(name)}\n"
    out += f"sidebarTitle: {json.dumps(humanize(name))}\n"
    out += f"openapi: {verb} {fm_path}\n"
    out += "playground: simple\n"
    out += f"description: {json.dumps(description_line)}\n"
    out += "---\n\n"

    if body_doc_lines:
        out += body_doc_lines + "\n\n"

    out += parameters_section(method, schema_defs)
    out += returns_section(method, schema_defs)
    out += render_request_example(method)
    out += render_response_example(method, schema_defs)
    return out


def main() -> int:
    schema = json.loads(JSON_SCHEMA.read_text())
    schema_defs: dict[str, Any] = (
        schema.get("definitions") or schema.get("$defs") or {}
    )

    methods = parse_traits()
    if not methods:
        sys.exit("no methods parsed from traits.rs")

    written = 0
    paths_by_tag: dict[str, list[str]] = {}
    for method in methods:
        name = method["name"]
        tag, _ = METHOD_META[name]
        out_path = OUTPUT_DIR / tag_dir(tag) / f"{slug(name)}.mdx"
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(render_method_mdx(method, schema_defs))
        rel = out_path.relative_to(ROOT)
        paths_by_tag.setdefault(tag, []).append(
            f"api/{tag_dir(tag)}/{slug(name)}"
        )
        written += 1

    print(f"wrote {written} MDX files under {OUTPUT_DIR.relative_to(ROOT)}/")
    print("\nPage paths by tag (paste into docs.json):")
    for tag in sorted(paths_by_tag):
        for p in sorted(paths_by_tag[tag]):
            print(f"  {tag:20s} {p}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
