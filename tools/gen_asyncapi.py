#!/usr/bin/env python3
"""Generate `docs/openpx.asyncapi.yaml` from the unified WS types in
`schema/openpx.schema.json`.

Models OpenPX's WebSocket abstraction as a single AsyncAPI 3.0 channel —
unified market-data + session-event stream. Each `WsUpdate` and
`SessionEvent` variant becomes a `receive`-action message; client-side
`subscribe` / `unsubscribe` calls become `send`-action messages so the
docs site renders both panels (Polymarket-style) without us hand-writing
either side.

OpenPX is an in-process Rust library, not a real WebSocket service. The
spec models the message shapes for documentation only; the `servers`
block calls this out explicitly so users aren't misled.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parent.parent
JSON_SCHEMA = ROOT / "schema" / "openpx.schema.json"
CARGO_TOML = ROOT / "Cargo.toml"
OUTPUT_YAML = ROOT / "docs" / "openpx.asyncapi.yaml"
OUTPUT_MDX = ROOT / "docs" / "websockets" / "stream.mdx"

# Unified-stream variants we surface in the AsyncAPI. Each entry is the
# discriminator value (kind) on the JSON Schema oneOf variant.
WS_UPDATE_VARIANTS = ["Snapshot", "Delta", "Clear", "Trade", "Fill"]
SESSION_VARIANTS = ["Connected", "Reconnected", "Lagged", "BookInvalidated", "Error"]


def read_workspace_version() -> str:
    text = CARGO_TOML.read_text()
    m = re.search(
        r"\[workspace\.package\][^\[]*?^version\s*=\s*\"([^\"]+)\"",
        text,
        re.MULTILINE | re.DOTALL,
    )
    return m.group(1) if m else "0.0.0"


def normalize_jsonschema(node: Any) -> Any:
    """JSON Schema $ref `#/definitions/X` → AsyncAPI/OpenAPI `#/components/schemas/X`."""
    if isinstance(node, dict):
        out: dict[str, Any] = {}
        for k, v in node.items():
            if k == "$ref" and isinstance(v, str) and v.startswith("#/definitions/"):
                out[k] = v.replace("#/definitions/", "#/components/schemas/", 1)
            else:
                out[k] = normalize_jsonschema(v)
        return out
    if isinstance(node, list):
        return [normalize_jsonschema(x) for x in node]
    return node


def find_variant(union_node: dict[str, Any], kind: str) -> dict[str, Any] | None:
    """Pull the variant schema out of a tagged-union (oneOf with `kind` const)."""
    for variant in union_node.get("oneOf", []):
        kind_node = variant.get("properties", {}).get("kind", {})
        const = kind_node.get("const")
        enum_vals = kind_node.get("enum") or ([const] if const else [])
        if kind in enum_vals:
            return variant
    return None


def extract_variant_schema(union_name: str, kind: str, schema_defs: dict[str, Any]) -> dict[str, Any]:
    """Return a standalone JSON Schema for a tagged-union variant. Drops the
    `kind` field from the published schema since it's the dispatch tag — users
    pattern-match on the variant type, they don't read the string."""
    variant = find_variant(schema_defs[union_name], kind)
    if variant is None:
        sys.exit(f"variant `{kind}` not found in `{union_name}`")
    cleaned = {k: v for k, v in variant.items() if k != "title"}
    return normalize_jsonschema(cleaned)


