# Overview

## Opinion OpenAPI

Welcome to the official documentation for the Opinion OpenAPI - a RESTful API for accessing OPINION Prediction Markets

> 📊 **Public Data API**: This API provides read-only access to market data, orderbooks, and price information. For trading operations (placing orders, managing positions), please use the [Opinion CLOB SDK](https://pypi.org/project/opinion-clob-sdk/).&#x20;
>
> To request API access, Please kindly fill out this [short application form ](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII).&#x20;
>
> *API Key can be used for Opinion OpenAPI, Opinion Websocket, and Opinion CLOB SDK*

### What is Opinion OpenAPI?

The Opinion OpenAPI provides a simple HTTP interface for accessing prediction market data from Opinion Labs' infrastructure. It enables developers to:

* **Query market data** - Access real-time market information, metadata, and trading volumes
* **Monitor prices** - Get latest trade prices and historical price data
* **Analyze orderbooks** - Retrieve order book depth for any market token
* **Discover quote tokens** - List available trading currencies and their configurations

### Key Features

#### Simple Integration

* **RESTful** - Standard HTTP/JSON API
* **OpenAPI 3.0** - Full specification with Swagger/Redoc support
* **Language Agnostic** - Use with any programming language
* **No Dependencies** - Just HTTP requests

#### &#x20;Performance Optimized

* **Low Latency** - Optimized for real-time data access
* **Rate Limited** - 15 requests/second per API key
* **Paginated** - Efficient handling of large datasets

#### &#x20;Secure Access

* **API Key Authentication** - Simple header-based auth
* **HTTPS Only** - All traffic encrypted
* **Production Ready** - Battle-tested infrastructure

#### &#x20;Blockchain Support

| Chain             | Chain ID | Status |
| ----------------- | -------- | ------ |
| BNB Chain Mainnet | 56       | ✅ Live |

### Use Cases

#### Market Analytics Dashboard

Aggregate and display market data for research or monitoring applications.

```bash
# Get all active markets sorted by 24h volume
curl -X GET "https://proxy.opinion.trade:8443/openapi/market?status=activated&sortBy=5&limit=20" \
  -H "apikey: your_api_key"
```

```json
{
  "code": 0,
  "msg": "success",
  "result": {
    "total": 150,
    "list": [
      {
        "marketId": 123,
        "marketTitle": "Will BTC reach $100k by end of 2025?",
        "status": 2,
        "statusEnum": "Activated",
        "yesTokenId": "0x1234...5678",
        "noTokenId": "0x8765...4321",
        "volume": "1500000.00",
        "volume24h": "125000.00"
      }
    ]
  }
}
```

#### Price Monitoring Bot

Track real-time prices for specific outcome tokens.

```bash
# Get latest price for a token
curl -X GET "https://proxy.opinion.trade:8443/openapi/token/latest-price?token_id=0x1234...5678" \
  -H "apikey: your_api_key"
```

```json
{
  "code": 0,
  "msg": "success", 
  "result": {
    "tokenId": "0x1234...5678",
    "price": "0.65",
    "side": "BUY",
    "size": "1000.00",
    "timestamp": 1733312400000
  }
}
```

#### Orderbook Analysis

Analyze market depth for trading insights.

```bash
# Get orderbook for a token
curl -X GET "https://proxy.opinion.trade:8443/openapi/token/orderbook?token_id=0x1234...5678" \
  -H "apikey: your_api_key"
```

```json
{
  "code": 0,
  "msg": "success",
  "result": {
    "market": "0xabc...def",
    "tokenId": "0x1234...5678",
    "timestamp": 1733312400000,
    "bids": [
      {"price": "0.64", "size": "5000.00"},
      {"price": "0.63", "size": "12000.00"}
    ],
    "asks": [
      {"price": "0.66", "size": "3000.00"},
      {"price": "0.67", "size": "8000.00"}
    ]
  }
}
```

#### Historical Price Charts

Build price charts with historical data.

```bash
# Get daily price history for the last 30 days
curl -X GET "https://proxy.opinion.trade:8443/openapi/token/price-history?token_id=0x1234...5678&interval=1d" \
  -H "apikey: your_api_key"
```

```json
{
  "code": 0,
  "msg": "success",
  "result": {
    "history": [
      {"t": 1733184000, "p": "0.58"},
      {"t": 1733270400, "p": "0.62"},
      {"t": 1733356800, "p": "0.65"}
    ]
  }
}
```

### API Endpoints Overview

| Endpoint               | Method | Description                    |
| ---------------------- | ------ | ------------------------------ |
| `/market`              | GET    | List all markets with filters  |
| `/market/{marketId}`   | GET    | Get market details by ID       |
| `/token/latest-price`  | GET    | Get latest trade price         |
| `/token/orderbook`     | GET    | Get order book depth           |
| `/token/price-history` | GET    | Get historical prices          |
| `/quoteToken`          | GET    | List quote tokens (currencies) |

### Authentication

All API requests require an API key passed in the `apikey` header:

```bash
curl -X GET "https://proxy.opinion.trade:8443/openapi/market" \
  -H "apikey: your_api_key" \
  -H "Content-Type: application/json"
```

> 📧 **Get an API Key**: Please kindly fill out this [short application form ](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII).

### Rate Limiting

| Limit               | Value |
| ------------------- | ----- |
| Requests per second | 15    |
| Max items per page  | 20    |

If you exceed rate limits, you'll receive a `429 Too Many Requests` response.

### Response Format

All responses follow a consistent JSON structure:

```json
{
  "code": 0,         // 0 = success, non-zero = error
  "msg": "success",  // Human-readable message
  "result": { ... }  // Response data (varies by endpoint)
}
```

#### Error Codes

| Code | Description                               |
| ---- | ----------------------------------------- |
| 0    | Success                                   |
| 400  | Bad Request - Invalid parameters          |
| 401  | Unauthorized - Invalid or missing API key |
| 404  | Not Found - Resource doesn't exist        |
| 429  | Too Many Requests - Rate limit exceeded   |
| 500  | Internal Server Error                     |

### Quick Links

| Resource   | Link                                                                 |
| ---------- | -------------------------------------------------------------------- |
| Python SDK | [Opinion CLOB SDK](https://github.com/opinion-labs/opinion-clob-sdk) |

### SDK vs OpenAPI

| Feature             | OpenAPI (This API) | CLOB SDK |
| ------------------- | ------------------ | -------- |
| Market Data         | ✅                  | ✅        |
| Orderbook           | ✅                  | ✅        |
| Price History       | ✅                  | ✅        |
| Place Orders        | ❌                  | ✅        |
| Cancel Orders       | ❌                  | ✅        |
| Manage Positions    | ❌                  | ✅        |
| On-chain Operations | ❌                  | ✅        |
| Language            | Any (HTTP)         | Python   |

**Recommendation**:

* Use **OPINION** **OpenAPI** for read-only data access, dashboards, and analytics
* Use **OPINION** **CLOB SDK** for trading, order management, and blockchain interactions

***

Ready to get started? Check the OpenAPI Specification for detailed endpoint documentation.

Authentication
Access to all endpoints requires providing your API key in the apikey HTTP request header.

Rate Limiting
The API imposes a rate limit of 15 requests per second. If you require a higher rate limit or have questions regarding rate limit policies, please contact nik@opinionlabs.xyz for professional assistance.

Pagination
Most list endpoints support pagination with page and limit parameters. Maximum limit per request is 20 items.