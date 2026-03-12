---
url: "https://docs.polymarket.com/developers/gamma-markets-api/get-events"
title: "Get Events - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/gamma-markets-api/get-events#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Get Events

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

List events

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/events
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "ticker": "<string>",\
    "slug": "<string>",\
    "title": "<string>",\
    "subtitle": "<string>",\
    "description": "<string>",\
    "resolutionSource": "<string>",\
    "startDate": "2023-11-07T05:31:56Z",\
    "creationDate": "2023-11-07T05:31:56Z",\
    "endDate": "2023-11-07T05:31:56Z",\
    "image": "<string>",\
    "icon": "<string>",\
    "active": true,\
    "closed": true,\
    "archived": true,\
    "new": true,\
    "featured": true,\
    "restricted": true,\
    "liquidity": 123,\
    "volume": 123,\
    "openInterest": 123,\
    "sortBy": "<string>",\
    "category": "<string>",\
    "subcategory": "<string>",\
    "isTemplate": true,\
    "templateVariables": "<string>",\
    "published_at": "<string>",\
    "createdBy": "<string>",\
    "updatedBy": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "commentsEnabled": true,\
    "competitive": 123,\
    "volume24hr": 123,\
    "volume1wk": 123,\
    "volume1mo": 123,\
    "volume1yr": 123,\
    "featuredImage": "<string>",\
    "disqusThread": "<string>",\
    "parentEvent": "<string>",\
    "enableOrderBook": true,\
    "liquidityAmm": 123,\
    "liquidityClob": 123,\
    "negRisk": true,\
    "negRiskMarketID": "<string>",\
    "negRiskFeeBips": 123,\
    "commentCount": 123,\
    "imageOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "iconOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "featuredImageOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "subEvents": [\
      "<string>"\
    ],\
    "markets": [\
      {\
        "id": "<string>",\
        "question": "<string>",\
        "conditionId": "<string>",\
        "slug": "<string>",\
        "twitterCardImage": "<string>",\
        "resolutionSource": "<string>",\
        "endDate": "2023-11-07T05:31:56Z",\
        "category": "<string>",\
        "ammType": "<string>",\
        "liquidity": "<string>",\
        "sponsorName": "<string>",\
        "sponsorImage": "<string>",\
        "startDate": "2023-11-07T05:31:56Z",\
        "xAxisValue": "<string>",\
        "yAxisValue": "<string>",\
        "denominationToken": "<string>",\
        "fee": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "lowerBound": "<string>",\
        "upperBound": "<string>",\
        "description": "<string>",\
        "outcomes": "<string>",\
        "outcomePrices": "<string>",\
        "volume": "<string>",\
        "active": true,\
        "marketType": "<string>",\
        "formatType": "<string>",\
        "lowerBoundDate": "<string>",\
        "upperBoundDate": "<string>",\
        "closed": true,\
        "marketMakerAddress": "<string>",\
        "createdBy": 123,\
        "updatedBy": 123,\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "closedTime": "<string>",\
        "wideFormat": true,\
        "new": true,\
        "mailchimpTag": "<string>",\
        "featured": true,\
        "archived": true,\
        "resolvedBy": "<string>",\
        "restricted": true,\
        "marketGroup": 123,\
        "groupItemTitle": "<string>",\
        "groupItemThreshold": "<string>",\
        "questionID": "<string>",\
        "umaEndDate": "<string>",\
        "enableOrderBook": true,\
        "orderPriceMinTickSize": 123,\
        "orderMinSize": 123,\
        "umaResolutionStatus": "<string>",\
        "curationOrder": 123,\
        "volumeNum": 123,\
        "liquidityNum": 123,\
        "endDateIso": "<string>",\
        "startDateIso": "<string>",\
        "umaEndDateIso": "<string>",\
        "hasReviewedDates": true,\
        "readyForCron": true,\
        "commentsEnabled": true,\
        "volume24hr": 123,\
        "volume1wk": 123,\
        "volume1mo": 123,\
        "volume1yr": 123,\
        "gameStartTime": "<string>",\
        "secondsDelay": 123,\
        "clobTokenIds": "<string>",\
        "disqusThread": "<string>",\
        "shortOutcomes": "<string>",\
        "teamAID": "<string>",\
        "teamBID": "<string>",\
        "umaBond": "<string>",\
        "umaReward": "<string>",\
        "fpmmLive": true,\
        "volume24hrAmm": 123,\
        "volume1wkAmm": 123,\
        "volume1moAmm": 123,\
        "volume1yrAmm": 123,\
        "volume24hrClob": 123,\
        "volume1wkClob": 123,\
        "volume1moClob": 123,\
        "volume1yrClob": 123,\
        "volumeAmm": 123,\
        "volumeClob": 123,\
        "liquidityAmm": 123,\
        "liquidityClob": 123,\
        "makerBaseFee": 123,\
        "takerBaseFee": 123,\
        "customLiveness": 123,\
        "acceptingOrders": true,\
        "notificationsEnabled": true,\
        "score": 123,\
        "imageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "iconOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "events": "<array>",\
        "categories": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "parentCategory": "<string>",\
            "slug": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z"\
          }\
        ],\
        "tags": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "slug": "<string>",\
            "forceShow": true,\
            "publishedAt": "<string>",\
            "createdBy": 123,\
            "updatedBy": 123,\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "forceHide": true,\
            "isCarousel": true\
          }\
        ],\
        "creator": "<string>",\
        "ready": true,\
        "funded": true,\
        "pastSlugs": "<string>",\
        "readyTimestamp": "2023-11-07T05:31:56Z",\
        "fundedTimestamp": "2023-11-07T05:31:56Z",\
        "acceptingOrdersTimestamp": "2023-11-07T05:31:56Z",\
        "competitive": 123,\
        "rewardsMinSize": 123,\
        "rewardsMaxSpread": 123,\
        "spread": 123,\
        "automaticallyResolved": true,\
        "oneDayPriceChange": 123,\
        "oneHourPriceChange": 123,\
        "oneWeekPriceChange": 123,\
        "oneMonthPriceChange": 123,\
        "oneYearPriceChange": 123,\
        "lastTradePrice": 123,\
        "bestBid": 123,\
        "bestAsk": 123,\
        "automaticallyActive": true,\
        "clearBookOnStart": true,\
        "chartColor": "<string>",\
        "seriesColor": "<string>",\
        "showGmpSeries": true,\
        "showGmpOutcome": true,\
        "manualActivation": true,\
        "negRiskOther": true,\
        "gameId": "<string>",\
        "groupItemRange": "<string>",\
        "sportsMarketType": "<string>",\
        "line": 123,\
        "umaResolutionStatuses": "<string>",\
        "pendingDeployment": true,\
        "deploying": true,\
        "deployingTimestamp": "2023-11-07T05:31:56Z",\
        "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",\
        "rfqEnabled": true,\
        "eventStartTime": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "series": [\
      {\
        "id": "<string>",\
        "ticker": "<string>",\
        "slug": "<string>",\
        "title": "<string>",\
        "subtitle": "<string>",\
        "seriesType": "<string>",\
        "recurrence": "<string>",\
        "description": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "layout": "<string>",\
        "active": true,\
        "closed": true,\
        "archived": true,\
        "new": true,\
        "featured": true,\
        "restricted": true,\
        "isTemplate": true,\
        "templateVariables": true,\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "commentsEnabled": true,\
        "competitive": "<string>",\
        "volume24hr": 123,\
        "volume": 123,\
        "liquidity": 123,\
        "startDate": "2023-11-07T05:31:56Z",\
        "pythTokenID": "<string>",\
        "cgAssetName": "<string>",\
        "score": 123,\
        "events": "<array>",\
        "collections": [\
          {\
            "id": "<string>",\
            "ticker": "<string>",\
            "slug": "<string>",\
            "title": "<string>",\
            "subtitle": "<string>",\
            "collectionType": "<string>",\
            "description": "<string>",\
            "tags": "<string>",\
            "image": "<string>",\
            "icon": "<string>",\
            "headerImage": "<string>",\
            "layout": "<string>",\
            "active": true,\
            "closed": true,\
            "archived": true,\
            "new": true,\
            "featured": true,\
            "restricted": true,\
            "isTemplate": true,\
            "templateVariables": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "commentsEnabled": true,\
            "imageOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            },\
            "iconOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            },\
            "headerImageOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            }\
          }\
        ],\
        "categories": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "parentCategory": "<string>",\
            "slug": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z"\
          }\
        ],\
        "tags": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "slug": "<string>",\
            "forceShow": true,\
            "publishedAt": "<string>",\
            "createdBy": 123,\
            "updatedBy": 123,\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "forceHide": true,\
            "isCarousel": true\
          }\
        ],\
        "commentCount": 123,\
        "chats": [\
          {\
            "id": "<string>",\
            "channelId": "<string>",\
            "channelName": "<string>",\
            "channelImage": "<string>",\
            "live": true,\
            "startTime": "2023-11-07T05:31:56Z",\
            "endTime": "2023-11-07T05:31:56Z"\
          }\
        ]\
      }\
    ],\
    "categories": [\
      {\
        "id": "<string>",\
        "label": "<string>",\
        "parentCategory": "<string>",\
        "slug": "<string>",\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "collections": [\
      {\
        "id": "<string>",\
        "ticker": "<string>",\
        "slug": "<string>",\
        "title": "<string>",\
        "subtitle": "<string>",\
        "collectionType": "<string>",\
        "description": "<string>",\
        "tags": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "headerImage": "<string>",\
        "layout": "<string>",\
        "active": true,\
        "closed": true,\
        "archived": true,\
        "new": true,\
        "featured": true,\
        "restricted": true,\
        "isTemplate": true,\
        "templateVariables": "<string>",\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "commentsEnabled": true,\
        "imageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "iconOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "headerImageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        }\
      }\
    ],\
    "tags": [\
      {\
        "id": "<string>",\
        "label": "<string>",\
        "slug": "<string>",\
        "forceShow": true,\
        "publishedAt": "<string>",\
        "createdBy": 123,\
        "updatedBy": 123,\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "forceHide": true,\
        "isCarousel": true\
      }\
    ],\
    "cyom": true,\
    "closedTime": "2023-11-07T05:31:56Z",\
    "showAllOutcomes": true,\
    "showMarketImages": true,\
    "automaticallyResolved": true,\
    "enableNegRisk": true,\
    "automaticallyActive": true,\
    "eventDate": "<string>",\
    "startTime": "2023-11-07T05:31:56Z",\
    "eventWeek": 123,\
    "seriesSlug": "<string>",\
    "score": "<string>",\
    "elapsed": "<string>",\
    "period": "<string>",\
    "live": true,\
    "ended": true,\
    "finishedTimestamp": "2023-11-07T05:31:56Z",\
    "gmpChartMode": "<string>",\
    "eventCreators": [\
      {\
        "id": "<string>",\
        "creatorName": "<string>",\
        "creatorHandle": "<string>",\
        "creatorUrl": "<string>",\
        "creatorImage": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "tweetCount": 123,\
    "chats": [\
      {\
        "id": "<string>",\
        "channelId": "<string>",\
        "channelName": "<string>",\
        "channelImage": "<string>",\
        "live": true,\
        "startTime": "2023-11-07T05:31:56Z",\
        "endTime": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "featuredOrder": 123,\
    "estimateValue": true,\
    "cantEstimate": true,\
    "estimatedValue": "<string>",\
    "templates": [\
      {\
        "id": "<string>",\
        "eventTitle": "<string>",\
        "eventSlug": "<string>",\
        "eventImage": "<string>",\
        "marketTitle": "<string>",\
        "description": "<string>",\
        "resolutionSource": "<string>",\
        "negRisk": true,\
        "sortBy": "<string>",\
        "showMarketImages": true,\
        "seriesSlug": "<string>",\
        "outcomes": "<string>"\
      }\
    ],\
    "spreadsMainLine": 123,\
    "totalsMainLine": 123,\
    "carouselMap": "<string>",\
    "pendingDeployment": true,\
    "deploying": true,\
    "deployingTimestamp": "2023-11-07T05:31:56Z",\
    "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",\
    "gameStatus": "<string>"\
  }\
]
```

GET

/

events

Try it

List events

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/events
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "ticker": "<string>",\
    "slug": "<string>",\
    "title": "<string>",\
    "subtitle": "<string>",\
    "description": "<string>",\
    "resolutionSource": "<string>",\
    "startDate": "2023-11-07T05:31:56Z",\
    "creationDate": "2023-11-07T05:31:56Z",\
    "endDate": "2023-11-07T05:31:56Z",\
    "image": "<string>",\
    "icon": "<string>",\
    "active": true,\
    "closed": true,\
    "archived": true,\
    "new": true,\
    "featured": true,\
    "restricted": true,\
    "liquidity": 123,\
    "volume": 123,\
    "openInterest": 123,\
    "sortBy": "<string>",\
    "category": "<string>",\
    "subcategory": "<string>",\
    "isTemplate": true,\
    "templateVariables": "<string>",\
    "published_at": "<string>",\
    "createdBy": "<string>",\
    "updatedBy": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "commentsEnabled": true,\
    "competitive": 123,\
    "volume24hr": 123,\
    "volume1wk": 123,\
    "volume1mo": 123,\
    "volume1yr": 123,\
    "featuredImage": "<string>",\
    "disqusThread": "<string>",\
    "parentEvent": "<string>",\
    "enableOrderBook": true,\
    "liquidityAmm": 123,\
    "liquidityClob": 123,\
    "negRisk": true,\
    "negRiskMarketID": "<string>",\
    "negRiskFeeBips": 123,\
    "commentCount": 123,\
    "imageOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "iconOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "featuredImageOptimized": {\
      "id": "<string>",\
      "imageUrlSource": "<string>",\
      "imageUrlOptimized": "<string>",\
      "imageSizeKbSource": 123,\
      "imageSizeKbOptimized": 123,\
      "imageOptimizedComplete": true,\
      "imageOptimizedLastUpdated": "<string>",\
      "relID": 123,\
      "field": "<string>",\
      "relname": "<string>"\
    },\
    "subEvents": [\
      "<string>"\
    ],\
    "markets": [\
      {\
        "id": "<string>",\
        "question": "<string>",\
        "conditionId": "<string>",\
        "slug": "<string>",\
        "twitterCardImage": "<string>",\
        "resolutionSource": "<string>",\
        "endDate": "2023-11-07T05:31:56Z",\
        "category": "<string>",\
        "ammType": "<string>",\
        "liquidity": "<string>",\
        "sponsorName": "<string>",\
        "sponsorImage": "<string>",\
        "startDate": "2023-11-07T05:31:56Z",\
        "xAxisValue": "<string>",\
        "yAxisValue": "<string>",\
        "denominationToken": "<string>",\
        "fee": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "lowerBound": "<string>",\
        "upperBound": "<string>",\
        "description": "<string>",\
        "outcomes": "<string>",\
        "outcomePrices": "<string>",\
        "volume": "<string>",\
        "active": true,\
        "marketType": "<string>",\
        "formatType": "<string>",\
        "lowerBoundDate": "<string>",\
        "upperBoundDate": "<string>",\
        "closed": true,\
        "marketMakerAddress": "<string>",\
        "createdBy": 123,\
        "updatedBy": 123,\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "closedTime": "<string>",\
        "wideFormat": true,\
        "new": true,\
        "mailchimpTag": "<string>",\
        "featured": true,\
        "archived": true,\
        "resolvedBy": "<string>",\
        "restricted": true,\
        "marketGroup": 123,\
        "groupItemTitle": "<string>",\
        "groupItemThreshold": "<string>",\
        "questionID": "<string>",\
        "umaEndDate": "<string>",\
        "enableOrderBook": true,\
        "orderPriceMinTickSize": 123,\
        "orderMinSize": 123,\
        "umaResolutionStatus": "<string>",\
        "curationOrder": 123,\
        "volumeNum": 123,\
        "liquidityNum": 123,\
        "endDateIso": "<string>",\
        "startDateIso": "<string>",\
        "umaEndDateIso": "<string>",\
        "hasReviewedDates": true,\
        "readyForCron": true,\
        "commentsEnabled": true,\
        "volume24hr": 123,\
        "volume1wk": 123,\
        "volume1mo": 123,\
        "volume1yr": 123,\
        "gameStartTime": "<string>",\
        "secondsDelay": 123,\
        "clobTokenIds": "<string>",\
        "disqusThread": "<string>",\
        "shortOutcomes": "<string>",\
        "teamAID": "<string>",\
        "teamBID": "<string>",\
        "umaBond": "<string>",\
        "umaReward": "<string>",\
        "fpmmLive": true,\
        "volume24hrAmm": 123,\
        "volume1wkAmm": 123,\
        "volume1moAmm": 123,\
        "volume1yrAmm": 123,\
        "volume24hrClob": 123,\
        "volume1wkClob": 123,\
        "volume1moClob": 123,\
        "volume1yrClob": 123,\
        "volumeAmm": 123,\
        "volumeClob": 123,\
        "liquidityAmm": 123,\
        "liquidityClob": 123,\
        "makerBaseFee": 123,\
        "takerBaseFee": 123,\
        "customLiveness": 123,\
        "acceptingOrders": true,\
        "notificationsEnabled": true,\
        "score": 123,\
        "imageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "iconOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "events": "<array>",\
        "categories": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "parentCategory": "<string>",\
            "slug": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z"\
          }\
        ],\
        "tags": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "slug": "<string>",\
            "forceShow": true,\
            "publishedAt": "<string>",\
            "createdBy": 123,\
            "updatedBy": 123,\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "forceHide": true,\
            "isCarousel": true\
          }\
        ],\
        "creator": "<string>",\
        "ready": true,\
        "funded": true,\
        "pastSlugs": "<string>",\
        "readyTimestamp": "2023-11-07T05:31:56Z",\
        "fundedTimestamp": "2023-11-07T05:31:56Z",\
        "acceptingOrdersTimestamp": "2023-11-07T05:31:56Z",\
        "competitive": 123,\
        "rewardsMinSize": 123,\
        "rewardsMaxSpread": 123,\
        "spread": 123,\
        "automaticallyResolved": true,\
        "oneDayPriceChange": 123,\
        "oneHourPriceChange": 123,\
        "oneWeekPriceChange": 123,\
        "oneMonthPriceChange": 123,\
        "oneYearPriceChange": 123,\
        "lastTradePrice": 123,\
        "bestBid": 123,\
        "bestAsk": 123,\
        "automaticallyActive": true,\
        "clearBookOnStart": true,\
        "chartColor": "<string>",\
        "seriesColor": "<string>",\
        "showGmpSeries": true,\
        "showGmpOutcome": true,\
        "manualActivation": true,\
        "negRiskOther": true,\
        "gameId": "<string>",\
        "groupItemRange": "<string>",\
        "sportsMarketType": "<string>",\
        "line": 123,\
        "umaResolutionStatuses": "<string>",\
        "pendingDeployment": true,\
        "deploying": true,\
        "deployingTimestamp": "2023-11-07T05:31:56Z",\
        "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",\
        "rfqEnabled": true,\
        "eventStartTime": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "series": [\
      {\
        "id": "<string>",\
        "ticker": "<string>",\
        "slug": "<string>",\
        "title": "<string>",\
        "subtitle": "<string>",\
        "seriesType": "<string>",\
        "recurrence": "<string>",\
        "description": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "layout": "<string>",\
        "active": true,\
        "closed": true,\
        "archived": true,\
        "new": true,\
        "featured": true,\
        "restricted": true,\
        "isTemplate": true,\
        "templateVariables": true,\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "commentsEnabled": true,\
        "competitive": "<string>",\
        "volume24hr": 123,\
        "volume": 123,\
        "liquidity": 123,\
        "startDate": "2023-11-07T05:31:56Z",\
        "pythTokenID": "<string>",\
        "cgAssetName": "<string>",\
        "score": 123,\
        "events": "<array>",\
        "collections": [\
          {\
            "id": "<string>",\
            "ticker": "<string>",\
            "slug": "<string>",\
            "title": "<string>",\
            "subtitle": "<string>",\
            "collectionType": "<string>",\
            "description": "<string>",\
            "tags": "<string>",\
            "image": "<string>",\
            "icon": "<string>",\
            "headerImage": "<string>",\
            "layout": "<string>",\
            "active": true,\
            "closed": true,\
            "archived": true,\
            "new": true,\
            "featured": true,\
            "restricted": true,\
            "isTemplate": true,\
            "templateVariables": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "commentsEnabled": true,\
            "imageOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            },\
            "iconOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            },\
            "headerImageOptimized": {\
              "id": "<string>",\
              "imageUrlSource": "<string>",\
              "imageUrlOptimized": "<string>",\
              "imageSizeKbSource": 123,\
              "imageSizeKbOptimized": 123,\
              "imageOptimizedComplete": true,\
              "imageOptimizedLastUpdated": "<string>",\
              "relID": 123,\
              "field": "<string>",\
              "relname": "<string>"\
            }\
          }\
        ],\
        "categories": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "parentCategory": "<string>",\
            "slug": "<string>",\
            "publishedAt": "<string>",\
            "createdBy": "<string>",\
            "updatedBy": "<string>",\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z"\
          }\
        ],\
        "tags": [\
          {\
            "id": "<string>",\
            "label": "<string>",\
            "slug": "<string>",\
            "forceShow": true,\
            "publishedAt": "<string>",\
            "createdBy": 123,\
            "updatedBy": 123,\
            "createdAt": "2023-11-07T05:31:56Z",\
            "updatedAt": "2023-11-07T05:31:56Z",\
            "forceHide": true,\
            "isCarousel": true\
          }\
        ],\
        "commentCount": 123,\
        "chats": [\
          {\
            "id": "<string>",\
            "channelId": "<string>",\
            "channelName": "<string>",\
            "channelImage": "<string>",\
            "live": true,\
            "startTime": "2023-11-07T05:31:56Z",\
            "endTime": "2023-11-07T05:31:56Z"\
          }\
        ]\
      }\
    ],\
    "categories": [\
      {\
        "id": "<string>",\
        "label": "<string>",\
        "parentCategory": "<string>",\
        "slug": "<string>",\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "collections": [\
      {\
        "id": "<string>",\
        "ticker": "<string>",\
        "slug": "<string>",\
        "title": "<string>",\
        "subtitle": "<string>",\
        "collectionType": "<string>",\
        "description": "<string>",\
        "tags": "<string>",\
        "image": "<string>",\
        "icon": "<string>",\
        "headerImage": "<string>",\
        "layout": "<string>",\
        "active": true,\
        "closed": true,\
        "archived": true,\
        "new": true,\
        "featured": true,\
        "restricted": true,\
        "isTemplate": true,\
        "templateVariables": "<string>",\
        "publishedAt": "<string>",\
        "createdBy": "<string>",\
        "updatedBy": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "commentsEnabled": true,\
        "imageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "iconOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        },\
        "headerImageOptimized": {\
          "id": "<string>",\
          "imageUrlSource": "<string>",\
          "imageUrlOptimized": "<string>",\
          "imageSizeKbSource": 123,\
          "imageSizeKbOptimized": 123,\
          "imageOptimizedComplete": true,\
          "imageOptimizedLastUpdated": "<string>",\
          "relID": 123,\
          "field": "<string>",\
          "relname": "<string>"\
        }\
      }\
    ],\
    "tags": [\
      {\
        "id": "<string>",\
        "label": "<string>",\
        "slug": "<string>",\
        "forceShow": true,\
        "publishedAt": "<string>",\
        "createdBy": 123,\
        "updatedBy": 123,\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z",\
        "forceHide": true,\
        "isCarousel": true\
      }\
    ],\
    "cyom": true,\
    "closedTime": "2023-11-07T05:31:56Z",\
    "showAllOutcomes": true,\
    "showMarketImages": true,\
    "automaticallyResolved": true,\
    "enableNegRisk": true,\
    "automaticallyActive": true,\
    "eventDate": "<string>",\
    "startTime": "2023-11-07T05:31:56Z",\
    "eventWeek": 123,\
    "seriesSlug": "<string>",\
    "score": "<string>",\
    "elapsed": "<string>",\
    "period": "<string>",\
    "live": true,\
    "ended": true,\
    "finishedTimestamp": "2023-11-07T05:31:56Z",\
    "gmpChartMode": "<string>",\
    "eventCreators": [\
      {\
        "id": "<string>",\
        "creatorName": "<string>",\
        "creatorHandle": "<string>",\
        "creatorUrl": "<string>",\
        "creatorImage": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "updatedAt": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "tweetCount": 123,\
    "chats": [\
      {\
        "id": "<string>",\
        "channelId": "<string>",\
        "channelName": "<string>",\
        "channelImage": "<string>",\
        "live": true,\
        "startTime": "2023-11-07T05:31:56Z",\
        "endTime": "2023-11-07T05:31:56Z"\
      }\
    ],\
    "featuredOrder": 123,\
    "estimateValue": true,\
    "cantEstimate": true,\
    "estimatedValue": "<string>",\
    "templates": [\
      {\
        "id": "<string>",\
        "eventTitle": "<string>",\
        "eventSlug": "<string>",\
        "eventImage": "<string>",\
        "marketTitle": "<string>",\
        "description": "<string>",\
        "resolutionSource": "<string>",\
        "negRisk": true,\
        "sortBy": "<string>",\
        "showMarketImages": true,\
        "seriesSlug": "<string>",\
        "outcomes": "<string>"\
      }\
    ],\
    "spreadsMainLine": 123,\
    "totalsMainLine": 123,\
    "carouselMap": "<string>",\
    "pendingDeployment": true,\
    "deploying": true,\
    "deployingTimestamp": "2023-11-07T05:31:56Z",\
    "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",\
    "gameStatus": "<string>"\
  }\
]
```

# [​](https://docs.polymarket.com/developers/gamma-markets-api/get-events\#events)  Events

Get events.

> **Note:** Markets can be traded via the CLOB _if_`enableOrderBook` is `true`.

#### Query Parameters

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-limit)

limit

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-offset)

offset

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-order)

order

string

Comma-separated list of fields to order by

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-ascending)

