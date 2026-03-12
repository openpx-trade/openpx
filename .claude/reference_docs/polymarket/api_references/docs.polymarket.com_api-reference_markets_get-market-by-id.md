[Skip to main content](https://docs.polymarket.com/api-reference/markets/get-market-by-id#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Markets

Get market by id

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get market by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets/{id}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "question": "<string>",
  "conditionId": "<string>",
  "slug": "<string>",
  "twitterCardImage": "<string>",
  "resolutionSource": "<string>",
  "endDate": "2023-11-07T05:31:56Z",
  "category": "<string>",
  "ammType": "<string>",
  "liquidity": "<string>",
  "sponsorName": "<string>",
  "sponsorImage": "<string>",
  "startDate": "2023-11-07T05:31:56Z",
  "xAxisValue": "<string>",
  "yAxisValue": "<string>",
  "denominationToken": "<string>",
  "fee": "<string>",
  "image": "<string>",
  "icon": "<string>",
  "lowerBound": "<string>",
  "upperBound": "<string>",
  "description": "<string>",
  "outcomes": "<string>",
  "outcomePrices": "<string>",
  "volume": "<string>",
  "active": true,
  "marketType": "<string>",
  "formatType": "<string>",
  "lowerBoundDate": "<string>",
  "upperBoundDate": "<string>",
  "closed": true,
  "marketMakerAddress": "<string>",
  "createdBy": 123,
  "updatedBy": 123,
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "closedTime": "<string>",
  "wideFormat": true,
  "new": true,
  "mailchimpTag": "<string>",
  "featured": true,
  "archived": true,
  "resolvedBy": "<string>",
  "restricted": true,
  "marketGroup": 123,
  "groupItemTitle": "<string>",
  "groupItemThreshold": "<string>",
  "questionID": "<string>",
  "umaEndDate": "<string>",
  "enableOrderBook": true,
  "orderPriceMinTickSize": 123,
  "orderMinSize": 123,
  "umaResolutionStatus": "<string>",
  "curationOrder": 123,
  "volumeNum": 123,
  "liquidityNum": 123,
  "endDateIso": "<string>",
  "startDateIso": "<string>",
  "umaEndDateIso": "<string>",
  "hasReviewedDates": true,
  "readyForCron": true,
  "commentsEnabled": true,
  "volume24hr": 123,
  "volume1wk": 123,
  "volume1mo": 123,
  "volume1yr": 123,
  "gameStartTime": "<string>",
  "secondsDelay": 123,
  "clobTokenIds": "<string>",
  "disqusThread": "<string>",
  "shortOutcomes": "<string>",
  "teamAID": "<string>",
  "teamBID": "<string>",
  "umaBond": "<string>",
  "umaReward": "<string>",
  "fpmmLive": true,
  "volume24hrAmm": 123,
  "volume1wkAmm": 123,
  "volume1moAmm": 123,
  "volume1yrAmm": 123,
  "volume24hrClob": 123,
  "volume1wkClob": 123,
  "volume1moClob": 123,
  "volume1yrClob": 123,
  "volumeAmm": 123,
  "volumeClob": 123,
  "liquidityAmm": 123,
  "liquidityClob": 123,
  "makerBaseFee": 123,
  "takerBaseFee": 123,
  "customLiveness": 123,
  "acceptingOrders": true,
  "notificationsEnabled": true,
  "score": 123,
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
  "creator": "<string>",
  "ready": true,
  "funded": true,
  "pastSlugs": "<string>",
  "readyTimestamp": "2023-11-07T05:31:56Z",
  "fundedTimestamp": "2023-11-07T05:31:56Z",
  "acceptingOrdersTimestamp": "2023-11-07T05:31:56Z",
  "competitive": 123,
  "rewardsMinSize": 123,
  "rewardsMaxSpread": 123,
  "spread": 123,
  "automaticallyResolved": true,
  "oneDayPriceChange": 123,
  "oneHourPriceChange": 123,
  "oneWeekPriceChange": 123,
  "oneMonthPriceChange": 123,
  "oneYearPriceChange": 123,
  "lastTradePrice": 123,
  "bestBid": 123,
  "bestAsk": 123,
  "automaticallyActive": true,
  "clearBookOnStart": true,
  "chartColor": "<string>",
  "seriesColor": "<string>",
  "showGmpSeries": true,
  "showGmpOutcome": true,
  "manualActivation": true,
  "negRiskOther": true,
  "gameId": "<string>",
  "groupItemRange": "<string>",
  "sportsMarketType": "<string>",
  "line": 123,
  "umaResolutionStatuses": "<string>",
  "pendingDeployment": true,
  "deploying": true,
  "deployingTimestamp": "2023-11-07T05:31:56Z",
  "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",
  "rfqEnabled": true,
  "eventStartTime": "2023-11-07T05:31:56Z"
}
```

GET

/

markets

/

{id}

Try it

Get market by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets/{id}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "question": "<string>",
  "conditionId": "<string>",
  "slug": "<string>",
  "twitterCardImage": "<string>",
  "resolutionSource": "<string>",
  "endDate": "2023-11-07T05:31:56Z",
  "category": "<string>",
  "ammType": "<string>",
  "liquidity": "<string>",
  "sponsorName": "<string>",
  "sponsorImage": "<string>",
  "startDate": "2023-11-07T05:31:56Z",
  "xAxisValue": "<string>",
  "yAxisValue": "<string>",
  "denominationToken": "<string>",
  "fee": "<string>",
  "image": "<string>",
  "icon": "<string>",
  "lowerBound": "<string>",
  "upperBound": "<string>",
  "description": "<string>",
  "outcomes": "<string>",
  "outcomePrices": "<string>",
  "volume": "<string>",
  "active": true,
  "marketType": "<string>",
  "formatType": "<string>",
  "lowerBoundDate": "<string>",
  "upperBoundDate": "<string>",
  "closed": true,
  "marketMakerAddress": "<string>",
  "createdBy": 123,
  "updatedBy": 123,
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "closedTime": "<string>",
  "wideFormat": true,
  "new": true,
  "mailchimpTag": "<string>",
  "featured": true,
  "archived": true,
  "resolvedBy": "<string>",
  "restricted": true,
  "marketGroup": 123,
  "groupItemTitle": "<string>",
  "groupItemThreshold": "<string>",
  "questionID": "<string>",
  "umaEndDate": "<string>",
  "enableOrderBook": true,
  "orderPriceMinTickSize": 123,
  "orderMinSize": 123,
  "umaResolutionStatus": "<string>",
  "curationOrder": 123,
  "volumeNum": 123,
  "liquidityNum": 123,
  "endDateIso": "<string>",
  "startDateIso": "<string>",
  "umaEndDateIso": "<string>",
  "hasReviewedDates": true,
  "readyForCron": true,
  "commentsEnabled": true,
  "volume24hr": 123,
  "volume1wk": 123,
  "volume1mo": 123,
  "volume1yr": 123,
  "gameStartTime": "<string>",
  "secondsDelay": 123,
  "clobTokenIds": "<string>",
  "disqusThread": "<string>",
  "shortOutcomes": "<string>",
  "teamAID": "<string>",
  "teamBID": "<string>",
  "umaBond": "<string>",
  "umaReward": "<string>",
  "fpmmLive": true,
  "volume24hrAmm": 123,
  "volume1wkAmm": 123,
  "volume1moAmm": 123,
  "volume1yrAmm": 123,
  "volume24hrClob": 123,
  "volume1wkClob": 123,
  "volume1moClob": 123,
  "volume1yrClob": 123,
  "volumeAmm": 123,
  "volumeClob": 123,
  "liquidityAmm": 123,
  "liquidityClob": 123,
  "makerBaseFee": 123,
  "takerBaseFee": 123,
  "customLiveness": 123,
  "acceptingOrders": true,
  "notificationsEnabled": true,
  "score": 123,
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
  "creator": "<string>",
  "ready": true,
  "funded": true,
  "pastSlugs": "<string>",
  "readyTimestamp": "2023-11-07T05:31:56Z",
  "fundedTimestamp": "2023-11-07T05:31:56Z",
  "acceptingOrdersTimestamp": "2023-11-07T05:31:56Z",
  "competitive": 123,
  "rewardsMinSize": 123,
  "rewardsMaxSpread": 123,
  "spread": 123,
  "automaticallyResolved": true,
  "oneDayPriceChange": 123,
  "oneHourPriceChange": 123,
  "oneWeekPriceChange": 123,
  "oneMonthPriceChange": 123,
  "oneYearPriceChange": 123,
  "lastTradePrice": 123,
  "bestBid": 123,
  "bestAsk": 123,
  "automaticallyActive": true,
  "clearBookOnStart": true,
  "chartColor": "<string>",
  "seriesColor": "<string>",
  "showGmpSeries": true,
  "showGmpOutcome": true,
  "manualActivation": true,
  "negRiskOther": true,
  "gameId": "<string>",
  "groupItemRange": "<string>",
  "sportsMarketType": "<string>",
  "line": 123,
  "umaResolutionStatuses": "<string>",
  "pendingDeployment": true,
  "deploying": true,
  "deployingTimestamp": "2023-11-07T05:31:56Z",
  "scheduledDeploymentTimestamp": "2023-11-07T05:31:56Z",
  "rfqEnabled": true,
  "eventStartTime": "2023-11-07T05:31:56Z"
}
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#parameter-id)

id

integer

required

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#parameter-include-tag)

include\_tag

boolean

#### Response

200

application/json

Market

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-id)

id

string

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-question-one-of-0)

question

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-condition-id)

conditionId

string

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-twitter-card-image-one-of-0)

twitterCardImage

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-resolution-source-one-of-0)

resolutionSource

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-end-date-one-of-0)

endDate

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-category-one-of-0)

category

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-amm-type-one-of-0)

ammType

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-liquidity-one-of-0)

liquidity

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-sponsor-name-one-of-0)

sponsorName

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-sponsor-image-one-of-0)

sponsorImage

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-start-date-one-of-0)

startDate

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-x-axis-value-one-of-0)

xAxisValue

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-y-axis-value-one-of-0)

yAxisValue

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-denomination-token-one-of-0)

denominationToken

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-fee-one-of-0)

fee

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-image-one-of-0)

image

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-icon-one-of-0)

icon

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-lower-bound-one-of-0)

lowerBound

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-upper-bound-one-of-0)

upperBound

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-description-one-of-0)

description

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-outcomes-one-of-0)

outcomes

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-outcome-prices-one-of-0)

outcomePrices

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume-one-of-0)

volume

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-active-one-of-0)

active

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-market-type-one-of-0)

marketType

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-format-type-one-of-0)

formatType

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-lower-bound-date-one-of-0)

lowerBoundDate

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-upper-bound-date-one-of-0)

upperBoundDate

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-closed-one-of-0)

closed

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-market-maker-address)

marketMakerAddress

string

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-created-by-one-of-0)

createdBy

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-updated-by-one-of-0)

updatedBy

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-closed-time-one-of-0)

closedTime

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-wide-format-one-of-0)

wideFormat

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-new-one-of-0)

new

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-mailchimp-tag-one-of-0)

mailchimpTag

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-featured-one-of-0)

featured

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-archived-one-of-0)

archived

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-resolved-by-one-of-0)

resolvedBy

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-restricted-one-of-0)

restricted

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-market-group-one-of-0)

marketGroup

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-group-item-title-one-of-0)

groupItemTitle

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-group-item-threshold-one-of-0)

groupItemThreshold

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-question-id-one-of-0)

questionID

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-end-date-one-of-0)

umaEndDate

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-enable-order-book-one-of-0)

enableOrderBook

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-order-price-min-tick-size-one-of-0)

orderPriceMinTickSize

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-order-min-size-one-of-0)

orderMinSize

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-resolution-status-one-of-0)

umaResolutionStatus

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-curation-order-one-of-0)

curationOrder

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume-num-one-of-0)

volumeNum

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-liquidity-num-one-of-0)

liquidityNum

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-end-date-iso-one-of-0)

endDateIso

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-start-date-iso-one-of-0)

startDateIso

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-end-date-iso-one-of-0)

umaEndDateIso

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-has-reviewed-dates-one-of-0)

hasReviewedDates

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-ready-for-cron-one-of-0)

readyForCron

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-comments-enabled-one-of-0)

commentsEnabled

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume24hr-one-of-0)

volume24hr

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1wk-one-of-0)

volume1wk

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1mo-one-of-0)

volume1mo

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1yr-one-of-0)

volume1yr

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-game-start-time-one-of-0)

gameStartTime

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-seconds-delay-one-of-0)

secondsDelay

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-clob-token-ids-one-of-0)

clobTokenIds

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-disqus-thread-one-of-0)

disqusThread

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-short-outcomes-one-of-0)

shortOutcomes

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-team-aid-one-of-0)

teamAID

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-team-bid-one-of-0)

teamBID

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-bond-one-of-0)

umaBond

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-reward-one-of-0)

umaReward

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-fpmm-live-one-of-0)

fpmmLive

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume24hr-amm-one-of-0)

volume24hrAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1wk-amm-one-of-0)

volume1wkAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1mo-amm-one-of-0)

volume1moAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1yr-amm-one-of-0)

volume1yrAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume24hr-clob-one-of-0)

volume24hrClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1wk-clob-one-of-0)

volume1wkClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1mo-clob-one-of-0)

volume1moClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume1yr-clob-one-of-0)

volume1yrClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume-amm-one-of-0)

volumeAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-volume-clob-one-of-0)

volumeClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-liquidity-amm-one-of-0)

liquidityAmm

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-liquidity-clob-one-of-0)

liquidityClob

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-maker-base-fee-one-of-0)

makerBaseFee

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-taker-base-fee-one-of-0)

takerBaseFee

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-custom-liveness-one-of-0)

customLiveness

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-accepting-orders-one-of-0)

acceptingOrders

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-notifications-enabled-one-of-0)

notificationsEnabled

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-score-one-of-0)

score

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-image-optimized)

imageOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-icon-optimized)

iconOptimized

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-events)

events

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-categories)

categories

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-tags)

tags

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-creator-one-of-0)

creator

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-ready-one-of-0)

ready

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-funded-one-of-0)

funded

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-past-slugs-one-of-0)

pastSlugs

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-ready-timestamp-one-of-0)

readyTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-funded-timestamp-one-of-0)

fundedTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-accepting-orders-timestamp-one-of-0)

acceptingOrdersTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-competitive-one-of-0)

competitive

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-rewards-min-size-one-of-0)

rewardsMinSize

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-rewards-max-spread-one-of-0)

rewardsMaxSpread

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-spread-one-of-0)

spread

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-automatically-resolved-one-of-0)

automaticallyResolved

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-one-day-price-change-one-of-0)

oneDayPriceChange

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-one-hour-price-change-one-of-0)

oneHourPriceChange

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-one-week-price-change-one-of-0)

oneWeekPriceChange

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-one-month-price-change-one-of-0)

oneMonthPriceChange

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-one-year-price-change-one-of-0)

oneYearPriceChange

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-last-trade-price-one-of-0)

lastTradePrice

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-best-bid-one-of-0)

bestBid

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-best-ask-one-of-0)

bestAsk

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-automatically-active-one-of-0)

automaticallyActive

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-clear-book-on-start-one-of-0)

clearBookOnStart

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-chart-color-one-of-0)

chartColor

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-series-color-one-of-0)

seriesColor

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-show-gmp-series-one-of-0)

showGmpSeries

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-show-gmp-outcome-one-of-0)

showGmpOutcome

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-manual-activation-one-of-0)

manualActivation

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-neg-risk-other-one-of-0)

negRiskOther

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-game-id-one-of-0)

gameId

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-group-item-range-one-of-0)

groupItemRange

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-sports-market-type-one-of-0)

sportsMarketType

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-line-one-of-0)

line

number \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-uma-resolution-statuses-one-of-0)

umaResolutionStatuses

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-pending-deployment-one-of-0)

pendingDeployment

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-deploying-one-of-0)

deploying

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-deploying-timestamp-one-of-0)

deployingTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-scheduled-deployment-timestamp-one-of-0)

scheduledDeploymentTimestamp

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-rfq-enabled-one-of-0)

rfqEnabled

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-by-id#response-event-start-time-one-of-0)

eventStartTime

string<date-time> \| null

[List markets](https://docs.polymarket.com/api-reference/markets/list-markets) [Get market tags by id](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.