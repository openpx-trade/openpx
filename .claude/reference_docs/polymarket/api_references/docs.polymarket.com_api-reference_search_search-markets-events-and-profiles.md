[Skip to main content](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Search

Search markets, events, and profiles

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Search markets, events, and profiles

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/public-search
```

200

Copy

Ask AI

```
{
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
  ],
  "tags": [\
    {\
      "id": "<string>",\
      "label": "<string>",\
      "slug": "<string>",\
      "event_count": 123\
    }\
  ],
  "profiles": [\
    {\
      "id": "<string>",\
      "name": "<string>",\
      "user": 123,\
      "referral": "<string>",\
      "createdBy": 123,\
      "updatedBy": 123,\
      "createdAt": "2023-11-07T05:31:56Z",\
      "updatedAt": "2023-11-07T05:31:56Z",\
      "utmSource": "<string>",\
      "utmMedium": "<string>",\
      "utmCampaign": "<string>",\
      "utmContent": "<string>",\
      "utmTerm": "<string>",\
      "walletActivated": true,\
      "pseudonym": "<string>",\
      "displayUsernamePublic": true,\
      "profileImage": "<string>",\
      "bio": "<string>",\
      "proxyWallet": "<string>",\
      "profileImageOptimized": {\
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
      "isCloseOnly": true,\
      "isCertReq": true,\
      "certReqDate": "2023-11-07T05:31:56Z"\
    }\
  ],
  "pagination": {
    "hasMore": true,
    "totalResults": 123
  }
}
```

GET

/

public-search

Try it

Search markets, events, and profiles

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/public-search
```

200

Copy

Ask AI

```
{
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
  ],
  "tags": [\
    {\
      "id": "<string>",\
      "label": "<string>",\
      "slug": "<string>",\
      "event_count": 123\
    }\
  ],
  "profiles": [\
    {\
      "id": "<string>",\
      "name": "<string>",\
      "user": 123,\
      "referral": "<string>",\
      "createdBy": 123,\
      "updatedBy": 123,\
      "createdAt": "2023-11-07T05:31:56Z",\
      "updatedAt": "2023-11-07T05:31:56Z",\
      "utmSource": "<string>",\
      "utmMedium": "<string>",\
      "utmCampaign": "<string>",\
      "utmContent": "<string>",\
      "utmTerm": "<string>",\
      "walletActivated": true,\
      "pseudonym": "<string>",\
      "displayUsernamePublic": true,\
      "profileImage": "<string>",\
      "bio": "<string>",\
      "proxyWallet": "<string>",\
      "profileImageOptimized": {\
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
      "isCloseOnly": true,\
      "isCertReq": true,\
      "certReqDate": "2023-11-07T05:31:56Z"\
    }\
  ],
  "pagination": {
    "hasMore": true,
    "totalResults": 123
  }
}
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-q)

q

string

required

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-cache)

cache

boolean

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-events-status)

events\_status

string

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-limit-per-type)

limit\_per\_type

integer

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-page)

page

integer

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-events-tag)

events\_tag

string\[\]

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-keep-closed-markets)

keep\_closed\_markets

integer

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-sort)

sort

string

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-ascending)

ascending

boolean

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-search-tags)

search\_tags

boolean

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-search-profiles)

search\_profiles

boolean

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-recurrence)

recurrence

string

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-exclude-tag-id)

exclude\_tag\_id

integer\[\]

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#parameter-optimized)

optimized

boolean

#### Response

200 - application/json

Search results

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#response-events-one-of-0)

events

object\[\] \| null

Showchild attributes

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#response-tags-one-of-0)

tags

object\[\] \| null

Showchild attributes

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#response-profiles-one-of-0)

profiles

object\[\] \| null

Showchild attributes

[​](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles#response-pagination)

pagination

object

Showchild attributes

[Get public profile by wallet address](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address) [Data API Health check](https://docs.polymarket.com/api-reference/data-api-status/data-api-health-check)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.