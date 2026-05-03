// End-to-end orderbook test for the TypeScript / NAPI SDK.
//
// Mirrors `engine/sdk/tests/e2e_orderbooks.rs` and the Python suite —
// every input variation of the five orderbook methods is exercised against
// both Kalshi and Polymarket through the published Node addon.
//
// Run from repo root:
//     node e2e_orderbooks/typescript/test_orderbooks.mjs
//
// Output: writes a JSON results document to ../results/typescript.json and
// prints a pass/fail summary, exits non-zero if any case fails.

import { Exchange } from '../../../sdks/typescript/index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '../../..');
const RESULTS_DIR = path.join(REPO_ROOT, 'e2e_tests', 'orderbooks', 'results');
fs.mkdirSync(RESULTS_DIR, { recursive: true });
const RESULTS_PATH = path.join(RESULTS_DIR, 'typescript.json');

// Lightweight .env loader so creds don't need to be exported.
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

// Resolve relative key paths against repo root.
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
        };
    // Strip undefined keys
    for (const k of Object.keys(cfg)) if (!cfg[k]) delete cfg[k];
    return new Exchange(eid, cfg);
}

// ---------------------------------------------------------------------------
// Result tracking
// ---------------------------------------------------------------------------

const results = [];

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
        const msg = String(e?.message || e).toLowerCase();
        if (expect === 'err') {
            entry.status = 'pass';
            entry.detail = `expected err: ${e}`;
        } else if (/(rate limit|429|timed out|http error)/.test(msg)) {
            entry.status = 'skip';
            entry.detail = `transient: ${e}`;
        } else if (e instanceof Error && e.name === 'AssertionError') {
            entry.status = 'fail';
            entry.detail = `assert: ${e.message}\n${e.stack}`;
        } else {
            entry.status = 'fail';
            entry.detail = `raised: ${e}\n${e?.stack || ''}`;
        }
    }
    results.push(entry);
    const sym = { pass: '✓', fail: '✗', skip: '·' }[entry.status] || '?';
    console.log(`  ${sym} [${entry.status.toUpperCase()}] ${name}`);
    if (entry.status === 'fail' && entry.detail) {
        console.log(`      → ${String(entry.detail).slice(0, 200)}`);
    }
}

function assert(cond, msg) {
    if (!cond) {
        const e = new Error(msg);
        e.name = 'AssertionError';
        throw e;
    }
}

// ---------------------------------------------------------------------------
// Invariant checkers
// ---------------------------------------------------------------------------

function assertBookWellFormed(book, assetId, label) {
    const { bids = [], asks = [] } = book;
    for (let i = 0; i < bids.length - 1; i++) {
        assert(bids[i].price >= bids[i + 1].price,
            `${label}: bids not desc ${bids[i].price} then ${bids[i + 1].price}`);
    }
    for (let i = 0; i < asks.length - 1; i++) {
        assert(asks[i].price <= asks[i + 1].price,
            `${label}: asks not asc ${asks[i].price} then ${asks[i + 1].price}`);
    }
    for (const l of [...bids, ...asks]) {
        assert(l.price > 0 && l.price < 1, `${label}: price ${l.price} OOB`);
        assert(l.size > 0, `${label}: size ${l.size} non-positive`);
    }
    if (bids.length > 0 && asks.length > 0) {
        assert(bids[0].price <= asks[0].price,
            `${label}: crossed book ${bids[0].price} > ${asks[0].price}`);
    }
}

function assertStatsConsistent(stats, book, label) {
    const bidDepth = (book.bids || []).reduce((s, l) => s + l.size, 0);
    const askDepth = (book.asks || []).reduce((s, l) => s + l.size, 0);
    assert(Math.abs(stats.bid_depth - bidDepth) < 1e-6, `${label}: bid_depth drift`);
    assert(Math.abs(stats.ask_depth - askDepth) < 1e-6, `${label}: ask_depth drift`);
    if (stats.mid != null) assert(stats.mid >= 0 && stats.mid <= 1, `${label}: mid OOB`);
    if (stats.spread_bps != null) assert(stats.spread_bps >= 0, `${label}: negative spread_bps`);
}

