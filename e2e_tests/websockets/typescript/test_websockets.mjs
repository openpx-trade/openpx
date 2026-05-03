// End-to-end WebSocket test for the TypeScript / NAPI SDK.
//
// Mirrors `e2e_tests/websockets/rust/test_websockets.rs` and the Python
// suite — exercises the unified `WebSocket` class against both exchanges
// to verify the Node binding hasn't drifted.
//
// Run from repo root:
//     just e2e-typescript websockets
//
// Output: writes ../results/typescript.json + pass/fail summary, exits
// non-zero if any case fails.

import { Exchange, WebSocket } from '../../../sdks/typescript/index.js';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, '../../..');
const RESULTS_DIR = path.join(REPO_ROOT, 'e2e_tests', 'websockets', 'results');
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

function trim(o) {
    return Object.fromEntries(Object.entries(o).filter(([, v]) => v));
}

function configFor(eid) {
    return eid === 'kalshi'
        ? trim({
            api_key_id: process.env.KALSHI_API_KEY_ID,
            private_key_path: process.env.KALSHI_PRIVATE_KEY_PATH,
        })
        : trim({
            private_key: process.env.POLYMARKET_PRIVATE_KEY,
            funder: process.env.POLYMARKET_FUNDER,
            signature_type: process.env.POLYMARKET_SIGNATURE_TYPE,
            api_key: process.env.POLYMARKET_API_KEY,
            api_secret: process.env.POLYMARKET_API_SECRET,
            api_passphrase: process.env.POLYMARKET_API_PASSPHRASE,
        });
}

const makeExchange = (eid) => new Exchange(eid, configFor(eid));
const makeWs = (eid) => new WebSocket(eid, configFor(eid));

// -----------------------------------------------------------------------
// Result tracking
// -----------------------------------------------------------------------

const results = [];

function record(name, expect, status, detail) {
    const entry = { name, expect, status, detail };
    results.push(entry);
    const sym = { pass: '✓', fail: '✗', skip: '·' }[status] || '?';
    console.log(`  ${sym} [${status.toUpperCase()}] ${name}`);
    if (status === 'fail' && detail) console.log(`      → ${String(detail).slice(0, 200)}`);
}

