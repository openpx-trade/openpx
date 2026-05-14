// Kalshi REST API head-to-head: OpenPX vs kalshi-typescript-sdk.
//
// 20 iterations × 100 ms gap. Writes JSON summary to stdout.
//
// Run: `node bench_kalshi.mjs > ../results/typescript_kalshi.json`

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { Bench } from "tinybench";
import { Exchange } from "@openpx/sdk";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const META = JSON.parse(
  fs.readFileSync(
    path.join(__dirname, "..", "fixtures", "kalshi_orderbook.meta.json"),
    "utf8"
  )
);
const TICKER = META.ticker;
const GAP_MS = 100;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Try the official Kalshi SDK; fall back to OpenPX-only.
let sdkClient = null;
let sdkName = null;
for (const pkg of [
  "kalshi-typescript",
  "kalshi-typescript-sdk",
  "kalshi-ts",
  "@kalshi/sdk",
]) {
  try {
    const mod = await import(pkg);
    sdkClient = mod.default ?? mod.KalshiClient ?? mod.MarketsApi ?? mod;
    sdkName = pkg;
    break;
  } catch {
    /* try next */
  }
}
if (!sdkClient) {
  console.error("note: no kalshi SDK installed, OpenPX-only run");
}

const openpx = new Exchange("kalshi", {});
const bench = new Bench({ time: 0, iterations: 20 });

bench.add("openpx::kalshi::fetch_markets", async () => {
  await sleep(GAP_MS);
  await openpx.fetchMarkets();
});

bench.add("openpx::kalshi::fetch_market", async () => {
  await sleep(GAP_MS);
  await openpx.fetchMarket(TICKER);
});

bench.add("openpx::kalshi::fetch_orderbook", async () => {
  await sleep(GAP_MS);
  await openpx.fetchOrderbook(TICKER);
});

bench.add("openpx::kalshi::fetch_trades", async () => {
  await sleep(GAP_MS);
  await openpx.fetchTrades(TICKER);
});

// Kalshi SDK shapes vary across packages; resolve methods defensively.
if (sdkClient) {
  const tagged = (op) => `${sdkName}::kalshi::${op}`;

  bench.add(tagged("fetch_markets"), async () => {
    await sleep(GAP_MS);
    if (sdkClient.getMarkets) return sdkClient.getMarkets({ limit: 100 });
    if (sdkClient.markets?.getMarkets)
      return sdkClient.markets.getMarkets({ limit: 100 });
  });

  bench.add(tagged("fetch_market"), async () => {
    await sleep(GAP_MS);
    if (sdkClient.getMarket) return sdkClient.getMarket({ ticker: TICKER });
    if (sdkClient.markets?.getMarket)
      return sdkClient.markets.getMarket({ ticker: TICKER });
  });

  bench.add(tagged("fetch_orderbook"), async () => {
    await sleep(GAP_MS);
    if (sdkClient.getMarketOrderbook)
      return sdkClient.getMarketOrderbook({ ticker: TICKER });
    if (sdkClient.markets?.getMarketOrderbook)
      return sdkClient.markets.getMarketOrderbook({ ticker: TICKER });
  });

  bench.add(tagged("fetch_trades"), async () => {
    await sleep(GAP_MS);
    if (sdkClient.getTrades)
      return sdkClient.getTrades({ ticker: TICKER, limit: 100 });
    if (sdkClient.markets?.getTrades)
      return sdkClient.markets.getTrades({ ticker: TICKER, limit: 100 });
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
