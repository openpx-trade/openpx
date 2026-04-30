#!/usr/bin/env python3
"""Generate `docs/websockets/openpx.asyncapi.yaml` + per-channel MDX wrappers.

Mintlify resolves `asyncapi:` frontmatter paths relative to the referencing
MDX file's directory, so the spec must live in `docs/websockets/` next to the
MDX pages — not at the docs root.

Five channels are documented, one MDX page each:

  orderbook  — Snapshot / Delta / Clear / BookInvalidated
  trades     — Trade
  fills      — Fill (authenticated user only)
  crypto     — CryptoPrice (Polymarket-hosted, public)
  sports     — SportResult (Polymarket-hosted, public)

Every channel includes the global session-lifecycle events (Connected,
Reconnected, Lagged, Error) and the relevant subscribe/unsubscribe send
operations.

Sources of truth: the JSON Schema at `schema/openpx.schema.json` (which now
includes CryptoPrice / CryptoPriceSource / SportResult — added to the
schema-export bin) and the Cargo workspace version.
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
DOCS_DIR = ROOT / "docs"
WS_DIR = DOCS_DIR / "websockets"
# Per Mintlify's AsyncAPI docs, the spec file must live alongside docs.json
# (not in a subdirectory). Using the .json extension matches the convention
# their renderer expects (Polymarket's docs use the same pattern).
OUTPUT_JSON = DOCS_DIR / "openpx.asyncapi.json"

# Per-channel content. Each channel name → (sidebarTitle, page title,
# description, list of (kind, source-union) tuples for receive messages,
# subscribe-payload-schema fragment).
SUBSCRIBE_MARKET_PAYLOAD = {
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
}
SUBSCRIBE_CRYPTO_PAYLOAD = {
    "type": "object",
    "required": ["source", "symbols"],
    "properties": {
        "source": {
            "$ref": "#/components/schemas/CryptoPriceSource",
            "description": "Upstream price source — Binance or Chainlink.",
        },
        "symbols": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Symbols to subscribe to (e.g. `[\"btcusdt\", \"ethusdt\"]`). Empty = all.",
        },
    },
}
SUBSCRIBE_SPORTS_PAYLOAD = {
    "type": "object",
    "required": ["league"],
    "properties": {
        "league": {
            "type": "string",
            "description": "League slug (e.g. `nba`, `nfl`, `mlb`).",
        },
    },
}

# Session lifecycle events — present on every channel.
SESSION_VARIANTS = ["Connected", "Reconnected", "Lagged", "BookInvalidated", "Error"]

CHANNELS: dict[str, dict[str, Any]] = {
    "orderbook": {
        "sidebar": "Orderbook",
        "title": "Orderbook Stream",
        "description": "Real-time L2 orderbook snapshots and incremental deltas.",
        "intro": (
            "Subscribe per market — OpenPX delivers a `Snapshot` on first "
            "subscribe and `Delta` updates thereafter. `Clear` and "
            "`BookInvalidated` indicate the cached book should be discarded "
            "and rebuilt from the next `Snapshot`."
        ),
        "address": "/v1/orderbook",
        "receives": [
            ("Snapshot", "WsUpdate"),
            ("Delta", "WsUpdate"),
            ("Clear", "WsUpdate"),
        ],
        "subscribe_payload": SUBSCRIBE_MARKET_PAYLOAD,
    },
    "trades": {
        "sidebar": "Trades",
        "title": "Trades Stream",
        "description": "Public trade-tape stream for any market.",
        "intro": (
            "Receive every public trade as it prints. The `Trade` payload "
            "carries the matched price, size, and aggressor side; not "
            "scoped to your orders (use the Fills stream for those)."
        ),
        "address": "/v1/trades",
        "receives": [("Trade", "WsUpdate")],
        "subscribe_payload": SUBSCRIBE_MARKET_PAYLOAD,
    },
    "fills": {
        "sidebar": "Fills",
        "title": "Fills Stream",
        "description": "Authenticated stream of your own order fills.",
        "intro": (
            "Receive a `Fill` event each time one of the authenticated "
            "user's orders is matched. Includes maker/taker role, fee "
            "rate, and the parent order ID. Auth required (the WebSocket "
            "uses the same exchange config as the REST surface)."
        ),
        "address": "/v1/fills",
        "receives": [("Fill", "WsUpdate")],
        "subscribe_payload": SUBSCRIBE_MARKET_PAYLOAD,
    },
    "crypto": {
        "sidebar": "Crypto",
        "title": "Crypto Price Stream",
        "description": "Real-time spot prices from Binance and Chainlink (Polymarket-hosted, public).",
        "intro": (
            "Polymarket re-publishes spot crypto prices over a public "
            "WebSocket; OpenPX exposes the same stream as a typed "
            "`CryptoPrice` event. No authentication needed."
        ),
        "address": "/v1/crypto",
        "receives": [],
        "extra_messages": [
            (
                "CryptoPrice",
                {
                    "name": "CryptoPrice",
                    "title": "CryptoPrice",
                    "summary": "Spot price update for a single symbol",
                    "description": "Spot price update from the upstream source.",
                    "contentType": "application/json",
                    "payload": {"$ref": "#/components/schemas/CryptoPrice"},
                },
            ),
        ],
        "subscribe_payload": SUBSCRIBE_CRYPTO_PAYLOAD,
    },
    "sports": {
        "sidebar": "Sports",
        "title": "Sports Stream",
        "description": "Live sports scores and game state (Polymarket-hosted, public).",
        "intro": (
            "Polymarket's public sports WebSocket emits live game updates "
            "for sports markets. OpenPX exposes the same stream as a typed "
            "`SportResult` event."
        ),
        "address": "/v1/sports",
        "receives": [],
        "extra_messages": [
            (
                "SportResult",
                {
                    "name": "SportResult",
                    "title": "SportResult",
                    "summary": "Live game state update",
                    "description": "Live score, period, and clock for a tracked game.",
                    "contentType": "application/json",
                    "payload": {"$ref": "#/components/schemas/SportResult"},
                },
            ),
        ],
        "subscribe_payload": SUBSCRIBE_SPORTS_PAYLOAD,
    },
}


def read_workspace_version() -> str:
    text = CARGO_TOML.read_text()
    m = re.search(
        r"\[workspace\.package\][^\[]*?^version\s*=\s*\"([^\"]+)\"",
        text,
        re.MULTILINE | re.DOTALL,
    )
    return m.group(1) if m else "0.0.0"


def normalize_jsonschema(node: Any) -> Any:
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
    for variant in union_node.get("oneOf", []):
        kind_node = variant.get("properties", {}).get("kind", {})
        const = kind_node.get("const")
        enum_vals = kind_node.get("enum") or ([const] if const else [])
        if kind in enum_vals:
            return variant
    return None


def extract_variant_schema(union_name: str, kind: str, schema_defs: dict[str, Any]) -> dict[str, Any]:
    variant = find_variant(schema_defs[union_name], kind)
    if variant is None:
        sys.exit(f"variant `{kind}` not found in `{union_name}`")
    cleaned = {k: v for k, v in variant.items() if k != "title"}
    return normalize_jsonschema(cleaned)


def build_message(kind: str, union_name: str, schema_defs: dict[str, Any]) -> dict[str, Any]:
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


def build_asyncapi(schema_defs: dict[str, Any]) -> dict[str, Any]:
    components_schemas: dict[str, Any] = {
        name: normalize_jsonschema(body) for name, body in schema_defs.items()
    }
    components_messages: dict[str, dict[str, Any]] = {}

    # Pull union variants out as standalone schemas.
    for kind in ["Snapshot", "Delta", "Clear", "Trade", "Fill"]:
        components_schemas[kind] = extract_variant_schema("WsUpdate", kind, schema_defs)
        components_messages[kind] = build_message(kind, "WsUpdate", schema_defs)
    for kind in SESSION_VARIANTS:
        components_schemas[kind] = extract_variant_schema("SessionEvent", kind, schema_defs)
        components_messages[kind] = build_message(kind, "SessionEvent", schema_defs)

    # Crypto + sports payload messages (one each — not tagged unions).
    for spec in CHANNELS.values():
        for name, msg in spec.get("extra_messages", []):
            components_messages[name] = msg

    channels: dict[str, Any] = {}
    operations: dict[str, Any] = {}

    def receive_op(channel: str, kind: str) -> dict[str, Any]:
        return {
            "action": "receive",
            "channel": {"$ref": f"#/channels/{channel}"},
            "title": kind,
            "summary": components_messages[kind]["summary"],
            "messages": [{"$ref": f"#/channels/{channel}/messages/{kind}"}],
        }

    for channel_id, spec in CHANNELS.items():
        # Build subscribe + unsubscribe message + ops first.
        sub_msg_id = f"{channel_id}_subscribe"
        unsub_msg_id = f"{channel_id}_unsubscribe"
        components_messages[sub_msg_id] = {
            "name": sub_msg_id,
            "title": "Subscribe",
            "summary": f"Add to the active {channel_id} subscription set",
            "description": (
                f"Call subscribe on the WebSocket after the stream is "
                f"connected to start receiving {channel_id} updates."
            ),
            "contentType": "application/json",
            "payload": spec["subscribe_payload"],
        }
        components_messages[unsub_msg_id] = {
            "name": unsub_msg_id,
            "title": "Unsubscribe",
            "summary": f"Remove from the active {channel_id} subscription set",
            "description": "Stop receiving updates without closing the stream.",
            "contentType": "application/json",
            "payload": spec["subscribe_payload"],
        }

        # Channel messages = receive variants + session events + sub/unsub.
        receive_kinds = [k for k, _ in spec.get("receives", [])] + [
            name for name, _ in spec.get("extra_messages", [])
        ]
        channel_msg_ids = receive_kinds + SESSION_VARIANTS + [sub_msg_id, unsub_msg_id]
        channels[channel_id] = {
            "address": spec["address"],
            "title": spec["title"],
            "description": spec["intro"],
            "messages": {
                m: {"$ref": f"#/components/messages/{m}"} for m in channel_msg_ids
            },
        }

        # Operations: subscribe/unsubscribe (send), each receive (receive).
        operations[f"{channel_id}_subscribe"] = {
            "action": "send",
            "channel": {"$ref": f"#/channels/{channel_id}"},
            "title": "Subscribe",
            "summary": components_messages[sub_msg_id]["summary"],
            "messages": [{"$ref": f"#/channels/{channel_id}/messages/{sub_msg_id}"}],
        }
        operations[f"{channel_id}_unsubscribe"] = {
            "action": "send",
            "channel": {"$ref": f"#/channels/{channel_id}"},
            "title": "Unsubscribe",
            "summary": components_messages[unsub_msg_id]["summary"],
            "messages": [{"$ref": f"#/channels/{channel_id}/messages/{unsub_msg_id}"}],
        }
        for kind in receive_kinds:
            operations[f"{channel_id}_receive_{kind}"] = receive_op(channel_id, kind)
        for kind in SESSION_VARIANTS:
            operations[f"{channel_id}_receive_{kind}"] = receive_op(channel_id, kind)

    spec: dict[str, Any] = {
        "asyncapi": "3.0.0",
        "info": {
            "title": "OpenPX WebSocket",
            "version": read_workspace_version(),
            "description": (
                "Unified WebSocket streams over prediction-market exchanges. "
                "OpenPX is an in-process Rust library; this AsyncAPI document "
                "models message shapes for documentation only — there is no "
                "hosted endpoint. The Rust engine translates these messages "
                "into each exchange's underlying WS protocol behind the trait."
            ),
            "license": {"name": "MIT"},
        },
        "servers": {
            "inprocess": {
                "host": "openpx-engine",
                "protocol": "ws",
                "description": (
                    "In-process Tokio channel — not a real WebSocket. "
                    "OpenPX dispatches messages directly to your subscribers."
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


# ---------------------------------------------------------------------------
# MDX wrappers
# ---------------------------------------------------------------------------

MDX_TEMPLATE = """\
---
title: {title!r}
sidebarTitle: {sidebar!r}
description: {description!r}
asyncapi: "/openpx.asyncapi.json {channel_id}"
---

