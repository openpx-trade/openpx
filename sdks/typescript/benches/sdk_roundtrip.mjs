// TypeScript SDK binding-overhead benchmarks.
//
// Measures the cost of the NAPI boundary alone — no network. We bench
// `describe()` (the canonical hot path commit 6fdc2aa optimised — bytes
// cache + per-call deserialize) and a constructor microbench. The
// autoresearch oracle divides the Node p99 by the Rust-direct p99 to
// derive `ts_overhead_ratio`.
//
// No credentials needed: `describe()` works with an empty config because
// the manifest is static metadata.
//
// Run via `node --experimental-vm-modules sdks/typescript/benches/sdk_roundtrip.mjs`.

import { Bench } from "tinybench";
import { Exchange } from "../index.js";

const TIME_MS = 2000; // per-bench wall-clock budget

const bench = new Bench({ time: TIME_MS });

const kalshi = new Exchange("kalshi", {});
const polymarket = new Exchange("polymarket", {});

// Prime caches so the steady-state numbers are what we report.
kalshi.describe();
polymarket.describe();

bench
  .add("describe_kalshi_cached", () => {
    kalshi.describe();
  })
  .add("describe_polymarket_cached", () => {
    polymarket.describe();
  })
  .add("construct_kalshi", () => {
    new Exchange("kalshi", {});
  })
  .add("construct_polymarket", () => {
    new Exchange("polymarket", {});
  })
  .add("id_getter_kalshi", () => {
    kalshi.id;
  });

await bench.run();

const tasks = bench.tasks.map((task) => {
  const r = task.result;
  return {
    name: task.name,
    samples: r?.samples?.length ?? 0,
    p50_ns: r?.p50 != null ? Math.round(r.p50 * 1e6) : null,
    p99_ns: r?.p99 != null ? Math.round(r.p99 * 1e6) : null,
    p999_ns: r?.p999 != null ? Math.round(r.p999 * 1e6) : null,
    mean_ns: r?.mean != null ? Math.round(r.mean * 1e6) : null,
    hz: r?.hz ?? null,
  };
});

console.log(JSON.stringify({ time_ms_budget: TIME_MS, tasks }));