ascending

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-id)

id

integer\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-slug)

slug

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-tag-id)

tag\_id

integer

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-exclude-tag-id)

exclude\_tag\_id

integer\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-related-tags)

related\_tags

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-featured)

featured

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-cyom)

cyom

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-include-chat)

include\_chat

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-include-template)

include\_template

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-recurrence)

recurrence

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-closed)

closed

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-start-date-min)

start\_date\_min

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-start-date-max)

start\_date\_max

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-end-date-min)

end\_date\_min

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#parameter-end-date-max)

end\_date\_max

string<date-time>

#### Response

200 - application/json

List of events

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-id)

id

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-ticker-one-of-0)

ticker

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-title-one-of-0)

title

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-subtitle-one-of-0)

subtitle

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-description-one-of-0)

description

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-resolution-source-one-of-0)

resolutionSource

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-start-date-one-of-0)

startDate

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-creation-date-one-of-0)

creationDate

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-end-date-one-of-0)

endDate

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-image-one-of-0)

image

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-icon-one-of-0)

icon

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-active-one-of-0)

active

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-closed-one-of-0)

closed

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-archived-one-of-0)

archived

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-new-one-of-0)

new

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-featured-one-of-0)

featured

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-restricted-one-of-0)

restricted

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-liquidity-one-of-0)

