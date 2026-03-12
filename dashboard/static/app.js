// ============================================================================
// OpenPX Trading Terminal — app.js
// ============================================================================

const CHART_COLORS = ['#00ff00', '#6699ff', '#ffcc00', '#ff3333', '#00cccc', '#ff66ff', '#ff9933', '#99ff33'];

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

const state = {
    exchanges: [],
    currentPage: 'terminal',
    // Terminal
    selectedExchange: null,
    lookupType: 'market', // 'market' or 'event'
    eventMarkets: [],     // UnifiedMarket[] for event view
    singleMarket: null,   // Market for single market view
    selectedMarketIdx: 0, // Which market is selected in event view
    chart: null,
    lineSeries: [],
    currentInterval: 'max',
    // Order panel
    orderSide: 'buy',
    orderOutcome: 'Yes',
    orderType: 'limit',
    balance: null,
    // Portfolio
    pnlChart: null,
    positionFilter: 'active',
    positionExchange: 'all',
};

// ---------------------------------------------------------------------------
// API Client
// ---------------------------------------------------------------------------

const api = {
    async request(method, path, body) {
        const opts = { method, headers: { 'Content-Type': 'application/json' } };
        if (body) opts.body = JSON.stringify(body);
        const res = await fetch(`/api${path}`, opts);
        if (!res.ok) {
            const err = await res.json().catch(() => ({ error: res.statusText }));
            throw new Error(err.error || res.statusText);
        }
        if (res.status === 204) return null;
        return res.json();
    },

    // Exchange management
    addExchange: (exchange, config) => api.request('POST', '/exchanges', { exchange, config }),
    listExchanges: () => api.request('GET', '/exchanges'),
    removeExchange: (id) => api.request('DELETE', `/exchanges/${id}`),

    // Market data
    fetchMarket: (ex, id) => api.request('GET', `/exchanges/${ex}/markets/${encodeURIComponent(id)}`),
    fetchEventMarkets: (ex, groupId) => api.request('GET', `/exchanges/${ex}/events/${encodeURIComponent(groupId)}`),
    fetchOrderbook: (ex, id, params = {}) => {
        const qs = new URLSearchParams(params).toString();
        return api.request('GET', `/exchanges/${ex}/markets/${encodeURIComponent(id)}/orderbook${qs ? '?' + qs : ''}`);
    },
    fetchPriceHistory: (ex, id, params = {}) => {
        const qs = new URLSearchParams(params).toString();
        return api.request('GET', `/exchanges/${ex}/markets/${encodeURIComponent(id)}/history${qs ? '?' + qs : ''}`);
    },
    fetchTrades: (ex, id, params = {}) => {
        const qs = new URLSearchParams(params).toString();
        return api.request('GET', `/exchanges/${ex}/markets/${encodeURIComponent(id)}/trades${qs ? '?' + qs : ''}`);
    },

    // Trading
    createOrder: (ex, order) => api.request('POST', `/exchanges/${ex}/orders`, order),
    fetchOrders: (ex) => api.request('GET', `/exchanges/${ex}/orders`),
    cancelOrder: (ex, orderId, marketId) => {
        const qs = marketId ? `?market_id=${encodeURIComponent(marketId)}` : '';
        return api.request('DELETE', `/exchanges/${ex}/orders/${encodeURIComponent(orderId)}${qs}`);
    },

    // Portfolio
    fetchPositions: (ex, marketId) => {
        const qs = marketId ? `?market_id=${encodeURIComponent(marketId)}` : '';
        return api.request('GET', `/exchanges/${ex}/positions${qs}`);
    },
    fetchBalance: (ex) => api.request('GET', `/exchanges/${ex}/balance`),
    fetchFills: (ex, params = {}) => {
        const qs = new URLSearchParams(params).toString();
        return api.request('GET', `/exchanges/${ex}/fills${qs ? '?' + qs : ''}`);
    },
    fetchAllPositions: () => api.request('GET', '/portfolio/positions'),
    fetchAllBalances: () => api.request('GET', '/portfolio/balances'),
};

// ---------------------------------------------------------------------------
// Toast notifications
// ---------------------------------------------------------------------------

