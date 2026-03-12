---
url: "https://docs.polymarket.com/developers/gamma-markets-api/get-markets"
title: "Get Markets - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Get Markets

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

List markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets
```

200

Copy

Ask AI

```
[\
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
    "events": [\
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
        "markets": "<array>",\
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
]
```

GET

/

markets

Try it

List markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets
```

200

Copy

Ask AI

```
[\
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
    "events": [\
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
        "markets": "<array>",\
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
]
```

# [​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets\#markets)  Markets

Get markets.

> **Note:** Markets can be traded via the CLOB _if_`enableOrderBook` is `true`.

#### Query Parameters

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-limit)

limit

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-offset)

offset

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-order)

order

string

Comma-separated list of fields to order by

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-ascending)

ascending

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-id)

id

integer\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-slug)

slug

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-clob-token-ids)

clob\_token\_ids

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-condition-ids)

condition\_ids

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-market-maker-address)

market\_maker\_address

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-liquidity-num-min)

liquidity\_num\_min

number

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-liquidity-num-max)

liquidity\_num\_max

number

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-volume-num-min)

volume\_num\_min

number

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-volume-num-max)

volume\_num\_max

number

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-start-date-min)

start\_date\_min

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-start-date-max)

start\_date\_max

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-end-date-min)

end\_date\_min

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-end-date-max)

end\_date\_max

string<date-time>

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-tag-id)

tag\_id

integer

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-related-tags)

related\_tags

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-cyom)

cyom

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-uma-resolution-status)

uma\_resolution\_status

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-game-id)

game\_id

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-sports-market-types)

sports\_market\_types

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-rewards-min-size)

rewards\_min\_size

number

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-question-ids)

question\_ids

string\[\]

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-include-tag)

include\_tag

boolean

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#parameter-closed)

closed

boolean

#### Response

200 - application/json

List of markets

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-id)

id

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-question-one-of-0)

question

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-condition-id)

conditionId

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-twitter-card-image-one-of-0)

twitterCardImage

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-resolution-source-one-of-0)

resolutionSource

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-end-date-one-of-0)

endDate

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-category-one-of-0)

category

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-amm-type-one-of-0)

ammType

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-liquidity-one-of-0)

liquidity

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-sponsor-name-one-of-0)

sponsorName

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-sponsor-image-one-of-0)

sponsorImage

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-start-date-one-of-0)

startDate

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-x-axis-value-one-of-0)

xAxisValue

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-y-axis-value-one-of-0)

yAxisValue

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-denomination-token-one-of-0)

denominationToken

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-fee-one-of-0)

fee

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-image-one-of-0)

image

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-icon-one-of-0)

icon

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-lower-bound-one-of-0)

lowerBound

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-upper-bound-one-of-0)

upperBound

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-description-one-of-0)

description

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-outcomes-one-of-0)

outcomes

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-outcome-prices-one-of-0)

outcomePrices

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume-one-of-0)

volume

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-active-one-of-0)

active

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-market-type-one-of-0)

marketType

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-format-type-one-of-0)

formatType

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-lower-bound-date-one-of-0)

lowerBoundDate

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-upper-bound-date-one-of-0)

upperBoundDate

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-closed-one-of-0)

closed

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-market-maker-address)

marketMakerAddress

string

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-created-by-one-of-0)

createdBy

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-updated-by-one-of-0)

updatedBy

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-closed-time-one-of-0)

closedTime

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-wide-format-one-of-0)

wideFormat

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-new-one-of-0)

new

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-mailchimp-tag-one-of-0)

mailchimpTag

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-featured-one-of-0)

featured

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-archived-one-of-0)

archived

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-resolved-by-one-of-0)

resolvedBy

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-restricted-one-of-0)

restricted

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-market-group-one-of-0)

marketGroup

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-group-item-title-one-of-0)

groupItemTitle

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-group-item-threshold-one-of-0)

groupItemThreshold

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-question-id-one-of-0)

questionID

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-end-date-one-of-0)

umaEndDate

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-enable-order-book-one-of-0)

enableOrderBook

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-order-price-min-tick-size-one-of-0)

orderPriceMinTickSize

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-order-min-size-one-of-0)

orderMinSize

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-resolution-status-one-of-0)

umaResolutionStatus

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-curation-order-one-of-0)

curationOrder

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume-num-one-of-0)

volumeNum

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-liquidity-num-one-of-0)

liquidityNum

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-end-date-iso-one-of-0)

endDateIso

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-start-date-iso-one-of-0)

