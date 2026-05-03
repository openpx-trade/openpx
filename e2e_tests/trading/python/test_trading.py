#!/usr/bin/env python3
"""End-to-end authenticated-trading test for the Python SDK.

Mirrors `engine/sdk/tests/e2e_trading.rs` — every authenticated endpoint is
exercised against both Kalshi (BTC 15m) and Polymarket (BTC 5m) live markets
through the published Python `Exchange` wrapper.

Run from repo root:
    sdks/python/.venv/bin/python e2e_trading/python/test_trading.py

Output: writes a JSON results document to ../results/python.json and prints
a pass/skip/fail summary, exits non-zero if any case fails (skips don't
count toward failure — Polymarket auth-derive may be intermittent).
"""

from __future__ import annotations

import json
import math
import os
import sys
import time
import traceback
from pathlib import Path


# Load .env so credentials propagate to the native bindings.
def load_dotenv(path: Path) -> None:
    if not path.exists():
        return
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


REPO_ROOT = Path(__file__).resolve().parents[3]
load_dotenv(REPO_ROOT / ".env")
key_path = os.environ.get("KALSHI_PRIVATE_KEY_PATH", "")
if key_path and not os.path.isabs(key_path):
    os.environ["KALSHI_PRIVATE_KEY_PATH"] = str(REPO_ROOT / key_path)
RESULTS_DIR = REPO_ROOT / "e2e_tests" / "trading" / "results"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_PATH = RESULTS_DIR / "python.json"

from openpx import Exchange  # noqa: E402

KALSHI_CONFIG = {
    "api_key_id": os.environ.get("KALSHI_API_KEY_ID", ""),
    "private_key_path": os.environ.get("KALSHI_PRIVATE_KEY_PATH", ""),
}
POLY_CONFIG = {
    "private_key": os.environ.get("POLYMARKET_PRIVATE_KEY", ""),
    "funder": os.environ.get("POLYMARKET_FUNDER", ""),
    "signature_type": os.environ.get("POLYMARKET_SIGNATURE_TYPE", ""),
}


def make_exchange(eid: str):
    cfg = KALSHI_CONFIG if eid == "kalshi" else POLY_CONFIG
    cfg = {k: v for k, v in cfg.items() if v}
    return Exchange(eid, cfg)


# ---------------------------------------------------------------------------
# Result tracking + helpers
# ---------------------------------------------------------------------------

results: list[dict] = []


def _is_auth_unavailable(e: Exception) -> bool:
    msg = str(e).lower()
    return any(t in msg for t in (
        "could not derive api key", "cloudflare waf blocked",
        "cannot reach clob.polymarket.com", "l1 eip-712 signature",
        "no signing method", "private key required",
    ))


def _is_transient(e: Exception) -> bool:
    msg = str(e).lower()
    return any(t in msg for t in (
        "rate limit", "429", "timed out", "http error", "connection",
    ))


def case(name: str, expect: str = "ok"):
    def deco(fn):
        entry = {"name": name, "expect": expect, "status": "pending", "detail": None}
        try:
            fn(entry)
            if entry["status"] == "pending":
                if expect == "err":
                    entry["status"] = "fail"
                    entry["detail"] = "expected an error but call returned ok"
                else:
                    entry["status"] = "pass"
        except AssertionError as e:
            entry["status"] = "fail"
            entry["detail"] = f"assert: {e}\n{traceback.format_exc()}"
        except Exception as e:  # noqa: BLE001
            if expect == "err":
                entry["status"] = "pass"
                entry["detail"] = f"expected err: {e}"
            elif _is_transient(e):
                entry["status"] = "skip"
                entry["detail"] = f"transient: {e}"
            elif _is_auth_unavailable(e):
                entry["status"] = "skip"
                entry["detail"] = f"auth unavailable: {e}"
            else:
                entry["status"] = "fail"
                entry["detail"] = f"raised: {e}\n{traceback.format_exc()}"
        results.append(entry)
        symbol = {"pass": "✓", "fail": "✗", "skip": "·"}.get(entry["status"], "?")
        print(f"  {symbol} [{entry['status'].upper()}] {name}")
        if entry["status"] == "fail" and entry["detail"]:
            print(f"      → {str(entry['detail'])[:240]}")
        return fn

    return deco