{intro}

> **Note:** OpenPX is an in-process Rust library. The AsyncAPI document
> below models the message shapes for docs rendering only — there is no
> hosted WebSocket endpoint to connect to.
"""


def render_mdx(channel_id: str, spec: dict[str, Any]) -> str:
    return MDX_TEMPLATE.format(
        title=spec["title"],
        sidebar=spec["sidebar"],
        description=spec["description"],
        intro=spec["intro"],
        channel_id=channel_id,
    )


def main() -> int:
    schema = json.loads(JSON_SCHEMA.read_text())
    schema_defs: dict[str, Any] = (
        schema.get("definitions") or schema.get("$defs") or {}
    )
    spec = build_asyncapi(schema_defs)

    WS_DIR.mkdir(parents=True, exist_ok=True)
    OUTPUT_JSON.write_text(json.dumps(spec, indent=2) + "\n")
    print(
        f"wrote {OUTPUT_JSON.relative_to(ROOT)} "
        f"({OUTPUT_JSON.stat().st_size:,} bytes, "
        f"{len(spec['operations'])} operations, "
        f"{len(spec['components']['messages'])} messages, "
        f"{len(spec['components']['schemas'])} schemas)"
    )

    for channel_id, channel_spec in CHANNELS.items():
        out = WS_DIR / f"{channel_id}.mdx"
        out.write_text(render_mdx(channel_id, channel_spec))
        print(f"wrote {out.relative_to(ROOT)} ({out.stat().st_size:,} bytes)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
