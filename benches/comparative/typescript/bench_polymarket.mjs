// Polymarket REST head-to-head: OpenPX vs @polymarket/clob-client.
//
// Single fair comparison: `fetch_orderbook`. Both clients hit the
// same upstream endpoint and return the same shape.
//
// 20 iterations × 100 ms gap, polyfill methodology.
//
// Run: `node bench_polymarket.mjs > ../results/typescript_polymarket.json`

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { Bench } from "tinybench";
import { Exchange } from "@openpx/sdk";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const META = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, "..", "fixtures", "polymarket_book.meta.json"),
    "utf8"
  )
);
const ASSET_ID = META.asset_id;
const GAP_MS = 100;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let ClobClient = null;
try {
  ({ ClobClient } = await import("@polymarket/clob-client"));
} catch {
  console.error("note: @polymarket/clob-client not installed, OpenPX-only run");
}

const openpx = new Exchange("polymarket", {});
const sdk = ClobClient ? new ClobClient("https://clob.polymarket.com") : null;

const bench = new Bench({ time: 0, iterations: 20 });

bench.add("openpx::polymarket::fetch_orderbook", async () => {
  await sleep(GAP_MS);
  await openpx.fetchOrderbook(ASSET_ID);
});
if (sdk) {
  bench.add("polymarket-clob-client::fetch_orderbook", async () => {
    await sleep(GAP_MS);
    await sdk.getOrderBook(ASSET_ID);
  });
}

await bench.run();

const tasks = bench.tasks.map((t) => ({
  name: t.name,
  mean_ns: t.result?.mean ? t.result.mean * 1e6 : null,
  stddev_ns: t.result?.sd ? t.result.sd * 1e6 : null,
  samples: t.result?.samples?.length ?? 0,
}));

console.log(JSON.stringify({ tasks }, null, 2));
