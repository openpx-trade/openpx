# Overview

## Opinion Websocket

Welcome to the official documentation for the Opinion WebSocket API — a real-time, event-driven interface for accessing live market data in OPINION Prediction Markets.

> 📊 **Public Websocket Data API**: This Websocket API provides live, streaming read-only access to market data, orderbooks, and price information. For trading operations (placing orders, managing positions), please use the [Opinion CLOB SDK](https://github.com/opinion-labs/opinion-clob-sdk).&#x20;
>
> To request API access, Please kindly fill out this [short application form ](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII).&#x20;
>
> *API Key can be used for Opinion OpenAPI, Opinion Websocket, and Opinion CLOB SDK*

### What is Opinion Websocket?

The Opinion WebSocket API provides a persistent connection that pushes updates in real time from Opinion prediction market. Unlike RESTful polling, WebSockets ensure developers receive data instantly when changes occur, reducing latency and network overhead - ideal for live dashboards, trading engines, and analytics:

* **Subscribe to market price streams** — Receive ticks and trade updates as they happen
* **Monitor orderbook changes** — Get bid/ask book deltas in near real-time
* **Receive market events** — Market activation, resolution, or status transitions
* **Track aggregated metrics** — Volume or event summaries streamed live

### Key Features

**Real-Time Streaming**

* Persistent WebSocket connection for low-latency updates
* Event-driven feeds eliminate the need for repeated polling
* Efficient delivery of high-frequency data

**Subscription Model**

* Subscribe to specific markets, tokens, or event types
* Receive targeted updates to minimize noise and bandwidth

**Secure & Compatible**

* API Key authentication on initial handshake
* TLS/SSL encrypted transport
* Works across languages and platforms with standard WebSocket clients

### Use Cases

**Live Market Dashboards**

Build interactive UIs that update instantly with trades, prices, and orderbook moves.

**Automated Trading Clients**

Feed real-time market data into trading algorithms, bots, or signal engines.

**Analytics & Alerting**

Trigger custom alerts when price thresholds, spread changes, or market events occur.

### How It Works

1. **Connect to the WebSocket endpoint** with your API Key.
2. **Authenticate and subscribe** to one or more topics (markets, tokens, events).
3. **Receive streaming messages** as soon as updates occur.
4. **Parse and act** on events in your application.