async function caseAsync(name, fn, expect = 'ok') {
    try {
        const r = await fn();
        if (r === 'skip') return record(name, expect, 'skip', 'skipped by test');
        record(name, expect, expect === 'err' ? 'fail' : 'pass', null);
    } catch (e) {
        const msg = String(e?.message || e).toLowerCase();
        if (expect === 'err') return record(name, expect, 'pass', `expected err: ${e}`);
        if (msg.includes('rate limit') || msg.includes('429') || msg.includes('timed out') || msg.includes('http error')) {
            return record(name, expect, 'skip', `transient: ${e}`);
        }
        record(name, expect, 'fail', `raised: ${e}`);
    }
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

async function discoverActiveMarket(eid) {
    const ex = makeExchange(eid);
    const page = await ex.fetchMarkets('active', undefined, undefined, undefined, undefined, 50);
    const markets = Array.isArray(page) ? page : page.markets || page;
    for (const m of markets.slice(0, 20)) {
        const token = (m.outcomes || []).map((o) => o.token_id).find(Boolean);
        const ticker = m.ticker;
        const assetId = token || ticker;
        if (!assetId) continue;
        try {
            const book = await ex.fetchOrderbook(assetId);
            if ((book.bids || []).length > 0 || (book.asks || []).length > 0) {
                return { subId: eid === 'kalshi' ? ticker : assetId, assetId };
            }
        } catch { /* keep trying */ }
    }
    return null;
}

function awaitFirstUpdate(ws, timeoutMs) {
    return new Promise((resolve) => {
        let resolved = false;
        const timer = setTimeout(() => {
            if (resolved) return;
            resolved = true;
            resolve(null);
        }, timeoutMs);
        // onUpdate is the NAPI binding's callback-based stream
        ws.onUpdate((err, update) => {
            if (resolved) return;
            resolved = true;
            clearTimeout(timer);
            resolve(err ? null : update);
        }).catch(() => {
            if (resolved) return;
            resolved = true;
            clearTimeout(timer);
            resolve(null);
        });
    });
}

// -----------------------------------------------------------------------
// Matrix
// -----------------------------------------------------------------------

async function runSingleMarket(eid) {
    await caseAsync(`${eid}.subscribe.snapshot_arrives`, async () => {
        const seed = await discoverActiveMarket(eid);
        if (!seed) return 'skip';
        const ws = makeWs(eid);
        await ws.connect();
        try {
            await ws.subscribe(seed.subId);
            const update = await awaitFirstUpdate(ws, 20000);
            if (!update) throw new Error(`${eid}: no WsUpdate within 20s`);
            if (!update.kind) throw new Error(`${eid}: update missing 'kind' discriminator`);
        } finally {
            try { await ws.disconnect(); } catch {}
        }
    });
}

async function runDisconnectClean(eid) {
    await caseAsync(`${eid}.connect_then_disconnect`, async () => {
        const ws = makeWs(eid);
        await ws.connect();
        await ws.disconnect();
        const s = ws.state;
        if (!['Closed', 'Disconnected'].includes(s)) {
            throw new Error(`${eid}: unexpected state after disconnect: ${s}`);
        }
    });
}

async function runBadMarketId(eid) {
    await caseAsync(`${eid}.subscribe.bad_market_id`, async () => {
        const ws = makeWs(eid);
        await ws.connect();
        const bogus = eid === 'kalshi' ? 'OPENPX-NOPE-NOEXIST-9999' : '0';
        try { await ws.subscribe(bogus); } catch { /* either ok */ }
        try { await ws.disconnect(); } catch {}
        // The contract is "no hang, no panic". Reaching here = pass.
    });
}

async function runPolymarketPublicNoAuth() {
    await caseAsync('polymarket.public_no_auth', async () => {
        const seed = await discoverActiveMarket('polymarket');
        if (!seed) return 'skip';
        const ws = new WebSocket('polymarket', {});
        await ws.connect();
        try {
            await ws.subscribe(seed.subId);
            const update = await awaitFirstUpdate(ws, 20000);
            if (!update) throw new Error('polymarket public: no WsUpdate within 20s');
        } finally {
            try { await ws.disconnect(); } catch {}
        }
    });
}

async function runKalshiNoAuthRejected() {
    await caseAsync('kalshi.no_auth_rejected', async () => {
        try {
            new WebSocket('kalshi', {});
        } catch {
            return; // expected
        }
        throw new Error('kalshi WebSocket without credentials must reject');
    });
}

// -----------------------------------------------------------------------
// Driver
// -----------------------------------------------------------------------

async function main() {
    console.log('== TypeScript WS e2e ==');
    for (const eid of ['kalshi', 'polymarket']) {
        console.log(`\n# ${eid}`);
        await runSingleMarket(eid);
        await runDisconnectClean(eid);
        await runBadMarketId(eid);
    }
    console.log('\n# cross-exchange contract');
    await runPolymarketPublicNoAuth();
    await runKalshiNoAuthRejected();

    const summary = {
        pass: results.filter((r) => r.status === 'pass').length,
        fail: results.filter((r) => r.status === 'fail').length,
        skip: results.filter((r) => r.status === 'skip').length,
        total: results.length,
    };
    fs.writeFileSync(RESULTS_PATH, JSON.stringify({ summary, cases: results }, null, 2));
    console.log(`\n${summary.pass} pass / ${summary.fail} fail / ${summary.skip} skip (${summary.total} total)`);
    console.log(`results -> ${path.relative(REPO_ROOT, RESULTS_PATH)}`);
    process.exit(summary.fail === 0 ? 0 : 1);
}

main().catch((e) => {
    console.error('fatal:', e);
    process.exit(2);
});