function showToast(message, type = 'info') {
    let container = document.querySelector('.toast-container');
    if (!container) {
        container = document.createElement('div');
        container.className = 'toast-container';
        document.body.appendChild(container);
    }
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(() => toast.remove(), 4000);
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

function navigate(page) {
    state.currentPage = page;
    document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
    document.querySelectorAll('.nav-link').forEach(l => l.classList.remove('active'));
    document.getElementById(`page-${page}`).classList.add('active');
    document.querySelector(`.nav-link[data-page="${page}"]`).classList.add('active');

    if (page === 'portfolio') {
        refreshPortfolio();
    }
}

// ---------------------------------------------------------------------------
// Exchange selector (terminal page)
// ---------------------------------------------------------------------------

function updateExchangeSelect() {
    const select = document.getElementById('exchange-select');
    const current = select.value;
    select.innerHTML = '<option value="">Select Exchange</option>';
    state.exchanges.forEach(ex => {
        const opt = document.createElement('option');
        opt.value = ex.id;
        opt.textContent = ex.name || ex.id;
        select.appendChild(opt);
    });
    if (current && state.exchanges.find(e => e.id === current)) {
        select.value = current;
    }
}

// ---------------------------------------------------------------------------
// Terminal: Load market / event
// ---------------------------------------------------------------------------

async function loadMarketOrEvent() {
    const exchangeId = document.getElementById('exchange-select').value;
    const input = document.getElementById('market-input').value.trim();
    if (!exchangeId) return showToast('Select an exchange first', 'error');
    if (!input) return showToast('Enter a market or event ID', 'error');

    state.selectedExchange = exchangeId;
    const content = document.getElementById('terminal-content');

    try {
        if (state.lookupType === 'event') {
            const markets = await api.fetchEventMarkets(exchangeId, input);
            if (!markets || markets.length === 0) {
                return showToast('No markets found for this event', 'error');
            }
            state.eventMarkets = markets;
            state.singleMarket = null;
            state.selectedMarketIdx = 0;
            content.classList.remove('hidden');
            renderEventView(markets);
        } else {
            const market = await api.fetchMarket(exchangeId, input);
            state.singleMarket = market;
            state.eventMarkets = [];
            content.classList.remove('hidden');
            renderSingleMarketView(market, input);
        }
    } catch (err) {
        showToast(err.message, 'error');
    }
}

// ---------------------------------------------------------------------------
// Terminal: Render event view (multiple markets)
// ---------------------------------------------------------------------------

async function renderEventView(markets) {
    // Header
    const first = markets[0];
    renderMarketHeader(first.title || first.question || 'Event', first.image_url, first.close_time, null);

    // Chart with all outcome lines
    await renderEventChart(markets);

    // Market rows
    renderMarketRows(markets);

    // Select first market for order panel + orderbook
    selectMarket(0);
}

// ---------------------------------------------------------------------------
// Terminal: Render single market view
// ---------------------------------------------------------------------------

async function renderSingleMarketView(market, marketId) {
    renderMarketHeader(market.question, null, market.close_time, market.volume);

    // For single binary market, show Yes/No lines
    await renderSingleMarketChart(market, marketId);

    // No market rows for single market
    document.getElementById('market-rows').innerHTML = '';

    // Order panel
    updateOrderPanel(market, marketId);

    // Load orderbook
    loadOrderbook(state.selectedExchange, marketId);
    loadOpenOrders();
}

// ---------------------------------------------------------------------------
// Market header
// ---------------------------------------------------------------------------

function renderMarketHeader(title, imageUrl, closeTime, volume) {
    document.getElementById('market-title').textContent = title;
    const img = document.getElementById('market-image');
    if (imageUrl) {
        img.src = imageUrl;
        img.style.display = '';
    } else {
        img.style.display = 'none';
    }
    let meta = '';
    if (volume != null) meta += `Vol. $${formatNumber(volume)}`;
    if (closeTime) {
        if (meta) meta += ' | ';
        meta += `Closes ${new Date(closeTime).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}`;
    }
    document.getElementById('market-meta').textContent = meta;
}

// ---------------------------------------------------------------------------
// Chart rendering (event — multiple markets)
// ---------------------------------------------------------------------------

async function renderEventChart(markets) {
    const container = document.getElementById('chart-container');
    destroyChart();

    state.chart = LightweightCharts.createChart(container, chartOptions(container));
    state.lineSeries = [];

    const legend = document.getElementById('chart-legend');
    legend.innerHTML = '';

    for (let i = 0; i < markets.length; i++) {
        const m = markets[i];
        const color = CHART_COLORS[i % CHART_COLORS.length];
        const series = state.chart.addLineSeries({ color, lineWidth: 2, priceFormat: { type: 'custom', formatter: p => (p * 100).toFixed(1) + '%' } });
        state.lineSeries.push({ series, marketId: m.id, color });

        // Legend
        const item = document.createElement('div');
        item.className = 'legend-item';
        const price = m.outcome_prices && Object.values(m.outcome_prices)[0];
        const priceStr = price != null ? ` ${(price * 100).toFixed(1)}%` : '';
        item.innerHTML = `<span class="legend-dot" style="background:${color}"></span><span>${m.question || m.title}${priceStr}</span>`;
        legend.appendChild(item);

        // Fetch price history
        try {
            const candles = await api.fetchPriceHistory(state.selectedExchange, m.id, { interval: state.currentInterval });
            if (candles && candles.length) {
                series.setData(candles.map(c => ({ time: Math.floor(new Date(c.timestamp).getTime() / 1000), value: c.close })));
            }
        } catch {
            // Some markets may not support price history
        }
    }

    state.chart.timeScale().fitContent();
    setupChartResize(container);
}

// ---------------------------------------------------------------------------
// Chart rendering (single binary market)
// ---------------------------------------------------------------------------

async function renderSingleMarketChart(market, marketId) {
    const container = document.getElementById('chart-container');
    destroyChart();

    state.chart = LightweightCharts.createChart(container, chartOptions(container));
    state.lineSeries = [];

    const legend = document.getElementById('chart-legend');
    legend.innerHTML = '';

    // For binary markets, show Yes and No lines
    const outcomes = market.outcomes || ['Yes', 'No'];
    for (let i = 0; i < outcomes.length; i++) {
        const outcome = outcomes[i];
        const color = i === 0 ? '#00ff00' : '#ff3333';
        const series = state.chart.addLineSeries({ color, lineWidth: 2, priceFormat: { type: 'custom', formatter: p => (p * 100).toFixed(1) + '%' } });
        state.lineSeries.push({ series, outcome, color });

        const price = market.prices && market.prices[outcome];
        const priceStr = price != null ? ` ${(price * 100).toFixed(1)}%` : '';
        const item = document.createElement('div');
        item.className = 'legend-item';
        item.innerHTML = `<span class="legend-dot" style="background:${color}"></span><span>${outcome}${priceStr}</span>`;
        legend.appendChild(item);

        try {
            const tokenIds = market.metadata && market.metadata.clobTokenIds;
            let tokenId;
            if (tokenIds) {
                const ids = typeof tokenIds === 'string' ? JSON.parse(tokenIds) : tokenIds;
                tokenId = ids[i];
            }
            const params = { interval: state.currentInterval };
            if (outcome) params.outcome = outcome;
            if (tokenId) params.token_id = tokenId;
            const candles = await api.fetchPriceHistory(state.selectedExchange, marketId, params);
            if (candles && candles.length) {
                series.setData(candles.map(c => ({ time: Math.floor(new Date(c.timestamp).getTime() / 1000), value: c.close })));
            }
        } catch {
            // fallback: no data
        }
    }

    state.chart.timeScale().fitContent();
    setupChartResize(container);
    updateOrderPanel(market, marketId);
}

function chartOptions(container) {
    return {
        width: container.clientWidth,
        height: 400,
        layout: { background: { type: 'solid', color: '#000000' }, textColor: '#666666', fontFamily: 'Menlo, SF Mono, Monaco, Consolas, monospace', fontSize: 10 },
        grid: { vertLines: { color: '#1a1a1a' }, horzLines: { color: '#1a1a1a' } },
        timeScale: { borderColor: '#333333', timeVisible: true },
        rightPriceScale: { borderColor: '#333333' },
        crosshair: { mode: LightweightCharts.CrosshairMode.Normal },
    };
}

function destroyChart() {
    if (state.chart) {
        state.chart.remove();
        state.chart = null;
        state.lineSeries = [];
    }
}

function setupChartResize(container) {
    const observer = new ResizeObserver(() => {
        if (state.chart) state.chart.resize(container.clientWidth, 400);
    });
    observer.observe(container);
}

// ---------------------------------------------------------------------------
// Time range change
// ---------------------------------------------------------------------------

async function changeTimeRange(interval) {
    state.currentInterval = interval;
    document.querySelectorAll('.time-range-buttons .range-btn').forEach(b => b.classList.toggle('active', b.dataset.range === interval));

    if (!state.selectedExchange) return;

    // Re-fetch data for each series
    for (const s of state.lineSeries) {
        try {
            const params = { interval };
            if (s.outcome) params.outcome = s.outcome;
            const id = s.marketId || document.getElementById('market-input').value.trim();
            const candles = await api.fetchPriceHistory(state.selectedExchange, id, params);
            if (candles && candles.length) {
                s.series.setData(candles.map(c => ({ time: Math.floor(new Date(c.timestamp).getTime() / 1000), value: c.close })));
            }
        } catch { /* ignore */ }
    }
    if (state.chart) state.chart.timeScale().fitContent();
}

// ---------------------------------------------------------------------------
// Market rows (event view)
// ---------------------------------------------------------------------------

function renderMarketRows(markets) {
    const container = document.getElementById('market-rows');
    container.innerHTML = '';

    markets.forEach((m, i) => {
        const row = document.createElement('div');
        row.className = `market-row${i === state.selectedMarketIdx ? ' selected' : ''}`;
        row.onclick = () => selectMarket(i);

        const price = m.outcome_prices ? Object.values(m.outcome_prices)[0] : null;
        const priceStr = price != null ? (price * 100).toFixed(0) + '%' : '--';
        const change = m.price_change_1d;
        const changeStr = change != null ? (change > 0 ? '+' : '') + (change * 100).toFixed(0) + '%' : '';
        const changeClass = change > 0 ? 'text-green' : change < 0 ? 'text-red' : '';

        const vol = m.volume != null ? formatNumber(m.volume) : '--';
        const yesPrice = price != null ? (price * 100).toFixed(1) : '--';
        const noPrice = price != null ? ((1 - price) * 100).toFixed(1) : '--';

        row.innerHTML = `
            <div class="market-row-info">
                <div class="market-row-name">${m.question || m.title}</div>
                <div class="market-row-vol">$${vol} Vol.</div>
            </div>
            <div class="market-row-price">${priceStr}</div>
            <div class="market-row-change ${changeClass}">${changeStr}</div>
            <div class="market-row-buttons">
                <button class="buy-yes-btn" onclick="event.stopPropagation(); quickSelectMarket(${i}, 'Yes')">Buy Yes ${yesPrice}c</button>
                <button class="buy-no-btn" onclick="event.stopPropagation(); quickSelectMarket(${i}, 'No')">Buy No ${noPrice}c</button>
            </div>
        `;
        container.appendChild(row);
    });
}

function selectMarket(idx) {
    state.selectedMarketIdx = idx;
    document.querySelectorAll('.market-row').forEach((r, i) => r.classList.toggle('selected', i === idx));

    const m = state.eventMarkets[idx];
    if (!m) return;

    // Update order panel
    const market = {
        question: m.question || m.title,
        outcomes: m.outcomes || ['Yes', 'No'],
        prices: m.outcome_prices || {},
        volume: m.volume,
        tick_size: m.tick_size || 0.01,
    };
    updateOrderPanel(market, m.id);
    loadOrderbook(state.selectedExchange, m.id);
    loadOpenOrders();
}

function quickSelectMarket(idx, outcome) {
    selectMarket(idx);
    state.orderOutcome = outcome;
    document.getElementById('outcome-yes').classList.toggle('active', outcome === 'Yes');
    document.getElementById('outcome-no').classList.toggle('active', outcome === 'No');
}

// ---------------------------------------------------------------------------
// Order panel
// ---------------------------------------------------------------------------

function updateOrderPanel(market, marketId) {
    document.getElementById('order-market-name').textContent = market.question || 'Market';

    const img = document.getElementById('order-market-image');
    if (market.image_url) {
        img.src = market.image_url;
        img.style.display = '';
    } else {
        img.style.display = 'none';
    }

    // Prices
    const outcomes = market.outcomes || ['Yes', 'No'];
    const yesPrice = market.prices[outcomes[0]];
    const noPrice = outcomes.length > 1 ? market.prices[outcomes[1]] : (yesPrice != null ? 1 - yesPrice : null);

    document.getElementById('yes-price').textContent = yesPrice != null ? (yesPrice * 100).toFixed(1) + 'c' : '--';
    document.getElementById('no-price').textContent = noPrice != null ? (noPrice * 100).toFixed(1) + 'c' : '--';

    // Set limit price to current price of selected outcome
    const currentPrice = state.orderOutcome === 'Yes' ? yesPrice : noPrice;
    if (currentPrice != null) {
        document.getElementById('limit-price-input').value = currentPrice.toFixed(2);
    }

    // Store market id for order submission
    document.getElementById('submit-order').dataset.marketId = marketId;
    document.getElementById('submit-order').dataset.exchange = state.selectedExchange;

    updateOrderSummary();
    loadBalance();
}

function updateOrderSummary() {
    const amount = parseFloat(document.getElementById('amount-input').value) || 0;
    let price;
    if (state.orderType === 'market') {
        // Use current market price
        const priceEl = state.orderOutcome === 'Yes' ? document.getElementById('yes-price') : document.getElementById('no-price');
        price = parseFloat(priceEl.textContent) / 100;
    } else {
        price = parseFloat(document.getElementById('limit-price-input').value) || 0;
    }

    const shares = price > 0 ? amount / price : 0;
    const potentialReturn = state.orderSide === 'buy' ? shares * (1 - price) : amount; // simplified

    document.getElementById('shares-display').textContent = shares.toFixed(2);
    document.getElementById('avg-price-display').textContent = price > 0 ? (price * 100).toFixed(1) + 'c' : '--';
    document.getElementById('return-display').textContent = '$' + potentialReturn.toFixed(2);
    document.getElementById('return-display').className = potentialReturn >= 0 ? 'text-green' : 'text-red';
}

async function loadBalance() {
    if (!state.selectedExchange) return;
    try {
        const bal = await api.fetchBalance(state.selectedExchange);
        state.balance = bal;
        const total = Object.values(bal).reduce((s, v) => s + (typeof v === 'number' ? v : 0), 0);
        document.getElementById('balance-display').textContent = `Balance $${total.toFixed(2)}`;
    } catch {
        document.getElementById('balance-display').textContent = 'Balance --';
    }
}

async function submitOrder() {
    const btn = document.getElementById('submit-order');
    const exchange = btn.dataset.exchange;
    const marketId = btn.dataset.marketId;
    if (!exchange || !marketId) return showToast('No market selected', 'error');

    const amount = parseFloat(document.getElementById('amount-input').value) || 0;
    if (amount <= 0) return showToast('Enter an amount', 'error');

    let price;
    const params = {};
    if (state.orderType === 'market') {
        price = state.orderSide === 'buy' ? 0.99 : 0.01;
        params.order_type = 'ioc';
    } else {
        price = parseFloat(document.getElementById('limit-price-input').value) || 0;
        if (price <= 0 || price >= 1) return showToast('Price must be between 0.01 and 0.99', 'error');
        params.order_type = 'gtc';
    }

    const size = amount / price;

    btn.disabled = true;
    btn.textContent = 'Placing...';

    try {
        await api.createOrder(exchange, {
            market_id: marketId,
            outcome: state.orderOutcome,
            side: state.orderSide,
            price,
            size,
            params,
        });
        showToast('Order placed successfully', 'success');
        document.getElementById('amount-input').value = '0';
        updateOrderSummary();
        loadOpenOrders();
        loadBalance();
    } catch (err) {
        showToast(err.message, 'error');
    } finally {
        btn.disabled = false;
        btn.textContent = 'Place Order';
    }
}

// ---------------------------------------------------------------------------
// Orderbook
// ---------------------------------------------------------------------------

async function loadOrderbook(exchange, marketId) {
    const container = document.getElementById('orderbook-content');
    if (!exchange || !marketId) return;

    container.innerHTML = '<div class="spinner"></div>';

    try {
        const ob = await api.fetchOrderbook(exchange, marketId);
        renderOrderbook(ob);
    } catch (err) {
        container.innerHTML = `<p class="text-muted">${err.message}</p>`;
    }
}

function renderOrderbook(ob) {
    const container = document.getElementById('orderbook-content');

    if ((!ob.asks || ob.asks.length === 0) && (!ob.bids || ob.bids.length === 0)) {
        container.innerHTML = '<p class="text-muted">No orderbook data available</p>';
        return;
    }

    const asks = (ob.asks || []).slice(0, 15).reverse();
    const bids = (ob.bids || []).slice(0, 15);

    let html = '<table class="orderbook-table"><thead><tr><th>Price</th><th>Shares</th><th>Total</th></tr></thead><tbody>';

    // Asks (sells) — top of book
    let runningTotal = 0;
    for (const level of asks) {
        runningTotal += level.price * level.size;
        html += `<tr><td class="price-ask">${(level.price * 100).toFixed(1)}c</td><td>${formatNumber(level.size)}</td><td>$${formatNumber(runningTotal)}</td></tr>`;
    }

    // Spread
    const spread = ob.asks && ob.asks.length && ob.bids && ob.bids.length
        ? ((ob.asks[0].price - ob.bids[0].price) * 100).toFixed(1)
        : '--';
    html += `<tr class="orderbook-spread-row"><td colspan="3">Spread: ${spread}c</td></tr>`;

    // Bids (buys)
    runningTotal = 0;
    for (const level of bids) {
        runningTotal += level.price * level.size;
        html += `<tr><td class="price-bid">${(level.price * 100).toFixed(1)}c</td><td>${formatNumber(level.size)}</td><td>$${formatNumber(runningTotal)}</td></tr>`;
    }

    html += '</tbody></table>';
    container.innerHTML = html;
}

// ---------------------------------------------------------------------------
// Recent trades
// ---------------------------------------------------------------------------

async function loadTrades() {
    const exchange = state.selectedExchange;
    const marketId = getCurrentMarketId();
    const container = document.getElementById('trades-content');
    if (!exchange || !marketId) return;

    container.innerHTML = '<div class="spinner"></div>';

    try {
        const data = await api.fetchTrades(exchange, marketId, { limit: 50 });
        renderTrades(data.trades || []);
    } catch (err) {
        container.innerHTML = `<p class="text-muted">${err.message}</p>`;
    }
}

function renderTrades(trades) {
    const container = document.getElementById('trades-content');
    if (!trades.length) {
        container.innerHTML = '<p class="text-muted">No recent trades</p>';
        return;
    }

    let html = '<table class="trades-table"><thead><tr><th>Price</th><th>Size</th><th>Side</th><th>Time</th></tr></thead><tbody>';
    for (const t of trades) {
        const side = t.side || t.aggressor_side || '--';
        const sideClass = side.toLowerCase() === 'buy' ? 'text-green' : side.toLowerCase() === 'sell' ? 'text-red' : '';
        const time = t.timestamp ? new Date(t.timestamp).toLocaleTimeString() : '--';
        html += `<tr>
            <td>${(t.price * 100).toFixed(1)}c</td>
            <td>${formatNumber(t.size)}</td>
            <td class="${sideClass}">${side}</td>
            <td>${time}</td>
        </tr>`;
    }
    html += '</tbody></table>';
    container.innerHTML = html;
}

// ---------------------------------------------------------------------------
// Open orders
// ---------------------------------------------------------------------------

async function loadOpenOrders() {
    const exchange = state.selectedExchange;
    const container = document.getElementById('open-orders-content');
    if (!exchange) return;

    try {
        const orders = await api.fetchOrders(exchange);
        renderOpenOrders(orders || []);
    } catch (err) {
        container.innerHTML = `<p class="text-muted">${err.message}</p>`;
    }
}

function renderOpenOrders(orders) {
    const container = document.getElementById('open-orders-content');
    if (!orders.length) {
        container.innerHTML = '<p class="text-muted">No open orders</p>';
        return;
    }

    let html = '<table class="orders-table"><thead><tr><th>Market</th><th>Side</th><th>Outcome</th><th>Price</th><th>Size</th><th>Filled</th><th>Status</th><th></th></tr></thead><tbody>';
    for (const o of orders) {
        const sideClass = o.side === 'buy' ? 'text-green' : 'text-red';
        html += `<tr>
            <td>${truncate(o.market_id, 16)}</td>
            <td class="${sideClass}">${o.side}</td>
            <td>${o.outcome}</td>
            <td>${(o.price * 100).toFixed(1)}c</td>
            <td>${o.size.toFixed(2)}</td>
            <td>${o.filled.toFixed(2)}</td>
            <td>${o.status}</td>
            <td><button class="btn-cancel" onclick="cancelOrderHandler('${o.id}', '${o.market_id}')">Cancel</button></td>
        </tr>`;
    }
    html += '</tbody></table>';
    container.innerHTML = html;
}

async function cancelOrderHandler(orderId, marketId) {
    try {
        await api.cancelOrder(state.selectedExchange, orderId, marketId);
        showToast('Order cancelled', 'success');
        loadOpenOrders();
    } catch (err) {
        showToast(err.message, 'error');
    }
}

// ---------------------------------------------------------------------------
// Portfolio: Exchange management
// ---------------------------------------------------------------------------

const EXCHANGE_FIELDS = {
    kalshi: [
        { name: 'api_key_id', label: 'API Key ID', type: 'text' },
        { name: 'private_key_pem', label: 'Private Key (PEM)', type: 'textarea' },
        { name: 'demo', label: 'Demo Mode', type: 'checkbox' },
    ],
    polymarket: [
        { name: 'private_key', label: 'Private Key (0x...)', type: 'password' },
        { name: 'api_key', label: 'API Key', type: 'text' },
        { name: 'api_secret', label: 'API Secret', type: 'password' },
        { name: 'api_passphrase', label: 'API Passphrase', type: 'password' },
    ],
    opinion: [
        { name: 'api_key', label: 'API Key', type: 'text' },
        { name: 'private_key', label: 'Private Key (0x...)', type: 'password' },
        { name: 'multi_sig_addr', label: 'Multi-Sig Address', type: 'text' },
    ],
    limitless: [
        { name: 'private_key', label: 'Private Key (0x...)', type: 'password' },
    ],
    predictfun: [
        { name: 'api_key', label: 'API Key', type: 'text' },
        { name: 'private_key', label: 'Private Key (0x...)', type: 'password' },
    ],
};

function renderCredentialFields(exchangeId) {
    const container = document.getElementById('credential-fields');
    const fields = EXCHANGE_FIELDS[exchangeId];
    if (!fields) {
        container.innerHTML = '';
        return;
    }

    container.innerHTML = fields.map(f => {
        if (f.type === 'textarea') {
            return `<label>${f.label}</label><textarea data-field="${f.name}" placeholder="${f.label}"></textarea>`;
        }
        if (f.type === 'checkbox') {
            return `<label><input type="checkbox" data-field="${f.name}" /> ${f.label}</label>`;
        }
        return `<label>${f.label}</label><input type="${f.type}" data-field="${f.name}" placeholder="${f.label}" />`;
    }).join('');
}

async function addExchangeHandler() {
    const select = document.getElementById('add-exchange-select');
    const exchangeId = select.value;
    if (!exchangeId) return showToast('Select an exchange', 'error');

    const fields = EXCHANGE_FIELDS[exchangeId] || [];
    const config = {};
    for (const f of fields) {
        const el = document.querySelector(`[data-field="${f.name}"]`);
        if (!el) continue;
        if (f.type === 'checkbox') {
            config[f.name] = el.checked;
        } else {
            const val = el.value.trim();
            if (val) config[f.name] = val;
        }
    }

    try {
        const result = await api.addExchange(exchangeId, config);
        showToast(`${result.name || exchangeId} connected`, 'success');
        await refreshExchanges();
        // Clear form
        select.value = '';
        document.getElementById('credential-fields').innerHTML = '';
    } catch (err) {
        showToast(err.message, 'error');
    }
}

async function disconnectExchange(id) {
    try {
        await api.removeExchange(id);
        showToast(`${id} disconnected`, 'success');
        await refreshExchanges();
    } catch (err) {
        showToast(err.message, 'error');
    }
}

function renderExchangeCards() {
    const container = document.getElementById('exchange-cards');
    if (!state.exchanges.length) {
        container.innerHTML = '<p class="text-muted">No exchanges connected. Add credentials to your .env file and restart.</p>';
        return;
    }

    container.innerHTML = state.exchanges.map(ex => {
        const bal = ex.balance != null ? `$${ex.balance.toFixed(2)}` : '--';
        return `
            <div class="exchange-card">
                <div class="exchange-card-info">
                    <div class="exchange-icon">${(ex.name || ex.id)[0].toUpperCase()}</div>
                    <div>
                        <div class="exchange-card-name">${ex.name || ex.id}</div>
                        <div class="exchange-card-status">Connected</div>
                    </div>
                </div>
                <div class="exchange-card-balance">${bal}</div>
            </div>
        `;
    }).join('');
}

// ---------------------------------------------------------------------------
// Portfolio: Balances
// ---------------------------------------------------------------------------

async function loadBalances() {
    const container = document.getElementById('balance-cards');
    try {
        const data = await api.fetchAllBalances();
        const balances = data.balances || {};
        if (Object.keys(balances).length === 0) {
            container.innerHTML = '<p class="text-muted">No balances available</p>';
            return;
        }
        container.innerHTML = Object.entries(balances).map(([exchange, bal]) => {
            const total = typeof bal === 'number' ? bal : Object.values(bal).reduce((s, v) => s + (typeof v === 'number' ? v : 0), 0);
            return `
                <div class="balance-card">
                    <div class="balance-card-exchange">${exchange}</div>
                    <div class="balance-card-amount">$${total.toFixed(2)}</div>
                </div>
            `;
        }).join('');
    } catch (err) {
        container.innerHTML = `<p class="text-muted">${err.message}</p>`;
    }
}

// ---------------------------------------------------------------------------
// Portfolio: PnL Chart
// ---------------------------------------------------------------------------

function renderPnLChart() {
    const container = document.getElementById('pnl-chart-container');

    if (state.pnlChart) {
        state.pnlChart.remove();
        state.pnlChart = null;
    }

    state.pnlChart = LightweightCharts.createChart(container, {
        width: container.clientWidth,
        height: 200,
        layout: { background: { type: 'solid', color: '#000000' }, textColor: '#666666', fontFamily: 'Menlo, SF Mono, Monaco, Consolas, monospace', fontSize: 10 },
        grid: { vertLines: { color: '#1a1a1a' }, horzLines: { color: '#1a1a1a' } },
        timeScale: { borderColor: '#333333' },
        rightPriceScale: { borderColor: '#333333' },
    });

    const series = state.pnlChart.addAreaSeries({
        topColor: 'rgba(0, 255, 0, 0.15)',
        bottomColor: 'rgba(0, 255, 0, 0.0)',
        lineColor: '#00ff00',
        lineWidth: 1,
    });

    // Placeholder data — in production this would come from fill history aggregation
    const now = Math.floor(Date.now() / 1000);
    const day = 86400;
    const data = [];
    let pnl = 0;
    for (let i = 30; i >= 0; i--) {
        pnl += (Math.random() - 0.48) * 2;
        data.push({ time: now - i * day, value: pnl });
    }
    series.setData(data);

    state.pnlChart.timeScale().fitContent();

    // Resize
    const observer = new ResizeObserver(() => {
        if (state.pnlChart) state.pnlChart.resize(container.clientWidth, 200);
    });
    observer.observe(container);
}

// ---------------------------------------------------------------------------
// Portfolio: Positions
// ---------------------------------------------------------------------------

async function loadPositions() {
    const tbody = document.getElementById('positions-body');
    const empty = document.getElementById('positions-empty');

    try {
        const data = await api.fetchAllPositions();
        const positions = data.positions || [];

        // Build exchange tabs
        const tabContainer = document.getElementById('position-exchange-tabs');
        const exchanges = ['all', ...new Set(positions.map(p => p.exchange))];
        tabContainer.innerHTML = exchanges.map(e =>
            `<button class="exchange-tab${e === state.positionExchange ? ' active' : ''}" data-exchange="${e}" onclick="filterPositionsByExchange('${e}')">${e === 'all' ? 'All' : e}</button>`
        ).join('');

        renderPositionRows(positions);
    } catch (err) {
        tbody.innerHTML = '';
        empty.innerHTML = `<p>${err.message}</p>`;
        empty.classList.remove('hidden');
    }
}

function renderPositionRows(positions) {
    const tbody = document.getElementById('positions-body');
    const empty = document.getElementById('positions-empty');
    const search = document.getElementById('position-search').value.toLowerCase();

    let filtered = positions;
    if (state.positionExchange !== 'all') {
        filtered = filtered.filter(p => p.exchange === state.positionExchange);
    }
    if (search) {
        filtered = filtered.filter(p => p.market_id.toLowerCase().includes(search) || p.outcome.toLowerCase().includes(search));
    }
    // Active = has size > 0
    if (state.positionFilter === 'active') {
        filtered = filtered.filter(p => p.size > 0);
    } else {
        filtered = filtered.filter(p => p.size === 0);
    }

    if (!filtered.length) {
        tbody.innerHTML = '';
        empty.classList.remove('hidden');
        return;
    }

    empty.classList.add('hidden');
    tbody.innerHTML = filtered.map(p => {
        const pnl = p.unrealized_pnl || 0;
        const pnlClass = pnl >= 0 ? 'text-green' : 'text-red';
        const pnlStr = (pnl >= 0 ? '+' : '') + '$' + pnl.toFixed(2);
        return `<tr>
            <td>${p.exchange}</td>
            <td>${truncate(p.market_id, 20)}</td>
            <td>${p.outcome}</td>
            <td>${p.size.toFixed(2)}</td>
            <td>${(p.average_price * 100).toFixed(1)}c</td>
            <td>${(p.current_price * 100).toFixed(1)}c</td>
            <td>$${(p.current_value || 0).toFixed(2)}</td>
            <td class="${pnlClass}">${pnlStr}</td>
        </tr>`;
    }).join('');
}

function filterPositionsByExchange(exchange) {
    state.positionExchange = exchange;
    document.querySelectorAll('.exchange-tab').forEach(t => t.classList.toggle('active', t.dataset.exchange === exchange));
    loadPositions();
}

// ---------------------------------------------------------------------------
// Portfolio: refresh all
// ---------------------------------------------------------------------------

async function refreshPortfolio() {
    renderExchangeCards();
    loadBalances();
    renderPnLChart();
    loadPositions();
}

async function refreshExchanges() {
    try {
        const data = await api.listExchanges();
        state.exchanges = data.exchanges || [];
    } catch {
        state.exchanges = [];
    }
    updateExchangeSelect();
    renderExchangeCards();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatNumber(n) {
    if (n == null) return '--';
    if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(1) + 'B';
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
    if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
    return typeof n === 'number' ? n.toFixed(2) : String(n);
}

function truncate(s, len) {
    if (!s) return '--';
    return s.length > len ? s.slice(0, len) + '...' : s;
}

function getCurrentMarketId() {
    if (state.eventMarkets.length) {
        return state.eventMarkets[state.selectedMarketIdx]?.id;
    }
    return document.getElementById('market-input').value.trim();
}

// ---------------------------------------------------------------------------
// Event listeners
// ---------------------------------------------------------------------------

document.addEventListener('DOMContentLoaded', () => {
    // Navigation
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', e => {
            e.preventDefault();
            navigate(link.dataset.page);
        });
    });

    // Terminal: search type toggle
    document.getElementById('type-market').addEventListener('click', () => {
        state.lookupType = 'market';
        document.getElementById('type-market').classList.add('active');
        document.getElementById('type-event').classList.remove('active');
    });
    document.getElementById('type-event').addEventListener('click', () => {
        state.lookupType = 'event';
        document.getElementById('type-event').classList.add('active');
        document.getElementById('type-market').classList.remove('active');
    });

    // Terminal: load button
    document.getElementById('load-btn').addEventListener('click', loadMarketOrEvent);
    document.getElementById('market-input').addEventListener('keydown', e => {
        if (e.key === 'Enter') loadMarketOrEvent();
    });

    // Time range buttons
    document.querySelectorAll('.time-range-buttons .range-btn').forEach(btn => {
        btn.addEventListener('click', () => changeTimeRange(btn.dataset.range));
    });

    // Bottom panel tabs
    document.querySelectorAll('.bottom-tabs .tab-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.bottom-tabs .tab-btn').forEach(b => b.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(`tab-${btn.dataset.tab}`).classList.add('active');
            // Load data for the tab
            if (btn.dataset.tab === 'trades') loadTrades();
            if (btn.dataset.tab === 'open-orders') loadOpenOrders();
        });
    });

    // Order panel: side tabs
    document.getElementById('side-buy').addEventListener('click', () => {
        state.orderSide = 'buy';
        document.getElementById('side-buy').classList.add('active');
        document.getElementById('side-sell').classList.remove('active');
        document.getElementById('submit-order').classList.remove('sell-mode');
        updateOrderSummary();
    });
    document.getElementById('side-sell').addEventListener('click', () => {
        state.orderSide = 'sell';
        document.getElementById('side-sell').classList.add('active');
        document.getElementById('side-buy').classList.remove('active');
        document.getElementById('submit-order').classList.add('sell-mode');
        updateOrderSummary();
    });

    // Order panel: outcome buttons
    document.getElementById('outcome-yes').addEventListener('click', () => {
        state.orderOutcome = 'Yes';
        document.getElementById('outcome-yes').classList.add('active');
        document.getElementById('outcome-no').classList.remove('active');
        updateOrderSummary();
    });
    document.getElementById('outcome-no').addEventListener('click', () => {
        state.orderOutcome = 'No';
        document.getElementById('outcome-no').classList.add('active');
        document.getElementById('outcome-yes').classList.remove('active');
        updateOrderSummary();
    });

    // Order panel: order type
    document.getElementById('order-type-select').addEventListener('change', e => {
        state.orderType = e.target.value;
        document.getElementById('limit-price-section').style.display = e.target.value === 'limit' ? '' : 'none';
        updateOrderSummary();
    });

    // Order panel: amount input
    document.getElementById('amount-input').addEventListener('input', updateOrderSummary);
    document.getElementById('limit-price-input').addEventListener('input', updateOrderSummary);

    // Quick amount buttons
    document.querySelectorAll('.quick-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const input = document.getElementById('amount-input');
            if (btn.dataset.amount === 'max') {
                if (state.balance) {
                    const total = Object.values(state.balance).reduce((s, v) => s + (typeof v === 'number' ? v : 0), 0);
                    input.value = total.toFixed(2);
                }
            } else {
                const current = parseFloat(input.value) || 0;
                input.value = (current + parseFloat(btn.dataset.amount)).toFixed(2);
            }
            updateOrderSummary();
        });
    });

    // Submit order
    document.getElementById('submit-order').addEventListener('click', submitOrder);

    // Portfolio: add exchange form (removed — credentials loaded from .env)

    // Portfolio: position search
    document.getElementById('position-search').addEventListener('input', () => {
        loadPositions();
    });

    // Portfolio: position filter tabs
    document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            state.positionFilter = btn.dataset.filter;
            document.querySelectorAll('.filter-btn').forEach(b => b.classList.toggle('active', b.dataset.filter === state.positionFilter));
            loadPositions();
        });
    });

    // Initial load
    refreshExchanges();
});
