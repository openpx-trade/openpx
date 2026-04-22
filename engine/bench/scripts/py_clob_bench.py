#!/usr/bin/env python3
"""Latency probe that mirrors the polyfill-rs methodology for the Python column.

Prints one JSON object per timed iteration on stdout, e.g.
    {"i": 1, "elapsed_ms": 412.3}

The Rust harness (src/bin/network_bench.rs) subprocesses this script and
consumes those lines. Requires:
    pip install py-clob-client
"""
import argparse
import json
import sys
import time


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--delay-ms", type=int, default=100)
    parser.add_argument("--warmup", type=int, default=5)
    parser.add_argument(
        "--endpoint",
        default="https://clob.polymarket.com/simplified-markets?next_cursor=MA==",
    )
    args = parser.parse_args()

    try:
        from py_clob_client.client import ClobClient  # type: ignore
    except Exception as e:  # noqa: BLE001
        print(f"py-clob-client not installed: {e}", file=sys.stderr)
        print("install with:  pip install py-clob-client", file=sys.stderr)
        return 1

    host = "https://clob.polymarket.com"
    client = ClobClient(host)

    for _ in range(args.warmup):
        try:
            client.get_simplified_markets(next_cursor="MA==")
        except Exception as e:  # noqa: BLE001
            print(f"warmup error: {e}", file=sys.stderr)
        time.sleep(args.delay_ms / 1000.0)

    for i in range(1, args.iterations + 1):
        start = time.perf_counter()
        try:
            client.get_simplified_markets(next_cursor="MA==")
            elapsed_ms = (time.perf_counter() - start) * 1000.0
            print(json.dumps({"i": i, "elapsed_ms": elapsed_ms}), flush=True)
        except Exception as e:  # noqa: BLE001
            print(f"iter {i} error: {e}", file=sys.stderr)
        time.sleep(args.delay_ms / 1000.0)
    return 0


if __name__ == "__main__":
    sys.exit(main())
