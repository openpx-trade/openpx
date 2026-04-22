#!/usr/bin/env python3
"""Hand-rolled Polymarket WebSocket baseline in Python — what a dev would
write without any Polymarket SDK (neither `py-clob-client` nor any
Polymarket-published SDK exposes market-book WebSockets).

Measures connect → subscribe → first book snapshot latency. Prints one
JSON line per timed iteration for the Rust harness to consume.

    pip install websockets
"""
import argparse
import asyncio
import json
import sys
import time

WS_URL = "wss://ws-subscriptions-clob.polymarket.com/ws/market"


async def one_iteration(asset_id: str, timeout_secs: float) -> float | None:
    """Returns elapsed ms on success, None on timeout / error."""
    try:
        import websockets
    except ImportError:
        print("websockets not installed: pip install websockets", file=sys.stderr)
        sys.exit(1)

    start = time.perf_counter()
    try:
        async with websockets.connect(WS_URL) as ws:
            subscribe = {
                "type": "market",
                "assets_ids": [asset_id],
                "markets": [],
            }
            await ws.send(json.dumps(subscribe))

            async def wait_for_book():
                async for raw in ws:
                    try:
                        parsed = json.loads(raw)
                    except Exception:
                        continue
                    items = parsed if isinstance(parsed, list) else [parsed]
                    for item in items:
                        if isinstance(item, dict) and item.get("event_type") == "book":
                            return True
                return False

            try:
                ok = await asyncio.wait_for(wait_for_book(), timeout=timeout_secs)
                if not ok:
                    return None
            except asyncio.TimeoutError:
                return None
    except Exception as e:  # noqa: BLE001
        print(f"iteration error: {e}", file=sys.stderr)
        return None
    return (time.perf_counter() - start) * 1000.0


async def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--iterations", type=int, default=12,
                        help="Total iterations (including warmup).")
    parser.add_argument("--warmup", type=int, default=2,
                        help="Number of leading iterations to discard.")
    parser.add_argument("--delay-ms", type=int, default=500)
    parser.add_argument("--timeout-secs", type=int, default=15)
    parser.add_argument("--asset-id", required=True)
    args = parser.parse_args()

    for i in range(1, args.iterations + 1):
        elapsed_ms = await one_iteration(args.asset_id, args.timeout_secs)
        if elapsed_ms is None:
            print(f"iter {i} timeout/error", file=sys.stderr)
        elif i > args.warmup:
            # Only emit JSON for timed iterations — warmup stays silent.
            print(json.dumps({"i": i - args.warmup, "elapsed_ms": elapsed_ms}), flush=True)
        await asyncio.sleep(args.delay_ms / 1000.0)
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