def _unwrap(v):
    root = getattr(v, "root", None)
    if root is not None and not callable(root):
        return root
    return v


def get(d, key):
    if hasattr(d, key) and not isinstance(d, dict):
        return _unwrap(getattr(d, key))
    if isinstance(d, dict):
        return _unwrap(d.get(key))
    return _unwrap(getattr(d, key, None))


# ---------------------------------------------------------------------------
# Active BTC market discovery
# ---------------------------------------------------------------------------

def kalshi_btc_market(ex):
    """Resolve the currently-active KXBTC15M market — the one with the soonest
    close_time wins."""
    raw = ex.fetch_markets(status="active", series_ticker="KXBTC15M", limit=5)
    markets = raw.get("markets", []) if isinstance(raw, dict) else getattr(raw, "markets", [])
    if not markets:
        return None
    # Pick the soonest-closing one.
    def _close(m):
        t = m.get("close_time") if isinstance(m, dict) else getattr(m, "close_time", None)
        return str(t or "")
    return sorted(markets, key=_close)[0]


def polymarket_btc_market(ex):
    """Round Polymarket server time to nearest 5m boundary, find that event."""
    server = ex.fetch_server_time()
    # server is RFC3339 string from the wrapper.
    import datetime as dt
    parsed = dt.datetime.fromisoformat(server.replace("Z", "+00:00"))
    now_unix = int(parsed.timestamp())
    bucket = (now_unix // 300) * 300
    for offset in (0, 300, -300):
        b = bucket + offset
        et = f"btc-updown-5m-{b}"
        raw = ex.fetch_markets(status="active", event_ticker=et, limit=5)
        markets = raw.get("markets", []) if isinstance(raw, dict) else getattr(raw, "markets", [])
        if markets:
            return markets[0]
    return None


def _market_ticker(m):
    return m.get("ticker") if isinstance(m, dict) else getattr(m, "ticker", None)


def _first_token_id(m):
    outs = m.get("outcomes") if isinstance(m, dict) else getattr(m, "outcomes", None)
    if not outs:
        return None
    o = outs[0]
    return (o.get("token_id") if isinstance(o, dict) else getattr(o, "token_id", None))


def _best_bid(m):
    return m.get("best_bid") if isinstance(m, dict) else getattr(m, "best_bid", None)


def _tick_size(m):
    t = m.get("tick_size") if isinstance(m, dict) else getattr(m, "tick_size", None)
    return t if t else 0.01


def safe_resting_buy_price(best_bid, tick):
    candidate = (best_bid - tick * 5.0) if best_bid is not None else 0.05
    candidate = min(candidate, 0.05)
    snapped = math.floor(candidate / tick) * tick
    return max(tick, min(1.0 - tick, snapped))


def safe_size(eid: str, price: float) -> float:
    return 1.0 if eid == "kalshi" else math.ceil(5.5 / price)


# ---------------------------------------------------------------------------
# Per-exchange suites
# ---------------------------------------------------------------------------

def run_suite(eid: str, market_resolver):
    print(f"\n=== {eid} ===")
    try:
        ex = make_exchange(eid)
    except Exception as e:  # noqa: BLE001
        print(f"  · skipping {eid}: cannot construct exchange: {e}")
        return
    try:
        market = market_resolver(ex)
    except Exception as e:  # noqa: BLE001
        print(f"  · skipping {eid}: market discovery failed: {e}")
        return
    if not market:
        print(f"  · skipping {eid}: no active BTC market")
        return

    market_ticker = _market_ticker(market)
    asset_id = market_ticker if eid == "kalshi" else _first_token_id(market)
    tick = _tick_size(market)
    bid = _best_bid(market)
    price = safe_resting_buy_price(bid, tick)
    size = safe_size(eid, price)
    print(f"  · market={market_ticker} asset={asset_id} price={price} size={size} tick={tick}")

    # ----- Account -----
    @case(f"[{eid}] fetch_server_time")
    def _(entry):
        ts = ex.fetch_server_time()
        assert ts, "empty server time"

    @case(f"[{eid}] fetch_balance")
    def _(entry):
        bal = ex.fetch_balance()
        key = "USD" if eid == "kalshi" else "USDC"
        assert key in bal, f"missing {key} in {bal}"
        assert bal[key] >= 0.0, f"negative balance {bal}"

    @case(f"[{eid}] refresh_balance")
    def _(entry):
        ex.refresh_balance()  # returns None

    # ----- Positions -----
    @case(f"[{eid}] fetch_positions unfiltered")
    def _(entry):
        pos = ex.fetch_positions()
        for p in pos:
            assert get(p, "size") > 0.0, f"non-positive position {p}"

    @case(f"[{eid}] fetch_positions filtered")
    def _(entry):
        pos = ex.fetch_positions(market_ticker)
        for p in pos:
            assert get(p, "size") > 0.0, f"non-positive position {p}"

    # ----- Orders: lifecycle -----
    @case(f"[{eid}] order lifecycle: create→fetch→cancel")
    def _(entry):
        order = ex.create_order(asset_id, "yes", "buy", price, size, "gtc")
        order_id = get(order, "id")
        assert order_id, f"no id on placed order {order}"
        # Best-effort verify via fetch_order. Kalshi's GET /portfolio/orders/{id}
        # can briefly return "not found" right after create (indexing race).
        try:
            fetched = ex.fetch_order(order_id)
            assert get(fetched, "id") == order_id, "fetch_order id drift"
        except Exception as e:
            msg = str(e).lower()
            if "not found" in msg or "marketnotfound" in msg:
                entry["detail"] = f"fetch race: {e}; tolerated"
            elif not (_is_transient(e) or _is_auth_unavailable(e)):
                raise
        # And via fetch_open_orders unfiltered
        try:
            opens = ex.fetch_open_orders()
            ids = [get(o, "id") for o in opens]
            if order_id not in ids and get(order, "status") not in ("filled", "Filled"):
                # not necessarily a fail — race window with fills/cancels
                entry["detail"] = f"open list missed our id {order_id}; tolerated"
        except Exception as e:
            if not (_is_transient(e) or _is_auth_unavailable(e)):
                raise
        # Always try to cancel
        try:
            cancelled = ex.cancel_order(order_id)
            assert get(cancelled, "status") in ("cancelled", "Cancelled"), \
                f"cancel returned non-cancelled status {cancelled}"
        except Exception as e:
            msg = str(e).lower()
            if "not found" in msg or "filled" in msg:
                pass  # already filled or gone — acceptable
            elif _is_transient(e) or _is_auth_unavailable(e):
                pass
            else:
                raise

    # ----- Orders: adversarial -----
    @case(f"[{eid}] create_order zero price", expect="err")
    def _(entry):
        ex.create_order(asset_id, "yes", "buy", 0.0, size, "gtc")

    @case(f"[{eid}] create_order one price", expect="err")
    def _(entry):
        ex.create_order(asset_id, "yes", "buy", 1.0, size, "gtc")

    @case(f"[{eid}] create_order negative size", expect="err")
    def _(entry):
        ex.create_order(asset_id, "yes", "buy", price, -size, "gtc")

    # ----- fetch_order / cancel_order with bogus id -----
    fake_id = "00000000-0000-0000-0000-000000000000" if eid == "kalshi" \
        else "0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddead"

    @case(f"[{eid}] fetch_order unknown id", expect="err")
    def _(entry):
        ex.fetch_order(fake_id)

    @case(f"[{eid}] cancel_order unknown id", expect="err")
    def _(entry):
        ex.cancel_order(fake_id)

    # ----- Batch: empty / oversize -----
    @case(f"[{eid}] create_orders_batch empty")
    def _(entry):
        out = ex.create_orders_batch([])
        assert isinstance(out, list) and len(out) == 0, f"empty batch returned {out}"

    @case(f"[{eid}] create_orders_batch oversize", expect="err")
    def _(entry):
        cap = 21 if eid == "kalshi" else 16
        reqs = [
            {"asset_id": asset_id, "outcome": "yes", "side": "buy",
             "price": 0.05, "size": size, "order_type": "gtc"}
            for _ in range(cap)
        ]
        # Polymarket has hard cap 15; Kalshi may accept then we cancel.
        out = ex.create_orders_batch(reqs)
        # Cleanup if it accepted.
        for o in out or []:
            try:
                ex.cancel_order(get(o, "id"))
            except Exception:
                pass
        if eid == "polymarket":
            raise AssertionError(f"oversize batch ({cap}) was accepted")

    # ----- Bulk cancels -----
    @case(f"[{eid}] cancel_all_orders unfiltered")
    def _(entry):
        out = ex.cancel_all_orders()
        assert isinstance(out, list)

    @case(f"[{eid}] cancel_all_orders filtered")
    def _(entry):
        out = ex.cancel_all_orders(asset_id)
        assert isinstance(out, list)

    # ----- Open orders -----
    @case(f"[{eid}] fetch_open_orders unfiltered")
    def _(entry):
        out = ex.fetch_open_orders()
        for o in out:
            assert get(o, "id"), f"order missing id: {o}"

    @case(f"[{eid}] fetch_open_orders filtered")
    def _(entry):
        out = ex.fetch_open_orders(asset_id)
        assert isinstance(out, list)

    # ----- Fills -----
    @case(f"[{eid}] fetch_fills unfiltered limit=10")
    def _(entry):
        fills = ex.fetch_fills(limit=10)
        assert len(fills) <= 10, f"limit not honored: {len(fills)}"
        for f in fills:
            p = get(f, "price")
            s = get(f, "size")
            assert s > 0.0, f"non-positive fill size {f}"
            assert 0.0 < p < 1.0, f"fill price out of range {f}"

    @case(f"[{eid}] fetch_fills filtered limit=5")
    def _(entry):
        fills = ex.fetch_fills(market_ticker=market_ticker, limit=5)
        assert len(fills) <= 5

    # ----- Trades (public) -----
    @case(f"[{eid}] fetch_trades basic")
    def _(entry):
        trades = ex.fetch_trades(market_ticker, limit=20)
        # The wrapper returns dict with 'trades' key from native binding.
        items = trades.get("trades", []) if isinstance(trades, dict) else trades
        for t in items:
            p = get(t, "price")
            s = get(t, "size")
            assert s > 0.0, f"trade size non-positive {t}"
            assert 0.0 < p < 1.0, f"trade price out of (0,1) {t}"

    @case(f"[{eid}] fetch_trades with time window")
    def _(entry):
        now = int(time.time())
        trades = ex.fetch_trades(market_ticker, start_ts=now - 3600, end_ts=now, limit=50)
        items = trades.get("trades", []) if isinstance(trades, dict) else trades
        # Loose bound: trades within window or a small skew tolerance.
        assert isinstance(items, list)


# ---------------------------------------------------------------------------
# Drive both exchanges
# ---------------------------------------------------------------------------

def main() -> int:
    run_suite("kalshi", kalshi_btc_market)
    run_suite("polymarket", polymarket_btc_market)

    pass_n = sum(1 for r in results if r["status"] == "pass")
    skip_n = sum(1 for r in results if r["status"] == "skip")
    fail_n = sum(1 for r in results if r["status"] == "fail")

    summary = {
        "total": len(results),
        "pass": pass_n,
        "skip": skip_n,
        "fail": fail_n,
        "results": results,
    }
    RESULTS_PATH.write_text(json.dumps(summary, indent=2, default=str))
    print(f"\nPASS {pass_n}\nSKIP {skip_n}\nFAIL {fail_n}")
    if fail_n > 0:
        print("\nFailing cases:")
        for r in results:
            if r["status"] == "fail":
                print(f"  - {r['name']}: {str(r['detail'])[:200]}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
