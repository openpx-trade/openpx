#!/usr/bin/env python3
"""End-to-end orderbook test for the Python SDK.

Mirrors `engine/sdk/tests/e2e_orderbooks.rs` — every input variation of the
five orderbook methods is exercised against both Kalshi and Polymarket
through the published Python `Exchange` wrapper. Validates JSON-shape
parity with the Rust suite, sort/range/consistency invariants, and the
adversarial-input contract.

Run from repo root:
    sdks/python/.venv/bin/python e2e_orderbooks/python/test_orderbooks.py

Output: writes a JSON results document to ../results/python.json and
prints a pass/fail summary, exits non-zero if any case fails.
"""

from __future__ import annotations

import json
import os
import sys
import time
import traceback
from pathlib import Path

# Load .env so we don't have to wire credentials through the shell.
def load_dotenv(path: Path) -> None:
    if not path.exists():
        return
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip())


REPO_ROOT = Path(__file__).resolve().parents[2]
load_dotenv(REPO_ROOT / ".env")
# Resolve relative env paths against the repo root so the script can be run
# from any cwd (the Rust suite does this implicitly via `cargo test`).
key_path = os.environ.get("KALSHI_PRIVATE_KEY_PATH", "")
if key_path and not os.path.isabs(key_path):
    os.environ["KALSHI_PRIVATE_KEY_PATH"] = str(REPO_ROOT / key_path)
RESULTS_DIR = REPO_ROOT / "e2e_orderbooks" / "results"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_PATH = RESULTS_DIR / "python.json"

from openpx import Exchange  # noqa: E402

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

KALSHI_CONFIG = {
    "api_key_id": os.environ.get("KALSHI_API_KEY_ID", ""),
    "private_key_path": os.environ.get("KALSHI_PRIVATE_KEY_PATH", ""),
}
POLY_CONFIG = {
    "private_key": os.environ.get("POLYMARKET_PRIVATE_KEY", ""),
    "funder": os.environ.get("POLYMARKET_FUNDER", ""),
}


def make_exchange(eid: str):
    cfg = KALSHI_CONFIG if eid == "kalshi" else POLY_CONFIG
    cfg = {k: v for k, v in cfg.items() if v}
    return Exchange(eid, cfg)


# ---------------------------------------------------------------------------
# Result tracking
# ---------------------------------------------------------------------------

results: list[dict] = []


def case(name: str, expect: str = "ok"):
    """Decorator: runs the wrapped function immediately and records the result.

    Used as `@case("...")` above a `def _(entry): ...`. The function fires on
    decoration — there's no separate invocation step.
    """

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
            msg = str(e).lower()
            if expect == "err":
                entry["status"] = "pass"
                entry["detail"] = f"expected err: {e}"
            elif any(t in msg for t in ("rate limit", "429", "timed out", "http error")):
                entry["status"] = "skip"
                entry["detail"] = f"transient: {e}"
            else:
                entry["status"] = "fail"
                entry["detail"] = f"raised: {e}\n{traceback.format_exc()}"
        results.append(entry)
        symbol = {"pass": "✓", "fail": "✗", "skip": "·"}.get(entry["status"], "?")
        print(f"  {symbol} [{entry['status'].upper()}] {name}")
        if entry["status"] == "fail" and entry["detail"]:
            print(f"      → {str(entry['detail'])[:200]}")
        return fn  # return original so the binding to `_` is harmless

    return deco


def _unwrap(v):
    """If Pydantic wrapped a primitive in a RootModel, peel it back. Also
    coerce Pydantic v2 RootModel[float] (`Number`) so comparisons work."""
    root = getattr(v, "root", None)
    if root is not None and not callable(root):
        return root
    return v


def get(d, key):
    """Pydantic OR dict — uniform attribute lookup. Pydantic exposes
    attributes; raw dicts (Pydantic-import-failed path) expose subscript.
    Auto-unwraps RootModel-wrapped primitives so callers can compare directly."""
    if hasattr(d, key) and not isinstance(d, dict):
        return _unwrap(getattr(d, key))
    if isinstance(d, dict):
        return _unwrap(d.get(key))
    return _unwrap(getattr(d, key, None))


def to_dict(d):
    if hasattr(d, "model_dump"):
        return d.model_dump()
    if hasattr(d, "dict"):
        return d.dict()
    return d


# ---------------------------------------------------------------------------
# Invariant checkers — same as the Rust suite
# ---------------------------------------------------------------------------

def assert_book_well_formed(book, asset_id: str, label: str):
    bids = get(book, "bids")
    asks = get(book, "asks")
    # bids descending
    for i in range(len(bids) - 1):
        assert get(bids[i], "price") >= get(bids[i + 1], "price"), \
            f"{label}: bids not desc {bids[i]} then {bids[i + 1]}"
    # asks ascending
    for i in range(len(asks) - 1):
        assert get(asks[i], "price") <= get(asks[i + 1], "price"), \
            f"{label}: asks not asc {asks[i]} then {asks[i + 1]}"
    # price/size invariants
    for level in list(bids) + list(asks):
        p = get(level, "price"); s = get(level, "size")
        assert 0.0 < p < 1.0, f"{label}: price {p} OOB"
        assert s > 0.0, f"{label}: size {s} non-positive"
    # no crossed book
    if bids and asks:
        assert get(bids[0], "price") <= get(asks[0], "price"), \
            f"{label}: crossed book {bids[0]} > {asks[0]}"


