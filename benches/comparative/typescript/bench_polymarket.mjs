// Polymarket REST API head-to-head: OpenPX vs @polymarket/clob-client.
//
// 20 iterations × 100 ms gap, polyfill methodology. Writes a JSON
// summary to stdout that the README renderer parses.
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
const CONDITION_ID = META.condition_id;
const GAP_MS = 100;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Optional comparison target — skip the SDK column gracefully if not installed.
let ClobClient = null;
try {
  ({ ClobClient } = await import("@polymarket/clob-client"));
} catch {
  console.error("note: @polymarket/clob-client not installed, OpenPX-only run");
}

const openpx = new Exchange("polymarket", {});
const sdk = ClobClient ? new ClobClient("https://clob.polymarket.com") : null;

const bench = new Bench({ time: 0, iterations: 20 });

// --- fetch_markets ---------------------------------------------------------
bench.add("openpx::polymarket::fetch_markets", async () => {
  await sleep(GAP_MS);
  await openpx.fetchMarkets();
});
if (sdk) {
  bench.add("polymarket-clob-client::fetch_markets", async () => {
    await sleep(GAP_MS);
    await sdk.getSamplingMarkets();
  });
}

// --- fetch_market ----------------------------------------------------------
bench.add("openpx::polymarket::fetch_market", async () => {
  await sleep(GAP_MS);
  await openpx.fetchMarket(CONDITION_ID);
});
if (sdk) {
  bench.add("polymarket-clob-client::fetch_market", async () => {
    await sleep(GAP_MS);
    await sdk.getMarket(CONDITION_ID);
  });
}

// --- fetch_orderbook -------------------------------------------------------
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

// --- fetch_trades ----------------------------------------------------------
bench.add("openpx::polymarket::fetch_trades", async () => {
  await sleep(GAP_MS);
  await openpx.fetchTrades(ASSET_ID);
});
if (sdk) {
  bench.add("polymarket-clob-client::fetch_trades", async () => {
    await sleep(GAP_MS);
    // Trade history is exposed via market trades on the SDK
    await sdk.getMarketTradesEvents(CONDITION_ID);
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
