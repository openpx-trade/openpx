"""Python SDK binding-overhead benchmarks.

Measures the cost of the PyO3 boundary alone — no network. We use
`describe()` (the canonical hot path the recent caching commits 918f77e /
4f21aac targeted) and a constructor microbench. The autoresearch oracle
divides the Python p99 by the Rust-direct p99 to derive `py_overhead_ratio`,
so any binding regression shows up immediately.

No credentials needed: `describe()` works with an empty config because
the manifest is static metadata. This keeps the bench fully reproducible.

Run via `pytest sdks/python/benches/ --benchmark-only --benchmark-json=...`.
"""

from __future__ import annotations

import pytest

from openpx._native import NativeExchange


@pytest.fixture(scope="module")
def kalshi_native() -> NativeExchange:
    # Empty dict is sufficient for describe() — manifest metadata is static.
    return NativeExchange("kalshi", {})


@pytest.fixture(scope="module")
def polymarket_native() -> NativeExchange:
    return NativeExchange("polymarket", {})


@pytest.mark.benchmark(group="describe_cached")
def test_describe_kalshi_cached(benchmark, kalshi_native: NativeExchange) -> None:
    # Prime the OnceLock so we measure the cached fast path the user-facing
    # `describe()` call hits in steady state.
    kalshi_native.describe()
    benchmark(kalshi_native.describe)


@pytest.mark.benchmark(group="describe_cached")
def test_describe_polymarket_cached(benchmark, polymarket_native: NativeExchange) -> None:
    polymarket_native.describe()
    benchmark(polymarket_native.describe)


@pytest.mark.benchmark(group="constructor")
def test_construct_native_kalshi(benchmark) -> None:
    # Cold-path baseline. Constructor cost includes config depythonize +
    # ExchangeInner::new + Arc allocation. Any regression here means the
    # binding got heavier on first contact.
    benchmark(lambda: NativeExchange("kalshi", {}))


@pytest.mark.benchmark(group="constructor")
def test_construct_native_polymarket(benchmark) -> None:
    benchmark(lambda: NativeExchange("polymarket", {}))


@pytest.mark.benchmark(group="getter")
def test_id_getter_kalshi(benchmark, kalshi_native: NativeExchange) -> None:
    # Pure attribute getter — the cheapest path across the boundary.
    # Useful as a floor/sanity reference.
    benchmark(lambda: kalshi_native.id)