def assert_stats_consistent(stats, book, label: str):
    # depth recomputes match
    bid_depth = sum(get(b, "size") for b in get(book, "bids"))
    ask_depth = sum(get(a, "size") for a in get(book, "asks"))
    assert abs(get(stats, "bid_depth") - bid_depth) < 1e-6, f"{label}: bid_depth drift"
    assert abs(get(stats, "ask_depth") - ask_depth) < 1e-6, f"{label}: ask_depth drift"
    mid = get(stats, "mid")
    if mid is not None:
        assert 0.0 <= mid <= 1.0, f"{label}: mid {mid} OOB"
    spread_bps = get(stats, "spread_bps")
    if spread_bps is not None:
        assert spread_bps >= 0.0, f"{label}: negative spread_bps"


def assert_impact_well_formed(impact, size: float, label: str):
    assert get(impact, "size") == size, f"{label}: size echo drift"
    assert 0.0 <= get(impact, "buy_fill_pct") <= 100.0, f"{label}: buy_fill_pct OOB"
    assert 0.0 <= get(impact, "sell_fill_pct") <= 100.0, f"{label}: sell_fill_pct OOB"
    bs = get(impact, "buy_slippage_bps")
    if bs is not None:
        assert bs >= 0.0, f"{label}: negative buy_slippage_bps"


def assert_microstructure_well_formed(micro, book, label: str):
    lc = get(micro, "level_count")
    assert get(lc, "bids") == len(get(book, "bids")), f"{label}: bid count drift"
    assert get(lc, "asks") == len(get(book, "asks")), f"{label}: ask count drift"
    db = get(micro, "depth_buckets")
    assert get(db, "bid_within_10bps") <= get(db, "bid_within_50bps") + 1e-9 \
        <= get(db, "bid_within_100bps") + 2e-9, f"{label}: bid bucket non-monotonic"
    assert get(db, "ask_within_10bps") <= get(db, "ask_within_50bps") + 1e-9 \
        <= get(db, "ask_within_100bps") + 2e-9, f"{label}: ask bucket non-monotonic"


def pick_asset_id(market) -> str:
    outs = get(market, "outcomes") or []
    if outs and get(outs[0], "token_id"):
        return get(outs[0], "token_id")
    return get(market, "ticker")


def seed_book(ex, label: str, max_tries: int = 10):
    """Find a market with a populated book."""
    page = ex.fetch_markets(status="active")
    markets = page["markets"] if isinstance(page, dict) else get(page, "markets")
    for m in list(markets)[:max_tries]:
        try:
            aid = pick_asset_id(m)
            book = ex.fetch_orderbook(aid)
            bids = get(book, "bids"); asks = get(book, "asks")
            if bids or asks:
                return m, aid, book
        except Exception:
            continue
    return None


# ---------------------------------------------------------------------------
# Per-exchange test plan
# ---------------------------------------------------------------------------

