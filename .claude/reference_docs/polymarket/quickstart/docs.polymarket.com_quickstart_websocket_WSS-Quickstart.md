---
url: "https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart"
title: "WSS Quickstart - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

- [Main Site](https://polymarket.com/)
- [Main Site](https://polymarket.com/)

Search...

Navigation

Websocket

WSS Quickstart

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

- [Polymarket](https://polymarket.com/)
- [Discord Community](https://discord.gg/polymarket)
- [Twitter](https://x.com/polymarket)

##### Developer Quickstart

- [Developer Quickstart](https://docs.polymarket.com/quickstart/overview)
- [Fetching Market Data](https://docs.polymarket.com/quickstart/fetching-data)
- [Placing Your First Order](https://docs.polymarket.com/quickstart/first-order)
- [Glossary](https://docs.polymarket.com/quickstart/reference/glossary)
- [API Rate Limits](https://docs.polymarket.com/quickstart/introduction/rate-limits)
- [Endpoints](https://docs.polymarket.com/quickstart/reference/endpoints)

##### Market Makers

- [Market Maker Introduction](https://docs.polymarket.com/developers/market-makers/introduction)
- [Setup](https://docs.polymarket.com/developers/market-makers/setup)
- [Trading](https://docs.polymarket.com/developers/market-makers/trading)
- [Liquidity Rewards](https://docs.polymarket.com/developers/market-makers/liquidity-rewards)
- [Maker Rebates Program](https://docs.polymarket.com/developers/market-makers/maker-rebates-program)
- [Data Feeds](https://docs.polymarket.com/developers/market-makers/data-feeds)
- [Inventory Management](https://docs.polymarket.com/developers/market-makers/inventory)

##### Polymarket Builders Program

- [Builder Program Introduction](https://docs.polymarket.com/developers/builders/builder-intro)
- [Builder Tiers](https://docs.polymarket.com/developers/builders/builder-tiers)
- [Builder Profile & Keys](https://docs.polymarket.com/developers/builders/builder-profile)
- [Order Attribution](https://docs.polymarket.com/developers/builders/order-attribution)
- [Relayer Client](https://docs.polymarket.com/developers/builders/relayer-client)
- [Examples](https://docs.polymarket.com/developers/builders/examples)

##### Central Limit Order Book

- [CLOB Introduction](https://docs.polymarket.com/developers/CLOB/introduction)
- [Status](https://docs.polymarket.com/developers/CLOB/status)
- [Quickstart](https://docs.polymarket.com/developers/CLOB/quickstart)
- [Authentication](https://docs.polymarket.com/developers/CLOB/authentication)
- [Geographic Restrictions](https://docs.polymarket.com/developers/CLOB/geoblock)
- Client

- REST API

- Historical Timeseries Data

- Order Management

- Trades


##### Websocket

- [WSS Overview](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)
- [WSS Quickstart](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart)
- [WSS Authentication](https://docs.polymarket.com/developers/CLOB/websocket/wss-auth)
- [User Channel](https://docs.polymarket.com/developers/CLOB/websocket/user-channel)
- [Market Channel](https://docs.polymarket.com/developers/CLOB/websocket/market-channel)
- Sports Websocket


##### Real Time Data Stream

- [RTDS Overview](https://docs.polymarket.com/developers/RTDS/RTDS-overview)
- [RTDS Crypto Prices](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices)
- [RTDS Comments](https://docs.polymarket.com/developers/RTDS/RTDS-comments)

##### Gamma Structure

- [Overview](https://docs.polymarket.com/developers/gamma-markets-api/overview)
- [Gamma Structure](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure)
- [Fetching Markets](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide)

##### Gamma Endpoints

- Gamma Status

- Sports

- Tags

- Events

- Markets

- Series

- Comments

- Profiles

- Search


##### Data-API

- Data API Status

- Misc

- Core

- Builders


##### Bridge & Swap

- [Overview](https://docs.polymarket.com/developers/misc-endpoints/bridge-overview)
- Bridge


##### Subgraph

- [Overview](https://docs.polymarket.com/developers/subgraph/overview)

##### Resolution

- [Resolution](https://docs.polymarket.com/developers/resolution/UMA)

##### Conditional Token Frameworks

- [Overview](https://docs.polymarket.com/developers/CTF/overview)
- [Splitting USDC](https://docs.polymarket.com/developers/CTF/split)
- [Merging Tokens](https://docs.polymarket.com/developers/CTF/merge)
- [Reedeeming Tokens](https://docs.polymarket.com/developers/CTF/redeem)
- [Deployment and Additional Information](https://docs.polymarket.com/developers/CTF/deployment-resources)

##### Proxy Wallets

- [Proxy wallet](https://docs.polymarket.com/developers/proxy-wallet)

##### Negative Risk

- [Overview](https://docs.polymarket.com/developers/neg-risk/overview)

On this page

- [Getting your API Keys](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart#getting-your-api-keys)
- [Using those keys to connect to the Market or User Websocket](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart#using-those-keys-to-connect-to-the-market-or-user-websocket)

Websocket

# WSS Quickstart

The following code samples and explanation will show you how to subscribe to the Marker and User channels of the Websocket.
You’ll need your API keys to do this so we’ll start with that.

## [​](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart\#getting-your-api-keys)  Getting your API Keys

DeriveAPIKeys-Python

DeriveAPIKeys-TS

Copy

Ask AI

```
from py_clob_client.client import ClobClient

host: str = "https://clob.polymarket.com"
key: str = "" #This is your Private Key. If using email login export from https://reveal.magic.link/polymarket otherwise export from your Web3 Application
chain_id: int = 137 #No need to adjust this
POLYMARKET_PROXY_ADDRESS: str = '' #This is the address you deposit/send USDC to to FUND your Polymarket account.

#Select from the following 3 initialization options to matches your login method, and remove any unused lines so only one client is initialized.

### Initialization of a client using a Polymarket Proxy associated with an Email/Magic account. If you login with your email use this example.
client = ClobClient(host, key=key, chain_id=chain_id, signature_type=1, funder=POLYMARKET_PROXY_ADDRESS)

### Initialization of a client using a Polymarket Proxy associated with a Browser Wallet(Metamask, Coinbase Wallet, etc)
client = ClobClient(host, key=key, chain_id=chain_id, signature_type=2, funder=POLYMARKET_PROXY_ADDRESS)

### Initialization of a client that trades directly from an EOA.
client = ClobClient(host, key=key, chain_id=chain_id)

print( client.derive_api_key() )
```

See all 20 lines

## [​](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart\#using-those-keys-to-connect-to-the-market-or-user-websocket)  Using those keys to connect to the Market or User Websocket

WSS-Connection

Copy

Ask AI

```
from websocket import WebSocketApp
import json
import time
import threading

MARKET_CHANNEL = "market"
USER_CHANNEL = "user"

class WebSocketOrderBook:
    def __init__(self, channel_type, url, data, auth, message_callback, verbose):
        self.channel_type = channel_type
        self.url = url
        self.data = data
        self.auth = auth
        self.message_callback = message_callback
        self.verbose = verbose
        furl = url + "/ws/" + channel_type
        self.ws = WebSocketApp(
            furl,
            on_message=self.on_message,
            on_error=self.on_error,
            on_close=self.on_close,
            on_open=self.on_open,
        )
        self.orderbooks = {}

    def on_message(self, ws, message):
        print(message)
        pass

    def on_error(self, ws, error):
        print("Error: ", error)
        exit(1)

    def on_close(self, ws, close_status_code, close_msg):
        print("closing")
        exit(0)

    def on_open(self, ws):
        if self.channel_type == MARKET_CHANNEL:
            ws.send(json.dumps({"assets_ids": self.data, "type": MARKET_CHANNEL}))
        elif self.channel_type == USER_CHANNEL and self.auth:
            ws.send(
                json.dumps(
                    {"markets": self.data, "type": USER_CHANNEL, "auth": self.auth}
                )
            )
        else:
            exit(1)

        thr = threading.Thread(target=self.ping, args=(ws,))
        thr.start()

    def subscribe_to_tokens_ids(self, assets_ids):
        if self.channel_type == MARKET_CHANNEL:
            self.ws.send(json.dumps({"assets_ids": assets_ids, "operation": "subscribe"}))

    def unsubscribe_to_tokens_ids(self, assets_ids):
        if self.channel_type == MARKET_CHANNEL:
            self.ws.send(json.dumps({"assets_ids": assets_ids, "operation": "unsubscribe"}))

    def ping(self, ws):
        while True:
            ws.send("PING")
            time.sleep(10)

    def run(self):
        self.ws.run_forever()

if __name__ == "__main__":
    url = "wss://ws-subscriptions-clob.polymarket.com"
    #Complete these by exporting them from your initialized client.
    api_key = ""
    api_secret = ""
    api_passphrase = ""

    asset_ids = [\
        "109681959945973300464568698402968596289258214226684818748321941747028805721376",\
    ]
    condition_ids = [] # no really need to filter by this one

    auth = {"apiKey": api_key, "secret": api_secret, "passphrase": api_passphrase}

    market_connection = WebSocketOrderBook(
        MARKET_CHANNEL, url, asset_ids, auth, None, True
    )
    user_connection = WebSocketOrderBook(
        USER_CHANNEL, url, condition_ids, auth, None, True
    )

    market_connection.subscribe_to_tokens_ids(["123"])
    # market_connection.unsubscribe_to_tokens_ids(["123"])

    market_connection.run()
    # user_connection.run()
```

See all 99 lines

[WSS Overview](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview) [WSS Authentication](https://docs.polymarket.com/developers/CLOB/websocket/wss-auth)

Ctrl+I

[github](https://github.com/polymarket)

[Powered by](https://www.mintlify.com/?utm_campaign=poweredBy&utm_medium=referral&utm_source=polymarket-292d1b1b)

Assistant

Responses are generated using AI and may contain mistakes.