startDateIso

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-end-date-iso-one-of-0)

umaEndDateIso

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-has-reviewed-dates-one-of-0)

hasReviewedDates

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-ready-for-cron-one-of-0)

readyForCron

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-comments-enabled-one-of-0)

commentsEnabled

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume24hr-one-of-0)

volume24hr

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1wk-one-of-0)

volume1wk

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1mo-one-of-0)

volume1mo

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1yr-one-of-0)

volume1yr

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-game-start-time-one-of-0)

gameStartTime

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-seconds-delay-one-of-0)

secondsDelay

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-clob-token-ids-one-of-0)

clobTokenIds

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-disqus-thread-one-of-0)

disqusThread

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-short-outcomes-one-of-0)

shortOutcomes

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-team-aid-one-of-0)

teamAID

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-team-bid-one-of-0)

teamBID

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-bond-one-of-0)

umaBond

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-reward-one-of-0)

umaReward

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-fpmm-live-one-of-0)

fpmmLive

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume24hr-amm-one-of-0)

volume24hrAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1wk-amm-one-of-0)

volume1wkAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1mo-amm-one-of-0)

volume1moAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1yr-amm-one-of-0)

volume1yrAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume24hr-clob-one-of-0)

volume24hrClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1wk-clob-one-of-0)

volume1wkClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1mo-clob-one-of-0)

volume1moClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume1yr-clob-one-of-0)

volume1yrClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume-amm-one-of-0)

volumeAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-volume-clob-one-of-0)

volumeClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-liquidity-amm-one-of-0)

liquidityAmm

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-liquidity-clob-one-of-0)

liquidityClob

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-maker-base-fee-one-of-0)

makerBaseFee

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-taker-base-fee-one-of-0)

takerBaseFee

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-custom-liveness-one-of-0)

customLiveness

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-accepting-orders-one-of-0)

acceptingOrders

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-notifications-enabled-one-of-0)

notificationsEnabled

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-score-one-of-0)

score

integer \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-image-optimized)

imageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-icon-optimized)

iconOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-events)

events

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-categories)

categories

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-tags)

tags

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-creator-one-of-0)

creator

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-ready-one-of-0)

ready

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-funded-one-of-0)

funded

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-past-slugs-one-of-0)

pastSlugs

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-ready-timestamp-one-of-0)

readyTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-funded-timestamp-one-of-0)

fundedTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-accepting-orders-timestamp-one-of-0)

acceptingOrdersTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-competitive-one-of-0)

competitive

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-rewards-min-size-one-of-0)

rewardsMinSize

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-rewards-max-spread-one-of-0)

rewardsMaxSpread

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-spread-one-of-0)

spread

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-automatically-resolved-one-of-0)

automaticallyResolved

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-one-day-price-change-one-of-0)

oneDayPriceChange

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-one-hour-price-change-one-of-0)

oneHourPriceChange

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-one-week-price-change-one-of-0)

oneWeekPriceChange

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-one-month-price-change-one-of-0)

oneMonthPriceChange

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-one-year-price-change-one-of-0)

oneYearPriceChange

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-last-trade-price-one-of-0)

lastTradePrice

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-best-bid-one-of-0)

bestBid

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-best-ask-one-of-0)

bestAsk

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-automatically-active-one-of-0)

automaticallyActive

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-clear-book-on-start-one-of-0)

clearBookOnStart

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-chart-color-one-of-0)

chartColor

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-series-color-one-of-0)

seriesColor

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-show-gmp-series-one-of-0)

showGmpSeries

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-show-gmp-outcome-one-of-0)

showGmpOutcome

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-manual-activation-one-of-0)

manualActivation

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-neg-risk-other-one-of-0)

negRiskOther

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-game-id-one-of-0)

gameId

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-group-item-range-one-of-0)

groupItemRange

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-sports-market-type-one-of-0)

sportsMarketType

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-line-one-of-0)

line

number \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-uma-resolution-statuses-one-of-0)

umaResolutionStatuses

string \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-pending-deployment-one-of-0)

pendingDeployment

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-deploying-one-of-0)

deploying

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-deploying-timestamp-one-of-0)

deployingTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-scheduled-deployment-timestamp-one-of-0)

scheduledDeploymentTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-rfq-enabled-one-of-0)

rfqEnabled

boolean \| null

[​](https://docs.polymarket.com/developers/gamma-markets-api/get-markets#response-items-event-start-time-one-of-0)

eventStartTime

string<date-time> \| null

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.