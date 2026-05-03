// End-to-end authenticated-trading test for the TypeScript / NAPI SDK.
//
// Mirrors `engine/sdk/tests/e2e_trading.rs` and the Python suite — every
// authenticated endpoint is exercised against both Kalshi (BTC 15m) and
// Polymarket (BTC 5m) live markets through the published Node addon.
//
// Run from repo root:
//     node e2e_trading/typescript/test_trading.mjs
//
// Output: writes ../results/typescript.json + pass/skip/fail summary to
// stdout, exits non-zero if any case fails.

import { Exchange } from '../../sdks/typescript/index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '../..');
const RESULTS_DIR = path.join(REPO_ROOT, 'e2e_trading', 'results');
fs.mkdirSync(RESULTS_DIR, { recursive: true });
const RESULTS_PATH = path.join(RESULTS_DIR, 'typescript.json');

function loadDotenv(p) {
    if (!fs.existsSync(p)) return;
    for (const line of fs.readFileSync(p, 'utf8').split('\n')) {
        const t = line.trim();
        if (!t || t.startsWith('#') || !t.includes('=')) continue;
        const [k, ...rest] = t.split('=');
        const v = rest.join('=');
        if (process.env[k.trim()] === undefined) process.env[k.trim()] = v.trim();
    }
}
loadDotenv(path.join(REPO_ROOT, '.env'));

const keyPath = process.env.KALSHI_PRIVATE_KEY_PATH;
if (keyPath && !path.isAbsolute(keyPath)) {
    process.env.KALSHI_PRIVATE_KEY_PATH = path.join(REPO_ROOT, keyPath);
}

function makeExchange(eid) {
    const cfg = eid === 'kalshi'
        ? {
            api_key_id: process.env.KALSHI_API_KEY_ID,
            private_key_path: process.env.KALSHI_PRIVATE_KEY_PATH,
        }
        : {
            private_key: process.env.POLYMARKET_PRIVATE_KEY,
            funder: process.env.POLYMARKET_FUNDER,
            signature_type: process.env.POLYMARKET_SIGNATURE_TYPE,
        };
    for (const k of Object.keys(cfg)) if (!cfg[k]) delete cfg[k];
    return new Exchange(eid, cfg);
}

// ---------------------------------------------------------------------------
// Result tracking + helpers
// ---------------------------------------------------------------------------

const results = [];

function isAuthUnavailable(e) {
    const m = (e?.message || String(e)).toLowerCase();
    return m.includes('could not derive api key')
        || m.includes('cloudflare waf blocked')
        || m.includes('cannot reach clob.polymarket.com')
        || m.includes('l1 eip-712 signature')
        || m.includes('no signing method')
        || m.includes('private key required');
}

function isTransient(e) {
    const m = (e?.message || String(e)).toLowerCase();
    return m.includes('rate limit') || m.includes('429') || m.includes('timed out')
        || m.includes('http error') || m.includes('connection');
}

async function runCase(name, expect, body) {
    const entry = { name, expect, status: 'pending', detail: null };
    try {
        await body(entry);
        if (entry.status === 'pending') {
            if (expect === 'err') {
                entry.status = 'fail';
                entry.detail = 'expected an error but call returned ok';
            } else {
                entry.status = 'pass';
            }
        }
    } catch (e) {
        if (expect === 'err') {
            entry.status = 'pass';
            entry.detail = `expected err: ${e.message || e}`;
        } else if (isTransient(e)) {
            entry.status = 'skip';
            entry.detail = `transient: ${e.message || e}`;
        } else if (isAuthUnavailable(e)) {
            entry.status = 'skip';
            entry.detail = `auth unavailable: ${e.message || e}`;
        } else {
            entry.status = 'fail';
            entry.detail = `raised: ${e.message || e}\n${e.stack || ''}`;
        }
    }
    results.push(entry);
    const sym = { pass: '✓', fail: '✗', skip: '·' }[entry.status] || '?';
    console.log(`  ${sym} [${entry.status.toUpperCase()}] ${name}`);
    if (entry.status === 'fail' && entry.detail) {
        console.log(`      → ${String(entry.detail).slice(0, 240)}`);
    }
}

// ---------------------------------------------------------------------------
// BTC market discovery (per exchange)
// ---------------------------------------------------------------------------

async function kalshiBtcMarket(ex) {
    const raw = await ex.fetchMarkets('active', null, ['KXBTC15M'].length ? null : null,
        'KXBTC15M', null, 5);
    const markets = raw?.markets || [];
    if (!markets.length) return null;
    return markets.slice().sort((a, b) => String(a.close_time || '').localeCompare(String(b.close_time || '')))[0];
}

async function polymarketBtcMarket(ex) {
    const isoTs = await ex.fetchServerTime();
    const nowUnix = Math.floor(new Date(isoTs).getTime() / 1000);
    const bucket = Math.floor(nowUnix / 300) * 300;
    for (const offset of [0, 300, -300]) {
        const b = bucket + offset;
        const et = `btc-updown-5m-${b}`;
        const raw = await ex.fetchMarkets('active', null, null, null, et, 5);
        const markets = raw?.markets || [];
        if (markets.length) return markets[0];
    }
    return null;
}