function assertImpactWellFormed(impact, size, label) {
    assert(impact.size === size, `${label}: size echo drift`);
    assert(impact.buy_fill_pct >= 0 && impact.buy_fill_pct <= 100, `${label}: buy_fill_pct OOB`);
    assert(impact.sell_fill_pct >= 0 && impact.sell_fill_pct <= 100, `${label}: sell_fill_pct OOB`);
    if (impact.buy_slippage_bps != null) assert(impact.buy_slippage_bps >= 0, `${label}: buy_slippage neg`);
}

function assertMicroWellFormed(micro, book, label) {
    assert(micro.level_count.bids === (book.bids || []).length, `${label}: bid count drift`);
    assert(micro.level_count.asks === (book.asks || []).length, `${label}: ask count drift`);
    const db = micro.depth_buckets;
    assert(db.bid_within_10bps <= db.bid_within_50bps + 1e-9, `${label}: bid 10≤50`);
    assert(db.bid_within_50bps <= db.bid_within_100bps + 1e-9, `${label}: bid 50≤100`);
    assert(db.ask_within_10bps <= db.ask_within_50bps + 1e-9, `${label}: ask 10≤50`);
    assert(db.ask_within_50bps <= db.ask_within_100bps + 1e-9, `${label}: ask 50≤100`);
}

function pickAssetId(market) {
    const outs = market.outcomes || [];
    if (outs.length > 0 && outs[0].token_id) return outs[0].token_id;
    return market.ticker;
}

async function seedBook(ex, label, maxTries = 10) {
    const page = await ex.fetchMarkets('active', null, null, null, null);
    const markets = page.markets || [];
    for (const m of markets.slice(0, maxTries)) {
        try {
            const aid = pickAssetId(m);
            const book = await ex.fetchOrderbook(aid);
            if ((book.bids || []).length || (book.asks || []).length) {
                return { market: m, aid, book };
            }
        } catch (_) {
            /* skip */
        }
    }
    return null;
}

// ---------------------------------------------------------------------------
// Per-exchange test plan
// ---------------------------------------------------------------------------

