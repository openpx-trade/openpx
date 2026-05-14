// WebSocket hot-path bench (TypeScript): OpenPX-equivalent vs DIY.
//
// The official Polymarket and Kalshi TS SDKs (@polymarket/clob-client,
// kalshi-typescript-sdk) don't ship WebSocket. A user replicating
// real-time book state writes: ws subscribe + JSON.parse + Map-based
// book. This bench measures the cost of that DIY path on real frames.
//
// The OpenPX-side number is sourced from the Rust `ws_decode_apply`
// bench (cargo bench -p px-bench-comparative --bench hot_path). The
// Node user pays roughly that plus ~50-200ns of NAPI marshalling per
// message.
//
// Writes JSON summary to stdout.
//
// Run: `node bench_ws_diy.mjs > ../results/typescript_ws_diy.json`

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { Bench } from "tinybench";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const FIXTURE = path.join(
  __dirname,
  "..",
  "fixtures",
  "polymarket_ws_book.jsonl"
);

if (!fs.existsSync(FIXTURE)) {
  console.error(`missing fixture ${FIXTURE}; run capture_bench_fixtures.py`);
  process.exit(1);
}

const lines = fs.readFileSync(FIXTURE, "utf8").split("\n");
// Skip array-wrapped snapshot frames so the bench matches the Rust
// `ws_decode_apply` set exactly.
const FRAMES = lines.filter((l) => l && l[0] !== "[");

function diyDecodeApply(frames) {
  const bids = new Map();
  const asks = new Map();
  let processed = 0;
  for (const frame of frames) {
    const msg = JSON.parse(frame);
    const etype = msg.event_type;
    if (etype === "book") {
      bids.clear();
      asks.clear();
      for (const lvl of msg.bids ?? []) bids.set(lvl.price, lvl.size);
      for (const lvl of msg.asks ?? []) asks.set(lvl.price, lvl.size);
    } else if (etype === "price_change") {
      for (const ch of msg.price_changes ?? []) {
        const side = (ch.side ?? "").toUpperCase();
        const book = side === "BUY" ? bids : asks;
        const sz = ch.size ?? "0";
        const px = ch.price ?? "";
        if (!px) continue;
        if (parseFloat(sz) === 0) book.delete(px);
        else book.set(px, sz);
      }
    }
    // last_trade_price has no book impact in the minimal handler
    processed += 1;
  }
  return { processed, levels: bids.size + asks.size };
}

const bench = new Bench({ time: 0, iterations: 20 });
bench.add("diy::polymarket::ws_decode_apply", () => {
  diyDecodeApply(FRAMES);
});

await bench.run();

const tasks = bench.tasks.map((t) => ({
  name: t.name,
  mean_ns: t.result?.mean ? t.result.mean * 1e6 : null,
  stddev_ns: t.result?.sd ? t.result.sd * 1e6 : null,
  samples: t.result?.samples?.length ?? 0,
  msgs_per_run: FRAMES.length,
}));

console.log(JSON.stringify({ tasks }, null, 2));