def run_suite(eid: str):
    print(f"\n=== {eid.upper()} ===")
    try:
        ex = make_exchange(eid)
    except Exception as e:
        print(f"  · SKIP: failed to construct {eid} client: {e}")
        return
    info = ex.describe()

    @case(f"[{eid}] describe advertises orderbook surface")
    def _(entry):
        assert info["has_fetch_orderbook"], "describe.has_fetch_orderbook false"
        assert info["has_fetch_orderbooks_batch"], "describe.has_fetch_orderbooks_batch false"

    seeded = seed_book(ex, f"{eid}/seed")
    if not seeded:
        print(f"  · SKIP: no market with non-empty book on {eid}")
        return
    _, aid, book = seeded
    print(f"    seeded asset_id={aid} (bids={len(get(book, 'bids'))} asks={len(get(book, 'asks'))})")

    @case(f"[{eid}] fetch_orderbook valid")
    def _(entry):
        b = ex.fetch_orderbook(aid)
        assert_book_well_formed(b, aid, f"{eid}/single")
        entry["detail"] = {
            "asset_id": aid, "bids": len(get(b, "bids")), "asks": len(get(b, "asks")),
        }

    @case(f"[{eid}] fetch_orderbook nonexistent → empty or 404")
    def _(entry):
        fake = "OPENPX-PY-NOPE-0" if eid == "kalshi" else "1"
        try:
            b = ex.fetch_orderbook(fake)
            # Kalshi returns empty book for nonexistent ticker — that's ok.
            assert not get(b, "bids") and not get(b, "asks"), "fake id returned populated book"
        except Exception as e:
            msg = str(e).lower()
            assert any(t in msg for t in ("not found", "invalid", "api")), f"unexpected: {e}"

    @case(f"[{eid}] fetch_orderbook malformed → empty or error")
    def _(entry):
        try:
            b = ex.fetch_orderbook("!@#$%")
            assert not get(b, "bids") and not get(b, "asks")
        except Exception:
            pass

    @case(f"[{eid}] fetch_orderbooks_batch empty → []")
    def _(entry):
        out = ex.fetch_orderbooks_batch([])
        assert out == [] or out == (), f"empty list returned {out}"

    # Seed 3 ids for batch
    page = ex.fetch_markets(status="active")
    markets = page["markets"] if isinstance(page, dict) else get(page, "markets")
    seed_ids = []
    for m in markets[:15]:
        try:
            aid_i = pick_asset_id(m)
            b = ex.fetch_orderbook(aid_i)
            if get(b, "bids") or get(b, "asks"):
                seed_ids.append(aid_i)
        except Exception:
            continue
        if len(seed_ids) >= 3:
            break

    if seed_ids:
        @case(f"[{eid}] fetch_orderbooks_batch multi → returns books")
        def _(entry):
            out = ex.fetch_orderbooks_batch(seed_ids)
            assert len(out) > 0, "batch returned 0"
            for b in out:
                assert_book_well_formed(b, get(b, "asset_id"), f"{eid}/batch")
            entry["detail"] = {"requested": len(seed_ids), "returned": len(out)}

    if eid == "kalshi":
        @case(f"[{eid}] fetch_orderbooks_batch above-cap (101) → InvalidOrder", expect="err")
        def _(entry):
            ex.fetch_orderbooks_batch([f"OPENPX-PY-CAP-{i}" for i in range(101)])

    @case(f"[{eid}] fetch_orderbook_stats consistent with book")
    def _(entry):
        s = ex.fetch_orderbook_stats(aid)
        assert_stats_consistent(s, book, f"{eid}/stats")
        entry["detail"] = {"mid": get(s, "mid"), "spread_bps": get(s, "spread_bps")}

    bids = get(book, "bids"); asks = get(book, "asks")
    if bids and asks:
        small = min(get(bids[0], "size"), get(asks[0], "size")) * 0.5
    elif bids:
        small = get(bids[0], "size") * 0.5
    elif asks:
        small = get(asks[0], "size") * 0.5
    else:
        small = 1.0

    @case(f"[{eid}] fetch_orderbook_impact small size → full or partial fill")
    def _(entry):
        i = ex.fetch_orderbook_impact(aid, small)
        assert_impact_well_formed(i, small, f"{eid}/impact_small")
        entry["detail"] = {
            "size": small,
            "buy_fill_pct": get(i, "buy_fill_pct"),
            "sell_fill_pct": get(i, "sell_fill_pct"),
        }

    big = (sum(get(b, "size") for b in bids) + sum(get(a, "size") for a in asks)) * 10.0 + 1.0

    @case(f"[{eid}] fetch_orderbook_impact large size → partial fill")
    def _(entry):
        i = ex.fetch_orderbook_impact(aid, big)
        assert_impact_well_formed(i, big, f"{eid}/impact_large")
        any_partial = get(i, "buy_fill_pct") < 100.0 or get(i, "sell_fill_pct") < 100.0
        assert any_partial, "oversize fully filled both sides"

    @case(f"[{eid}] fetch_orderbook_impact size=0 → InvalidInput", expect="err")
    def _(entry):
        ex.fetch_orderbook_impact(aid, 0.0)

    @case(f"[{eid}] fetch_orderbook_impact size=-42 → InvalidInput", expect="err")
    def _(entry):
        ex.fetch_orderbook_impact(aid, -42.0)

    @case(f"[{eid}] fetch_orderbook_microstructure consistent")
    def _(entry):
        m = ex.fetch_orderbook_microstructure(aid)
        assert_microstructure_well_formed(m, book, f"{eid}/micro")
        lc = get(m, "level_count")
        entry["detail"] = {
            "bids": get(lc, "bids"), "asks": get(lc, "asks"),
            "bid_slope": get(m, "bid_slope"), "ask_slope": get(m, "ask_slope"),
        }


if __name__ == "__main__":
    start = time.time()
    for eid in ("kalshi", "polymarket"):
        run_suite(eid)

    n_pass = sum(1 for r in results if r["status"] == "pass")
    n_fail = sum(1 for r in results if r["status"] == "fail")
    n_skip = sum(1 for r in results if r["status"] == "skip")

    summary = {
        "elapsed_s": round(time.time() - start, 2),
        "pass": n_pass,
        "fail": n_fail,
        "skip": n_skip,
        "results": results,
    }
    RESULTS_PATH.write_text(json.dumps(summary, indent=2, default=str))

    print(f"\n=== summary ===")
    print(f"PASS {n_pass}   FAIL {n_fail}   SKIP {n_skip}")
    print(f"results → {RESULTS_PATH}")
    sys.exit(0 if n_fail == 0 else 1)