liquidity

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-volume-one-of-0)

volume

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-open-interest-one-of-0)

openInterest

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-sort-by-one-of-0)

sortBy

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-category-one-of-0)

category

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-subcategory-one-of-0)

subcategory

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-is-template-one-of-0)

isTemplate

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-template-variables-one-of-0)

templateVariables

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-published-at-one-of-0)

published\_at

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-created-by-one-of-0)

createdBy

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-updated-by-one-of-0)

updatedBy

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-comments-enabled-one-of-0)

commentsEnabled

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-competitive-one-of-0)

competitive

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-volume24hr-one-of-0)

volume24hr

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-volume1wk-one-of-0)

volume1wk

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-volume1mo-one-of-0)

volume1mo

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-volume1yr-one-of-0)

volume1yr

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-featured-image-one-of-0)

featuredImage

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-disqus-thread-one-of-0)

disqusThread

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-parent-event-one-of-0)

parentEvent

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-enable-order-book-one-of-0)

enableOrderBook

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-liquidity-amm-one-of-0)

liquidityAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-liquidity-clob-one-of-0)

liquidityClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-neg-risk-one-of-0)

negRisk

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-neg-risk-market-id-one-of-0)

negRiskMarketID

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-neg-risk-fee-bips-one-of-0)

negRiskFeeBips

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-comment-count-one-of-0)

