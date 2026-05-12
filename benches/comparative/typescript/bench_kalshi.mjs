// OpenPX vs `kalshi-typescript-sdk` — end-to-end orderbook fetch on
// Kalshi. Market ticker comes from `nextActiveMarketInSeries`, so the
// bench always hits whatever 15-min BTC market is open right now.
//
// Real network, real bytes, no credentials. Walltime is the only
// Codspeed instrument that applies to JS.
//
// Run via:
//   npx @codspeed/tinybench-plugin run bench_kalshi.mjs

import { Bench } from "tinybench";
import { withCodSpeed } from "@codspeed/tinybench-plugin";
import { Exchange } from "../../../sdks/typescript/index.js";

const KALSHI_SERIES = "KXBTC15M";

const openpx = new Exchange("kalshi", {});
const market = await openpx.nextActiveMarketInSeries(KALSHI_SERIES);
if (!market) {
  console.error(`no active kalshi market in series '${KALSHI_SERIES}'`);
  process.exit(1);
}
if (!market.openpx_id || !market.openpx_id.startsWith("kalshi:")) {
  console.error(`unexpected openpx_id: ${market.openpx_id}`);
  process.exit(1);
}
const ticker = market.openpx_id.slice("kalshi:".length);

const bench = withCodSpeed(new Bench({ time: 2000 }));

bench.add("openpx::kalshi::fetch_orderbook", async () => {
  await openpx.fetchOrderbook(ticker);
});

// Optional comparison target. Kalshi's official TS SDK ships under
// `kalshi-typescript` (an OpenAPI-generated axios client). Older/community
// names tried after as a fallback.
const kalshiCandidates = ["kalshi-typescript", "kalshi-typescript-sdk", "kalshi-ts", "@kalshi/sdk"];
for (const name of kalshiCandidates) {
  try {
    const mod = await import(name);
    // The OpenAPI generator emits `MarketsApi` / `Configuration` /
    // `BasePath` style exports. Probe the most likely orderbook entry
    // points across both that shape and any hand-rolled wrappers.
    let fn = null;
    // OpenAPI-generated TS clients tend to call this `MarketApi` (singular)
    // — that's what `kalshi-typescript@3.x` ships. `MarketsApi` (plural)
    // covers older / hand-written variants.
    const ApiClass = mod.MarketApi ?? mod.MarketsApi;
    if (ApiClass && mod.Configuration) {
      const config = new mod.Configuration({ basePath: "https://api.elections.kalshi.com/trade-api/v2" });
      const api = new ApiClass(config);
      fn = (api.getMarketOrderbook ?? api.get_market_orderbook)?.bind(api);
    } else {
      const KalshiClient = mod.Kalshi ?? mod.KalshiClient ?? mod.default;
      if (typeof KalshiClient === "function") {
        const k = new KalshiClient();
        fn = (k.getMarketOrderbook ?? k.getOrderbook ?? k.fetchOrderbook)?.bind(k);
      }
    }
    if (fn) {
      bench.add(`${name}::kalshi::fetch_orderbook`, async () => {
        await fn(ticker);
      });
      break;
    }
  } catch {
    // try next candidate
  }
}

await bench.run();

const tasks = bench.tasks.map((t) => {
  const r = t.result;
  return {
    name: t.name,
    samples: r?.samples?.length ?? 0,
    p50_ns: r?.p50 != null ? Math.round(r.p50 * 1e6) : null,
    p99_ns: r?.p99 != null ? Math.round(r.p99 * 1e6) : null,
    mean_ns: r?.mean != null ? Math.round(r.mean * 1e6) : null,
    hz: r?.hz ?? null,
  };
});

console.log(JSON.stringify({ tasks }, null, 2));