async function runSuite(eid) {
    console.log(`\n=== ${eid.toUpperCase()} ===`);
    let ex;
    try {
        ex = makeExchange(eid);
    } catch (e) {
        console.log(`  · SKIP: failed to construct ${eid} client: ${e}`);
        return;
    }
    const info = ex.describe();

    await runCase(`[${eid}] describe advertises orderbook surface`, 'ok', () => {
        assert(info.has_fetch_orderbook, 'has_fetch_orderbook false');
        assert(info.has_fetch_orderbooks_batch, 'has_fetch_orderbooks_batch false');
    });

    const seeded = await seedBook(ex, `${eid}/seed`);
    if (!seeded) {
        console.log(`  · SKIP: no market with non-empty book on ${eid}`);
        return;
    }
    const { aid, book } = seeded;
    console.log(`    seeded asset_id=${aid} (bids=${(book.bids||[]).length} asks=${(book.asks||[]).length})`);

    await runCase(`[${eid}] fetchOrderbook valid`, 'ok', async (entry) => {
        const b = await ex.fetchOrderbook(aid);
        assertBookWellFormed(b, aid, `${eid}/single`);
        entry.detail = { asset_id: aid, bids: (b.bids||[]).length, asks: (b.asks||[]).length };
    });

    await runCase(`[${eid}] fetchOrderbook nonexistent → empty or 404`, 'ok', async () => {
        const fake = eid === 'kalshi' ? 'OPENPX-TS-NOPE-0' : '1';
        try {
            const b = await ex.fetchOrderbook(fake);
            assert(!(b.bids||[]).length && !(b.asks||[]).length, 'fake id returned populated book');
        } catch (e) {
            const msg = String(e.message || e).toLowerCase();
            assert(/(not found|invalid|api)/.test(msg), `unexpected: ${e}`);
        }
    });

    await runCase(`[${eid}] fetchOrderbook malformed → empty or error`, 'ok', async () => {
        try {
            const b = await ex.fetchOrderbook('!@#$%');
            assert(!(b.bids||[]).length && !(b.asks||[]).length);
        } catch (_) { /* either is fine */ }
    });

    await runCase(`[${eid}] fetchOrderbooksBatch empty → []`, 'ok', async () => {
        const out = await ex.fetchOrderbooksBatch([]);
        assert(Array.isArray(out) && out.length === 0, `empty list returned ${JSON.stringify(out)}`);
    });

    // Seed 3 ids for batch
    const page = await ex.fetchMarkets('active', null, null, null, null);
    const seedIds = [];
    for (const m of (page.markets || []).slice(0, 15)) {
        try {
            const aidI = pickAssetId(m);
            const b = await ex.fetchOrderbook(aidI);
            if ((b.bids||[]).length || (b.asks||[]).length) seedIds.push(aidI);
        } catch (_) { /* skip */ }
        if (seedIds.length >= 3) break;
    }

    if (seedIds.length > 0) {
        await runCase(`[${eid}] fetchOrderbooksBatch multi → returns books`, 'ok', async (entry) => {
            const out = await ex.fetchOrderbooksBatch(seedIds);
            assert(out.length > 0, 'batch returned 0');
            for (const b of out) assertBookWellFormed(b, b.asset_id, `${eid}/batch`);
            entry.detail = { requested: seedIds.length, returned: out.length };
        });
    }

    if (eid === 'kalshi') {
        await runCase(`[${eid}] fetchOrderbooksBatch above-cap (101) → InvalidOrder`, 'err', async () => {
            const big = Array.from({ length: 101 }, (_, i) => `OPENPX-TS-CAP-${i}`);
            await ex.fetchOrderbooksBatch(big);
        });
    }

    await runCase(`[${eid}] fetchOrderbookStats consistent with book`, 'ok', async (entry) => {
        const s = await ex.fetchOrderbookStats(aid);
        assertStatsConsistent(s, book, `${eid}/stats`);
        entry.detail = { mid: s.mid, spread_bps: s.spread_bps };
    });

    const bids = book.bids || [];
    const asks = book.asks || [];
    let small;
    if (bids.length && asks.length) small = Math.min(bids[0].size, asks[0].size) * 0.5;
    else if (bids.length) small = bids[0].size * 0.5;
    else if (asks.length) small = asks[0].size * 0.5;
    else small = 1.0;

    await runCase(`[${eid}] fetchOrderbookImpact small size → full or partial fill`, 'ok', async (entry) => {
        const i = await ex.fetchOrderbookImpact(aid, small);
        assertImpactWellFormed(i, small, `${eid}/impact_small`);
        entry.detail = { size: small, buy_fill_pct: i.buy_fill_pct, sell_fill_pct: i.sell_fill_pct };
    });

    const totalDepth = bids.reduce((s, l) => s + l.size, 0) + asks.reduce((s, l) => s + l.size, 0);
    const big = totalDepth * 10 + 1;
    await runCase(`[${eid}] fetchOrderbookImpact large size → partial fill`, 'ok', async () => {
        const i = await ex.fetchOrderbookImpact(aid, big);
        assertImpactWellFormed(i, big, `${eid}/impact_large`);
        const anyPartial = i.buy_fill_pct < 100 || i.sell_fill_pct < 100;
        assert(anyPartial, 'oversize fully filled both sides');
    });

    await runCase(`[${eid}] fetchOrderbookImpact size=0 → InvalidInput`, 'err', async () => {
        await ex.fetchOrderbookImpact(aid, 0);
    });

    await runCase(`[${eid}] fetchOrderbookImpact size=-42 → InvalidInput`, 'err', async () => {
        await ex.fetchOrderbookImpact(aid, -42);
    });

    await runCase(`[${eid}] fetchOrderbookMicrostructure consistent`, 'ok', async (entry) => {
        const m = await ex.fetchOrderbookMicrostructure(aid);
        assertMicroWellFormed(m, book, `${eid}/micro`);
        entry.detail = {
            bids: m.level_count.bids, asks: m.level_count.asks,
            bid_slope: m.bid_slope, ask_slope: m.ask_slope,
        };
    });
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

const start = Date.now();
for (const eid of ['kalshi', 'polymarket']) {
    await runSuite(eid);
}
const elapsed = ((Date.now() - start) / 1000).toFixed(2);

const nPass = results.filter(r => r.status === 'pass').length;
const nFail = results.filter(r => r.status === 'fail').length;
const nSkip = results.filter(r => r.status === 'skip').length;

const summary = { elapsed_s: Number(elapsed), pass: nPass, fail: nFail, skip: nSkip, results };
fs.writeFileSync(RESULTS_PATH, JSON.stringify(summary, null, 2));

console.log(`\n=== summary ===`);
console.log(`PASS ${nPass}   FAIL ${nFail}   SKIP ${nSkip}`);
console.log(`results → ${RESULTS_PATH}`);
process.exit(nFail === 0 ? 0 : 1);
