// OpenPX vs `@polymarket/clob-client` — end-to-end orderbook fetch on
// Polymarket. Token id comes from `nextActiveMarketInSeries`, so the
// bench always hits whatever 5-min BTC market is open right now.
//
// Real network, real bytes, no credentials. Walltime is the only
// Codspeed instrument that applies to JS — `withCodSpeed` activates
// instrumented mode under `codspeed exec`, otherwise the bench falls
// through to vanilla tinybench wall-clock so local runs work too.
//
// Run via:
//   npx @codspeed/tinybench-plugin run bench_polymarket.mjs
// Or for a plain local run (no Codspeed dashboard upload):
//   node bench_polymarket.mjs

import { Bench } from "tinybench";
import { withCodSpeed } from "@codspeed/tinybench-plugin";
// Relative import — CI builds the TS SDK in place via `napi build --release`
// and we point straight at the resulting `index.js`.
import { Exchange } from "../../../sdks/typescript/index.js";

const POLYMARKET_SERIES = "btc-up-or-down-5m";

const openpx = new Exchange("polymarket", {});
const market = await openpx.nextActiveMarketInSeries(POLYMARKET_SERIES);
if (!market || !market.outcomes?.length) {
  console.error(`no active polymarket market in series '${POLYMARKET_SERIES}'`);
  process.exit(1);
}
const tokenId = market.outcomes.find((o) => o.token_id)?.token_id;
if (!tokenId) {
  console.error("active market has no outcome token_ids");
  process.exit(1);
}

const bench = withCodSpeed(new Bench({ time: 2000 }));

bench.add("openpx::polymarket::fetch_orderbook", async () => {
  await openpx.fetchOrderbook(tokenId);
});

// Optional comparison target. `npm install --no-save @polymarket/clob-client`
// before running for a head-to-head; otherwise this row is omitted.
try {
  const mod = await import("@polymarket/clob-client");
  const ClobClient = mod.ClobClient ?? mod.default?.ClobClient;
  if (ClobClient) {
    const pclob = new ClobClient("https://clob.polymarket.com", 137);
    bench.add("polymarket-clob-client::fetch_orderbook", async () => {
      await pclob.getOrderBook(tokenId);
    });
  }
} catch {
  // optional dep missing — leave openpx-only row in the output
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