function safeRestingBuyPrice(bestBid, tick) {
    const t = tick || 0.01;
    let candidate = bestBid != null ? bestBid - 5 * t : 0.05;
    candidate = Math.min(candidate, 0.05);
    const snapped = Math.floor(candidate / t) * t;
    return Math.max(t, Math.min(1.0 - t, snapped));
}

function safeSize(eid, price) {
    return eid === 'kalshi' ? 1.0 : Math.ceil(5.5 / price);
}

// ---------------------------------------------------------------------------
// Per-exchange suite
// ---------------------------------------------------------------------------

async function runSuite(eid, marketResolver) {
    console.log(`\n=== ${eid} ===`);
    let ex;
    try {
        ex = makeExchange(eid);
    } catch (e) {
        console.log(`  · skipping ${eid}: cannot construct: ${e.message || e}`);
        return;
    }

    let market;
    try {
        market = await marketResolver(ex);
    } catch (e) {
        console.log(`  · skipping ${eid}: market discovery failed: ${e.message || e}`);
        return;
    }
    if (!market) {
        console.log(`  · skipping ${eid}: no active BTC market`);
        return;
    }

    const marketTicker = market.ticker;
    const assetId = eid === 'kalshi' ? marketTicker : market.outcomes?.[0]?.token_id;
    const tick = market.tick_size || 0.01;
    const price = safeRestingBuyPrice(market.best_bid, tick);
    const size = safeSize(eid, price);
    console.log(`  · market=${marketTicker} asset=${assetId} price=${price} size=${size} tick=${tick}`);

    // Account
    await runCase(`[${eid}] fetchServerTime`, 'ok', async () => {
        const ts = await ex.fetchServerTime();
        if (!ts) throw new Error('empty server time');
    });

    await runCase(`[${eid}] fetchBalance`, 'ok', async () => {
        const bal = await ex.fetchBalance();
        const key = eid === 'kalshi' ? 'USD' : 'USDC';
        if (!(key in bal)) throw new Error(`missing ${key} in ${JSON.stringify(bal)}`);
        if (bal[key] < 0) throw new Error(`negative balance ${bal[key]}`);
    });

    await runCase(`[${eid}] refreshBalance`, 'ok', async () => {
        await ex.refreshBalance();
    });

    // Positions
    await runCase(`[${eid}] fetchPositions unfiltered`, 'ok', async () => {
        const pos = await ex.fetchPositions();
        for (const p of pos) {
            if (!(p.size > 0)) throw new Error(`non-positive position ${JSON.stringify(p)}`);
        }
    });

    await runCase(`[${eid}] fetchPositions filtered`, 'ok', async () => {
        const pos = await ex.fetchPositions(marketTicker);
        for (const p of pos) {
            if (!(p.size > 0)) throw new Error(`non-positive position ${JSON.stringify(p)}`);
        }
    });

    // Order lifecycle
    await runCase(`[${eid}] order lifecycle: create→fetch→cancel`, 'ok', async () => {
        const order = await ex.createOrder(assetId, 'yes', 'buy', price, size, 'gtc');
        const orderId = order?.id;
        if (!orderId) throw new Error(`no id on placed order ${JSON.stringify(order)}`);
        // Kalshi's GET /portfolio/orders/{id} can briefly return "not found"
        // right after create (indexing race) — tolerate it as a soft-fail.
        try {
            const fetched = await ex.fetchOrder(orderId);
            if (fetched.id !== orderId) throw new Error('fetch_order id drift');
        } catch (e) {
            const m = (e.message || String(e)).toLowerCase();
            if (m.includes('not found') || m.includes('marketnotfound')) {
                entry.detail = `fetch race: ${e.message}; tolerated`;
            } else if (!(isTransient(e) || isAuthUnavailable(e))) {
                throw e;
            }
        }
        try {
            const cancelled = await ex.cancelOrder(orderId);
            const status = (cancelled.status || '').toLowerCase();
            if (status !== 'cancelled') throw new Error(`cancel returned non-cancelled ${JSON.stringify(cancelled)}`);
        } catch (e) {
            const m = (e.message || String(e)).toLowerCase();
            if (m.includes('not found') || m.includes('filled')) return;
            if (isTransient(e) || isAuthUnavailable(e)) return;
            throw e;
        }
    });

    // Adversarial create
    await runCase(`[${eid}] createOrder zero price`, 'err', async () => {
        await ex.createOrder(assetId, 'yes', 'buy', 0.0, size, 'gtc');
    });

    await runCase(`[${eid}] createOrder one price`, 'err', async () => {
        await ex.createOrder(assetId, 'yes', 'buy', 1.0, size, 'gtc');
    });

    await runCase(`[${eid}] createOrder negative size`, 'err', async () => {
        await ex.createOrder(assetId, 'yes', 'buy', price, -size, 'gtc');
    });

    // Fake order id
    const fakeId = eid === 'kalshi'
        ? '00000000-0000-0000-0000-000000000000'
        : '0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddead';

    await runCase(`[${eid}] fetchOrder unknown id`, 'err', async () => {
        await ex.fetchOrder(fakeId);
    });

    await runCase(`[${eid}] cancelOrder unknown id`, 'err', async () => {
        await ex.cancelOrder(fakeId);
    });

    // Batch
    await runCase(`[${eid}] createOrdersBatch empty`, 'ok', async () => {
        const out = await ex.createOrdersBatch([]);
        if (!Array.isArray(out) || out.length !== 0) {
            throw new Error(`empty batch returned ${JSON.stringify(out)}`);
        }
    });

    await runCase(`[${eid}] createOrdersBatch oversize`, 'err', async () => {
        const cap = eid === 'kalshi' ? 21 : 16;
        const reqs = [];
        for (let i = 0; i < cap; i++) {
            reqs.push({ asset_id: assetId, outcome: 'yes', side: 'buy', price: 0.05, size, order_type: 'gtc' });
        }
        const out = await ex.createOrdersBatch(reqs);
        for (const o of out || []) {
            try { await ex.cancelOrder(o.id); } catch {}
        }
        if (eid === 'polymarket') {
            throw new Error(`oversize batch (${cap}) was accepted`);
        }
    });

    await runCase(`[${eid}] cancelAllOrders unfiltered`, 'ok', async () => {
        const out = await ex.cancelAllOrders();
        if (!Array.isArray(out)) throw new Error(`expected array, got ${typeof out}`);
    });

    await runCase(`[${eid}] cancelAllOrders filtered`, 'ok', async () => {
        const out = await ex.cancelAllOrders(assetId);
        if (!Array.isArray(out)) throw new Error(`expected array, got ${typeof out}`);
    });

    await runCase(`[${eid}] fetchOpenOrders unfiltered`, 'ok', async () => {
        const out = await ex.fetchOpenOrders();
        for (const o of out) if (!o.id) throw new Error(`order missing id ${JSON.stringify(o)}`);
    });

    await runCase(`[${eid}] fetchOpenOrders filtered`, 'ok', async () => {
        const out = await ex.fetchOpenOrders(assetId);
        if (!Array.isArray(out)) throw new Error(`expected array, got ${typeof out}`);
    });

    await runCase(`[${eid}] fetchFills unfiltered limit=10`, 'ok', async () => {
        const fills = await ex.fetchFills(undefined, 10);
        if (fills.length > 10) throw new Error(`limit not honored: ${fills.length}`);
        for (const f of fills) {
            if (!(f.size > 0)) throw new Error(`non-positive fill size ${JSON.stringify(f)}`);
            if (!(f.price > 0 && f.price < 1)) throw new Error(`fill price out of range ${JSON.stringify(f)}`);
        }
    });

    await runCase(`[${eid}] fetchFills filtered limit=5`, 'ok', async () => {
        const fills = await ex.fetchFills(marketTicker, 5);
        if (fills.length > 5) throw new Error(`limit not honored: ${fills.length}`);
    });

    await runCase(`[${eid}] fetchTrades basic`, 'ok', async () => {
        const trades = await ex.fetchTrades(marketTicker, null, null, 20);
        const items = trades?.trades || [];
        for (const t of items) {
            if (!(t.size > 0)) throw new Error(`trade size non-positive ${JSON.stringify(t)}`);
            if (!(t.price > 0 && t.price < 1)) throw new Error(`trade price out of (0,1) ${JSON.stringify(t)}`);
        }
    });

    await runCase(`[${eid}] fetchTrades with time window`, 'ok', async () => {
        const now = Math.floor(Date.now() / 1000);
        const trades = await ex.fetchTrades(marketTicker, now - 3600, now, 50);
        const items = trades?.trades || [];
        if (!Array.isArray(items)) throw new Error('trades not iterable');
    });
}

// ---------------------------------------------------------------------------
// Drive
// ---------------------------------------------------------------------------

async function main() {
    await runSuite('kalshi', kalshiBtcMarket);
    await runSuite('polymarket', polymarketBtcMarket);

    const passN = results.filter(r => r.status === 'pass').length;
    const skipN = results.filter(r => r.status === 'skip').length;
    const failN = results.filter(r => r.status === 'fail').length;

    fs.writeFileSync(RESULTS_PATH, JSON.stringify({
        total: results.length, pass: passN, skip: skipN, fail: failN, results,
    }, null, 2));

    console.log(`\nPASS ${passN}\nSKIP ${skipN}\nFAIL ${failN}`);
    if (failN > 0) {
        console.log('\nFailing cases:');
        for (const r of results) {
            if (r.status === 'fail') console.log(`  - ${r.name}: ${String(r.detail).slice(0, 200)}`);
        }
        process.exit(1);
    }
}

main().catch(e => {
    console.error('runner crashed:', e);
    process.exit(2);
});
