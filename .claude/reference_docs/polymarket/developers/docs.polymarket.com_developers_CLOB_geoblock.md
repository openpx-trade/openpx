---
url: "https://docs.polymarket.com/developers/CLOB/geoblock"
title: "Geographic Restrictions - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/geoblock#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Central Limit Order Book

Geographic Restrictions

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/CLOB/geoblock#overview)
- [Server Infrastructure](https://docs.polymarket.com/developers/CLOB/geoblock#server-infrastructure)
- [Geoblock Endpoint](https://docs.polymarket.com/developers/CLOB/geoblock#geoblock-endpoint)
- [Response](https://docs.polymarket.com/developers/CLOB/geoblock#response)
- [Blocked Countries](https://docs.polymarket.com/developers/CLOB/geoblock#blocked-countries)
- [Blocked Regions](https://docs.polymarket.com/developers/CLOB/geoblock#blocked-regions)
- [Usage Examples](https://docs.polymarket.com/developers/CLOB/geoblock#usage-examples)

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#overview)  Overview

Polymarket restricts order placement from certain geographic locations due to regulatory requirements and compliance with international sanctions.
Before placing orders, builders should verify the location.

Orders submitted from blocked regions will be rejected. Implement geoblock checks
in your application to provide users with appropriate feedback before they attempt to trade.

* * *

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#server-infrastructure)  Server Infrastructure

- **Primary Servers**: eu-west-2
- **Closest Non-Georestricted Region**: eu-west-1

* * *

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#geoblock-endpoint)  Geoblock Endpoint

Check the geographic eligibility of the requesting IP address:

Copy

Ask AI

```
GET https://polymarket.com/api/geoblock
```

### [​](https://docs.polymarket.com/developers/CLOB/geoblock\#response)  Response

Copy

Ask AI

```
{
  "blocked": boolean;
  "ip": string;
  "country": string;
  "region": string;
}
```

| Field | Type | Description |
| --- | --- | --- |
| `blocked` | boolean | Whether the user is blocked from placing orders |
| `ip` | string | Detected IP address |
| `country` | string | ISO 3166-1 alpha-2 country code |
| `region` | string | Region/state code |

* * *

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#blocked-countries)  Blocked Countries

The following **33 countries** are completely restricted from placing orders on Polymarket:

| Country Code | Country Name |
| --- | --- |
| AU | Australia |
| BE | Belgium |
| BY | Belarus |
| BI | Burundi |
| CF | Central African Republic |
| CD | Congo (Kinshasa) |
| CU | Cuba |
| DE | Germany |
| ET | Ethiopia |
| FR | France |
| GB | United Kingdom |
| IR | Iran |
| IQ | Iraq |
| IT | Italy |
| KP | North Korea |
| LB | Lebanon |
| LY | Libya |
| MM | Myanmar |
| NI | Nicaragua |
| PL | Poland |
| RU | Russia |
| SG | Singapore |
| SO | Somalia |
| SS | South Sudan |
| SD | Sudan |
| SY | Syria |
| TH | Thailand |
| TW | Taiwan |
| UM | United States Minor Outlying Islands |
| US | United States |
| VE | Venezuela |
| YE | Yemen |
| ZW | Zimbabwe |

* * *

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#blocked-regions)  Blocked Regions

In addition to fully blocked countries, the following specific regions within otherwise accessible countries are also restricted:

| Country | Region | Region Code |
| --- | --- | --- |
| Canada (CA) | Ontario | ON |
| Ukraine (UA) | Crimea | 43 |
| Ukraine (UA) | Donetsk | 14 |
| Ukraine (UA) | Luhansk | 09 |

* * *

## [​](https://docs.polymarket.com/developers/CLOB/geoblock\#usage-examples)  Usage Examples

- TypeScript

- Python


Copy

Ask AI

```
interface GeoblockResponse {
  blocked: boolean;
  ip: string;
  country: string;
  region: string;
}

async function checkGeoblock(): Promise<GeoblockResponse> {
  const response = await fetch("https://polymarket.com/api/geoblock");
  return response.json();
}

// Usage
const geo = await checkGeoblock();

if (geo.blocked) {
  console.log(`Trading not available in ${geo.country}`);
} else {
  console.log("Trading available");
}
```

Copy

Ask AI

```
import requests

def check_geoblock() -> dict:
    response = requests.get("https://polymarket.com/api/geoblock")
    return response.json()

# Usage
geo = check_geoblock()

if geo["blocked"]:
    print(f"Trading not available in {geo['country']}")
else:
    print("Trading available")
```

[Authentication](https://docs.polymarket.com/developers/CLOB/authentication) [Methods Overview](https://docs.polymarket.com/developers/CLOB/clients/methods-overview)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.