---
url: "https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide"
title: "How to Fetch Markets - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Gamma Structure

How to Fetch Markets

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#overview)
- [1\. Fetch by Slug](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#1-fetch-by-slug)
- [How to Extract the Slug](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#how-to-extract-the-slug)
- [API Endpoints](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#api-endpoints)
- [Examples](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#examples)
- [2\. Fetch by Tags](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#2-fetch-by-tags)
- [Discover Available Tags](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#discover-available-tags)
- [Using Tags in Market Requests](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#using-tags-in-market-requests)
- [Additional Tag Filtering](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#additional-tag-filtering)
- [3\. Fetch All Active Markets](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#3-fetch-all-active-markets)
- [Key Parameters](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#key-parameters)
- [Examples](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#examples-2)
- [Pagination](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#pagination)
- [Best Practices](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#best-practices)
- [Related Endpoints](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#related-endpoints)

Both the getEvents and getMarkets are paginated. See [pagination section](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide#pagination) for details.

This guide covers the three recommended approaches for fetching market data from the Gamma API, each optimized for different use cases.

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#overview)  Overview

There are three main strategies for retrieving market data:

1. **By Slug** \- Best for fetching specific individual markets or events
2. **By Tags** \- Ideal for filtering markets by category or sport
3. **Via Events Endpoint** \- Most efficient for retrieving all active markets

* * *

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#1-fetch-by-slug)  1\. Fetch by Slug

**Use Case:** When you need to retrieve a specific market or event that you already know about.Individual markets and events are best fetched using their unique slug identifier. The slug can be found directly in the Polymarket frontend URL.

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#how-to-extract-the-slug)  How to Extract the Slug

From any Polymarket URL, the slug is the path segment after `/event/` or `/market/`:

Copy

Ask AI

```
https://polymarket.com/event/fed-decision-in-october?tid=1758818660485
                            ↑
                  Slug: fed-decision-in-october
```

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#api-endpoints)  API Endpoints

**For Events:** [GET /events/slug/](https://docs.polymarket.com/api-reference/events/list-events)**For Markets:** [GET /markets/slug/](https://docs.polymarket.com/api-reference/markets/list-markets)

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#examples)  Examples

Copy

Ask AI

```
curl "https://gamma-api.polymarket.com/events/slug/fed-decision-in-october"
```

* * *

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#2-fetch-by-tags)  2\. Fetch by Tags

**Use Case:** When you want to filter markets by category, sport, or topic.Tags provide a powerful way to categorize and filter markets. You can discover available tags and then use them to filter your market requests.

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#discover-available-tags)  Discover Available Tags

**General Tags:** [GET /tags](https://docs.polymarket.com/api-reference/tags/list-tags)**Sports Tags & Metadata:** [GET /sports](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information)The `/sports` endpoint returns comprehensive metadata for sports including tag IDs, images, resolution sources, and series information.

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#using-tags-in-market-requests)  Using Tags in Market Requests

Once you have tag IDs, you can use them with the `tag_id` parameter in both markets and events endpoints.**Markets with Tags:** [GET /markets](https://docs.polymarket.com/api-reference/markets/list-markets)**Events with Tags:** [GET /events](https://docs.polymarket.com/api-reference/events/list-events)

Copy

Ask AI

```
curl "https://gamma-api.polymarket.com/events?tag_id=100381&limit=1&closed=false"
```

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#additional-tag-filtering)  Additional Tag Filtering

You can also:

- Use `related_tags=true` to include related tag markets
- Exclude specific tags with `exclude_tag_id`

* * *

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#3-fetch-all-active-markets)  3\. Fetch All Active Markets

**Use Case:** When you need to retrieve all available active markets, typically for broader analysis or market discovery.The most efficient approach is to use the `/events` endpoint and work backwards, as events contain their associated markets.**Events Endpoint:** [GET /events](https://docs.polymarket.com/api-reference/events/list-events)**Markets Endpoint:** [GET /markets](https://docs.polymarket.com/api-reference/markets/list-markets)

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#key-parameters)  Key Parameters

- `order=id` \- Order by event ID
- `ascending=false` \- Get newest events first
- `closed=false` \- Only active markets
- `limit` \- Control response size
- `offset` \- For pagination

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#examples-2)  Examples

Copy

Ask AI

```
curl "https://gamma-api.polymarket.com/events?order=id&ascending=false&closed=false&limit=100"
```

This approach gives you all active markets ordered from newest to oldest, allowing you to systematically process all available trading opportunities.

### [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#pagination)  Pagination

For large datasets, use pagination with `limit` and `offset` parameters:

- `limit=50` \- Return 50 results per page
- `offset=0` \- Start from the beginning (increment by limit for subsequent pages)

**Pagination Examples:**

Copy

Ask AI

```
# Page 1: First 50 results (offset=0)
curl "https://gamma-api.polymarket.com/events?order=id&ascending=false&closed=false&limit=50&offset=0"
```

Copy

Ask AI

```
# Page 2: Next 50 results (offset=50)
curl "https://gamma-api.polymarket.com/events?order=id&ascending=false&closed=false&limit=50&offset=50"
```

Copy

Ask AI

```
# Page 3: Next 50 results (offset=100)
curl "https://gamma-api.polymarket.com/events?order=id&ascending=false&closed=false&limit=50&offset=100"
```

Copy

Ask AI

```
# Paginating through markets with tag filtering
curl "https://gamma-api.polymarket.com/markets?tag_id=100381&closed=false&limit=25&offset=0"
```

Copy

Ask AI

```
# Next page of markets with tag filtering
curl "https://gamma-api.polymarket.com/markets?tag_id=100381&closed=false&limit=25&offset=25"
```

* * *

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#best-practices)  Best Practices

1. **For Individual Markets:** Always use the slug method for best performance
2. **For Category Browsing:** Use tag filtering to reduce API calls
3. **For Complete Market Discovery:** Use the events endpoint with pagination
4. **Always Include `closed=false`:** Unless you specifically need historical data
5. **Implement Rate Limiting:** Respect API limits for production applications

## [​](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide\#related-endpoints)  Related Endpoints

- [Get Markets](https://docs.polymarket.com/developers/gamma-markets-api/get-markets) \- Full markets endpoint documentation
- [Get Events](https://docs.polymarket.com/developers/gamma-markets-api/get-events) \- Full events endpoint documentation
- [Search Markets](https://docs.polymarket.com/developers/gamma-markets-api/get-public-search) \- Search functionality

[Gamma Structure](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure) [Gamma API Health check](https://docs.polymarket.com/api-reference/gamma-status/gamma-api-health-check)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.