commentCount

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-image-optimized)

imageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-icon-optimized)

iconOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-featured-image-optimized)

featuredImageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-sub-events-one-of-0)

subEvents

string\[\] \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-markets)

markets

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-series)

series

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-categories)

categories

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-collections)

collections

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-tags)

tags

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-cyom-one-of-0)

cyom

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-closed-time-one-of-0)

closedTime

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-show-all-outcomes-one-of-0)

showAllOutcomes

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-show-market-images-one-of-0)

showMarketImages

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-automatically-resolved-one-of-0)

automaticallyResolved

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-enable-neg-risk-one-of-0)

enableNegRisk

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-automatically-active-one-of-0)

automaticallyActive

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-event-date-one-of-0)

eventDate

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-start-time-one-of-0)

startTime

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-event-week-one-of-0)

eventWeek

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-series-slug-one-of-0)

seriesSlug

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-score-one-of-0)

score

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-elapsed-one-of-0)

elapsed

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-period-one-of-0)

period

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-live-one-of-0)

live

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-ended-one-of-0)

ended

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-finished-timestamp-one-of-0)

finishedTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-gmp-chart-mode-one-of-0)

gmpChartMode

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-event-creators)

eventCreators

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-tweet-count-one-of-0)

tweetCount

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-chats)

chats

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-featured-order-one-of-0)

featuredOrder

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-estimate-value-one-of-0)

estimateValue

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-cant-estimate-one-of-0)

cantEstimate

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-estimated-value-one-of-0)

estimatedValue

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-templates)

templates

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-spreads-main-line-one-of-0)

spreadsMainLine

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-totals-main-line-one-of-0)

totalsMainLine

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-carousel-map-one-of-0)

carouselMap

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-pending-deployment-one-of-0)

pendingDeployment

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-deploying-one-of-0)

deploying

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-deploying-timestamp-one-of-0)

deployingTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-scheduled-deployment-timestamp-one-of-0)

scheduledDeploymentTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-events#response-items-game-status-one-of-0)

gameStatus

string \| null

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.