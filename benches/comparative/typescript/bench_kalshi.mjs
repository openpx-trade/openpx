// Kalshi REST head-to-head: OpenPX vs (no official TypeScript SDK).
//
// Kalshi has no published TypeScript SDK on npm — OpenPX is the only
// TS client. We still emit the OpenPX-side number so the README can
// show absolute latency and the renderer can call out the absence.
//
// 20 iterations × 100 ms gap, polyfill methodology.
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

bench.add("openpx::kalshi::fetch_orderbook", async () => {
  await sleep(GAP_MS);
  await openpx.fetchOrderbook(TICKER);
});

if (sdkClient) {
  bench.add(`${sdkName}::kalshi::fetch_orderbook`, async () => {
    await sleep(GAP_MS);
    if (sdkClient.getMarketOrderbook)
      return sdkClient.getMarketOrderbook({ ticker: TICKER });
    if (sdkClient.markets?.getMarketOrderbook)
      return sdkClient.markets.getMarketOrderbook({ ticker: TICKER });
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
