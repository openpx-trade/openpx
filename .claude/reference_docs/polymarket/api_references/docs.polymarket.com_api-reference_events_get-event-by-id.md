[Skip to main content](https://docs.polymarket.com/api-reference/events/get-event-by-id#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Events

Get event by id

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get event by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/events/{id}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "ticker": "<string>",
  "slug": "<string>",
  "title": "<string>",
  "subtitle": "<string>",
  "description": "<string>",
  "resolutionSource": "<string>",
  "startDate": "2023-11-07T05:31:56Z",
  "creationDate": "2023-11-07T05:31:56Z",
  "endDate": "2023-11-07T05:31:56Z",
  "image": "<string>",
  "icon": "<string>",
  "active": true,
  "closed": true,
  "archived": true,
  "new": true,
  "featured": true,
  "restricted": true,
  "liquidity": 123,
  "volume": 123,
  "openInterest": 123,
  "sortBy": "<string>",
  "category": "<string>",
  "subcategory": "<string>",
  "isTemplate": true,
  "templateVariables": "<string>",
  "published_at": "<string>",
  "createdBy": "<string>",
  "updatedBy": "<string>",
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "commentsEnabled": true,
  "competitive": 123,
  "volume24hr": 123,
  "volume1wk": 123,
  "volume1mo": 123,
  "volume1yr": 123,
  "featuredImage": "<string>",
  "disqusThread": "<string>",
  "parentEvent": "<string>",
  "enableOrderBook": true,
  "liquidityAmm": 123,
  "liquidityClob": 123,
  "negRisk": true,
  "negRiskMarketID": "<string>",
  "negRiskFeeBips": 123,
  "commentCount": 123,
  "imageOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "iconOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "featuredImageOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "subEvents": [\
    "<string>"\
  ],
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
  ],
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
  ],
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
  ],
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
  ],
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
  ],
  "cyom": true,
  "closedTime": "2023-11-07T05:31:56Z",
  "showAllOutcomes": true,
  "showMarketImages": true,
  "automaticallyResolved": true,
  "enableNegRisk": true,
  "automaticallyActive": true,
  "eventDate": "<string>",
  "startTime": "2023-11-07T05:31:56Z",
  "eventWeek": 123,
  "seriesSlug": "<string>",
  "score": "<string>",
  "elapsed": "<string>",
  "period": "<string>",
  "live": true,
  "ended": true,
  "finishedTimestamp": "2023-11-07T05:31:56Z",
  "gmpChartMode": "<string>",
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
  ],
  "tweetCount": 123,
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
  ],
  "featuredOrder": 123,
  "estimateValue": true,
  "cantEstimate": true,
  "estimatedValue": "<string>",
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
  ],
  "spreadsMainLine": 123,
  "totalsMainLine": 123,
  "carouselMap": "<string>",
  "pendingDeployment": true,
  "deploying": true,
  "deployingTimestamp": "2023-11-07T05:31:56Z",
  "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",
  "gameStatus": "<string>"
}
```

GET

/

events

/

{id}

Try it

Get event by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/events/{id}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "ticker": "<string>",
  "slug": "<string>",
  "title": "<string>",
  "subtitle": "<string>",
  "description": "<string>",
  "resolutionSource": "<string>",
  "startDate": "2023-11-07T05:31:56Z",
  "creationDate": "2023-11-07T05:31:56Z",
  "endDate": "2023-11-07T05:31:56Z",
  "image": "<string>",
  "icon": "<string>",
  "active": true,
  "closed": true,
  "archived": true,
  "new": true,
  "featured": true,
  "restricted": true,
  "liquidity": 123,
  "volume": 123,
  "openInterest": 123,
  "sortBy": "<string>",
  "category": "<string>",
  "subcategory": "<string>",
  "isTemplate": true,
  "templateVariables": "<string>",
  "published_at": "<string>",
  "createdBy": "<string>",
  "updatedBy": "<string>",
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "commentsEnabled": true,
  "competitive": 123,
  "volume24hr": 123,
  "volume1wk": 123,
  "volume1mo": 123,
  "volume1yr": 123,
  "featuredImage": "<string>",
  "disqusThread": "<string>",
  "parentEvent": "<string>",
  "enableOrderBook": true,
  "liquidityAmm": 123,
  "liquidityClob": 123,
  "negRisk": true,
  "negRiskMarketID": "<string>",
  "negRiskFeeBips": 123,
  "commentCount": 123,
  "imageOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "iconOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "featuredImageOptimized": {
    "id": "<string>",
    "imageUrlSource": "<string>",
    "imageUrlOptimized": "<string>",
    "imageSizeKbSource": 123,
    "imageSizeKbOptimized": 123,
    "imageOptimizedComplete": true,
    "imageOptimizedLastUpdated": "<string>",
    "relID": 123,
    "field": "<string>",
    "relname": "<string>"
  },
  "subEvents": [\
    "<string>"\
  ],
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
  ],
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
  ],
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
  ],
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
  ],
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
  ],
  "cyom": true,
  "closedTime": "2023-11-07T05:31:56Z",
  "showAllOutcomes": true,
  "showMarketImages": true,
  "automaticallyResolved": true,
  "enableNegRisk": true,
  "automaticallyActive": true,
  "eventDate": "<string>",
  "startTime": "2023-11-07T05:31:56Z",
  "eventWeek": 123,
  "seriesSlug": "<string>",
  "score": "<string>",
  "elapsed": "<string>",
  "period": "<string>",
  "live": true,
  "ended": true,
  "finishedTimestamp": "2023-11-07T05:31:56Z",
  "gmpChartMode": "<string>",
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
  ],
  "tweetCount": 123,
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
  ],
  "featuredOrder": 123,
  "estimateValue": true,
  "cantEstimate": true,
  "estimatedValue": "<string>",
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
  ],
  "spreadsMainLine": 123,
  "totalsMainLine": 123,
  "carouselMap": "<string>",
  "pendingDeployment": true,
  "deploying": true,
  "deployingTimestamp": "2023-11-07T05:31:56Z",
  "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",
  "gameStatus": "<string>"
}
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#parameter-id)

id

integer

required

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#parameter-include-chat)

include\_chat

boolean

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#parameter-include-template)

include\_template

boolean

#### Response

200

application/json

Event

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-id)

id

string

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-ticker-one-of-0)

ticker

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-title-one-of-0)

title

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-subtitle-one-of-0)

subtitle

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-description-one-of-0)

description

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-resolution-source-one-of-0)

resolutionSource

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-start-date-one-of-0)

startDate

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-creation-date-one-of-0)

creationDate

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-end-date-one-of-0)

endDate

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-image-one-of-0)

image

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-icon-one-of-0)

icon

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-active-one-of-0)

active

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-closed-one-of-0)

closed

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-archived-one-of-0)

archived

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-new-one-of-0)

new

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-featured-one-of-0)

featured

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-restricted-one-of-0)

restricted

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-liquidity-one-of-0)

liquidity

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-volume-one-of-0)

volume

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-open-interest-one-of-0)

openInterest

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-sort-by-one-of-0)

sortBy

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-category-one-of-0)

category

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-subcategory-one-of-0)

subcategory

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-is-template-one-of-0)

isTemplate

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-template-variables-one-of-0)

templateVariables

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-published-at-one-of-0)

published\_at

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-created-by-one-of-0)

createdBy

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-updated-by-one-of-0)

updatedBy

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-comments-enabled-one-of-0)

commentsEnabled

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-competitive-one-of-0)

competitive

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-volume24hr-one-of-0)

volume24hr

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-volume1wk-one-of-0)

volume1wk

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-volume1mo-one-of-0)

volume1mo

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-volume1yr-one-of-0)

volume1yr

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-featured-image-one-of-0)

featuredImage

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-disqus-thread-one-of-0)

disqusThread

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-parent-event-one-of-0)

parentEvent

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-enable-order-book-one-of-0)

enableOrderBook

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-liquidity-amm-one-of-0)

liquidityAmm

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-liquidity-clob-one-of-0)

liquidityClob

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-neg-risk-one-of-0)

negRisk

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-neg-risk-market-id-one-of-0)

negRiskMarketID

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-neg-risk-fee-bips-one-of-0)

negRiskFeeBips

integer \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-comment-count-one-of-0)

commentCount

integer \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-image-optimized)

imageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-icon-optimized)

iconOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-featured-image-optimized)

featuredImageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-sub-events-one-of-0)

subEvents

string\[\] \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-markets)

markets

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-series)

series

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-categories)

categories

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-collections)

collections

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-tags)

tags

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-cyom-one-of-0)

cyom

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-closed-time-one-of-0)

closedTime

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-show-all-outcomes-one-of-0)

showAllOutcomes

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-show-market-images-one-of-0)

showMarketImages

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-automatically-resolved-one-of-0)

automaticallyResolved

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-enable-neg-risk-one-of-0)

enableNegRisk

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-automatically-active-one-of-0)

automaticallyActive

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-event-date-one-of-0)

eventDate

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-start-time-one-of-0)

startTime

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-event-week-one-of-0)

eventWeek

integer \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-series-slug-one-of-0)

seriesSlug

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-score-one-of-0)

score

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-elapsed-one-of-0)

elapsed

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-period-one-of-0)

period

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-live-one-of-0)

live

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-ended-one-of-0)

ended

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-finished-timestamp-one-of-0)

finishedTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-gmp-chart-mode-one-of-0)

gmpChartMode

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-event-creators)

eventCreators

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-tweet-count-one-of-0)

tweetCount

integer \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-chats)

chats

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-featured-order-one-of-0)

featuredOrder

integer \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-estimate-value-one-of-0)

estimateValue

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-cant-estimate-one-of-0)

cantEstimate

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-estimated-value-one-of-0)

estimatedValue

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-templates)

templates

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-spreads-main-line-one-of-0)

spreadsMainLine

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-totals-main-line-one-of-0)

totalsMainLine

number \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-carousel-map-one-of-0)

carouselMap

string \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-pending-deployment-one-of-0)

pendingDeployment

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-deploying-one-of-0)

deploying

boolean \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-deploying-timestamp-one-of-0)

deployingTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-scheduled-deployment-timestamp-one-of-0)

scheduledDeploymentTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/events/get-event-by-id#response-game-status-one-of-0)

gameStatus

string \| null

[List events](https://docs.polymarket.com/api-reference/events/list-events) [Get event tags](https://docs.polymarket.com/api-reference/events/get-event-tags)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.