def build_asyncapi(schema_defs: dict[str, Any]) -> dict[str, Any]:
    components_schemas: dict[str, Any] = {}
    components_messages: dict[str, Any] = {}

    # Pull every variant out as its own schema.
    variant_schemas: dict[str, Any] = {}
    for kind in WS_UPDATE_VARIANTS:
        variant_schemas[kind] = extract_variant_schema("WsUpdate", kind, schema_defs)
    for kind in SESSION_VARIANTS:
        variant_schemas[kind] = extract_variant_schema("SessionEvent", kind, schema_defs)

    # Re-publish all canonical schemas referenced from the variants. We include
    # the full definitions block so $refs resolve cleanly inside the AsyncAPI.
    for name, body in schema_defs.items():
        components_schemas[name] = normalize_jsonschema(body)

    for name, schema in variant_schemas.items():
        components_schemas[name] = schema

    # Build the messages.
    def message_for_variant(kind: str, union_name: str) -> dict[str, Any]:
        variant = find_variant(schema_defs[union_name], kind)
        description = (variant or {}).get("description", "").strip()
        return {
            "name": kind,
            "title": kind,
            "summary": description.splitlines()[0] if description else kind,
            "description": description,
            "contentType": "application/json",
            "payload": {"$ref": f"#/components/schemas/{kind}"},
        }

    for kind in WS_UPDATE_VARIANTS:
        components_messages[kind] = message_for_variant(kind, "WsUpdate")
    for kind in SESSION_VARIANTS:
        components_messages[kind] = message_for_variant(kind, "SessionEvent")

    components_messages["Subscribe"] = {
        "name": "Subscribe",
        "title": "Subscribe",
        "summary": "Add a market to the active subscription set",
        "description": (
            "Call `WebSocket.subscribe(market_id)` after the stream is "
            "connected to receive updates for that market. Subscriptions are "
            "additive — call multiple times to follow more markets. "
            "OpenPX translates this into the per-exchange subscription "
            "protocol under the hood."
        ),
        "contentType": "application/json",
        "payload": {
            "type": "object",
            "required": ["market_id"],
            "properties": {
                "market_id": {
                    "type": "string",
                    "description": "Native market identifier (Kalshi ticker or Polymarket condition_id).",
                },
                "outcome": {
                    "type": ["string", "null"],
                    "description": "Optional outcome filter for binary markets.",
                },
            },
        },
    }
    components_messages["Unsubscribe"] = {
        "name": "Unsubscribe",
        "title": "Unsubscribe",
        "summary": "Remove a market from the active subscription set",
        "description": (
            "Call `WebSocket.unsubscribe(market_id)` to stop receiving "
            "updates for a market. The session continues for any remaining "
            "subscriptions."
        ),
        "contentType": "application/json",
        "payload": {
            "type": "object",
            "required": ["market_id"],
            "properties": {
                "market_id": {"type": "string"},
            },
        },
    }

    # Channels.
    channel_messages: dict[str, dict[str, Any]] = {
        name: {"$ref": f"#/components/messages/{name}"}
        for name in components_messages
    }

    channels = {
        "stream": {
            "address": "/v1/stream",
            "title": "OpenPX Stream",
            "description": (
                "Unified market-data + session-event stream for the "
                "connected exchange. Subscribe to any number of markets; "
                "OpenPX dispatches every update through one Tokio channel."
            ),
            "messages": channel_messages,
        }
    }

    # Operations.
    operations: dict[str, Any] = {}

    def receive_op(op_id: str, kind: str) -> dict[str, Any]:
        return {
            "action": "receive",
            "channel": {"$ref": "#/channels/stream"},
            "title": kind,
            "summary": components_messages[kind]["summary"],
            "messages": [{"$ref": f"#/channels/stream/messages/{kind}"}],
        }

    def send_op(op_id: str, kind: str, summary: str) -> dict[str, Any]:
        return {
            "action": "send",
            "channel": {"$ref": "#/channels/stream"},
            "title": kind,
            "summary": summary,
            "messages": [{"$ref": f"#/channels/stream/messages/{kind}"}],
        }

    operations["subscribe"] = send_op(
        "subscribe", "Subscribe", "Add a market to the active subscription set"
    )
    operations["unsubscribe"] = send_op(
        "unsubscribe", "Unsubscribe", "Remove a market from the active subscription set"
    )
    for kind in WS_UPDATE_VARIANTS:
        operations[f"receive{kind}"] = receive_op(f"receive{kind}", kind)
    for kind in SESSION_VARIANTS:
        operations[f"receive{kind}"] = receive_op(f"receive{kind}", kind)

    spec: dict[str, Any] = {
        "asyncapi": "3.0.0",
        "info": {
            "title": "OpenPX WebSocket",
            "version": read_workspace_version(),
            "description": (
                "Unified WebSocket stream over prediction-market exchanges. "
                "OpenPX is an in-process Rust library; this AsyncAPI document "
                "models the message shapes for documentation rendering only "
                "— there is no hosted WebSocket endpoint. The Rust engine "
                "translates these messages into each exchange's underlying "
                "WS protocol (Kalshi multiplex, Polymarket Socket.IO) "
                "behind the trait."
            ),
            "license": {"name": "MIT"},
        },
        "servers": {
            "inprocess": {
                "host": "openpx-engine",
                "protocol": "ws",
                "description": (
                    "In-process Tokio channel — not a real WebSocket. "
                    "OpenPX dispatches messages directly to your "
                    "subscribers without going over the network."
                ),
            }
        },
        "channels": channels,
        "operations": operations,
        "components": {
            "schemas": components_schemas,
            "messages": components_messages,
        },
    }
    return spec


MDX_TEMPLATE = """\
---
title: "WebSocket Stream"
sidebarTitle: "Stream"
description: "Unified WebSocket stream over Kalshi and Polymarket."
---

OpenPX exposes one stream per exchange that interleaves market-data
updates and session events. Subscribe to any number of markets — OpenPX
translates each call into the underlying exchange's subscription
protocol (Kalshi multiplex, Polymarket Socket.IO) and delivers every
message as a typed `WsUpdate` or `SessionEvent` variant.

> **Note:** OpenPX is an in-process Rust library. The AsyncAPI document
> below models the message shapes for docs rendering only — there is no
> hosted WebSocket endpoint to connect to.

## AsyncAPI

```yaml openpx.asyncapi.yaml stream
{asyncapi_body}
```
"""


def render_mdx(asyncapi_yaml: str) -> str:
    return MDX_TEMPLATE.format(asyncapi_body=asyncapi_yaml.rstrip("\n"))


def main() -> int:
    schema = json.loads(JSON_SCHEMA.read_text())
    schema_defs: dict[str, Any] = (
        schema.get("definitions") or schema.get("$defs") or {}
    )
    spec = build_asyncapi(schema_defs)
    asyncapi_body = yaml.safe_dump(spec, sort_keys=False, width=120)
    OUTPUT_YAML.write_text(asyncapi_body)

    OUTPUT_MDX.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MDX.write_text(render_mdx(asyncapi_body))

    print(
        f"wrote {OUTPUT_YAML.relative_to(ROOT)} "
        f"({OUTPUT_YAML.stat().st_size:,} bytes, "
        f"{len(spec['operations'])} operations, "
        f"{len(spec['components']['messages'])} messages, "
        f"{len(spec['components']['schemas'])} schemas)"
    )
    print(
        f"wrote {OUTPUT_MDX.relative_to(ROOT)} "
        f"({OUTPUT_MDX.stat().st_size:,} bytes)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
