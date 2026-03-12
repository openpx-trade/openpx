[Skip to main content](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Comments

Get comments by user address

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get comments by user address

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/comments/user_address/{user_address}
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "body": "<string>",\
    "parentEntityType": "<string>",\
    "parentEntityID": 123,\
    "parentCommentID": "<string>",\
    "userAddress": "<string>",\
    "replyAddress": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "profile": {\
      "name": "<string>",\
      "pseudonym": "<string>",\
      "displayUsernamePublic": true,\
      "bio": "<string>",\
      "isMod": true,\
      "isCreator": true,\
      "proxyWallet": "<string>",\
      "baseAddress": "<string>",\
      "profileImage": "<string>",\
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
      "positions": [\
        {\
          "tokenId": "<string>",\
          "positionSize": "<string>"\
        }\
      ]\
    },\
    "reactions": [\
      {\
        "id": "<string>",\
        "commentID": 123,\
        "reactionType": "<string>",\
        "icon": "<string>",\
        "userAddress": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "profile": {\
          "name": "<string>",\
          "pseudonym": "<string>",\
          "displayUsernamePublic": true,\
          "bio": "<string>",\
          "isMod": true,\
          "isCreator": true,\
          "proxyWallet": "<string>",\
          "baseAddress": "<string>",\
          "profileImage": "<string>",\
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
          "positions": [\
            {\
              "tokenId": "<string>",\
              "positionSize": "<string>"\
            }\
          ]\
        }\
      }\
    ],\
    "reportCount": 123,\
    "reactionCount": 123\
  }\
]
```

GET

/

comments

/

user\_address

/

{user\_address}

Try it

Get comments by user address

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/comments/user_address/{user_address}
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "body": "<string>",\
    "parentEntityType": "<string>",\
    "parentEntityID": 123,\
    "parentCommentID": "<string>",\
    "userAddress": "<string>",\
    "replyAddress": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "profile": {\
      "name": "<string>",\
      "pseudonym": "<string>",\
      "displayUsernamePublic": true,\
      "bio": "<string>",\
      "isMod": true,\
      "isCreator": true,\
      "proxyWallet": "<string>",\
      "baseAddress": "<string>",\
      "profileImage": "<string>",\
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
      "positions": [\
        {\
          "tokenId": "<string>",\
          "positionSize": "<string>"\
        }\
      ]\
    },\
    "reactions": [\
      {\
        "id": "<string>",\
        "commentID": 123,\
        "reactionType": "<string>",\
        "icon": "<string>",\
        "userAddress": "<string>",\
        "createdAt": "2023-11-07T05:31:56Z",\
        "profile": {\
          "name": "<string>",\
          "pseudonym": "<string>",\
          "displayUsernamePublic": true,\
          "bio": "<string>",\
          "isMod": true,\
          "isCreator": true,\
          "proxyWallet": "<string>",\
          "baseAddress": "<string>",\
          "profileImage": "<string>",\
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
          "positions": [\
            {\
              "tokenId": "<string>",\
              "positionSize": "<string>"\
            }\
          ]\
        }\
      }\
    ],\
    "reportCount": 123,\
    "reactionCount": 123\
  }\
]
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#parameter-user-address)

user\_address

string

required

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#parameter-limit)

limit

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#parameter-offset)

offset

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#parameter-order)

order

string

Comma-separated list of fields to order by

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#parameter-ascending)

ascending

boolean

#### Response

200 - application/json

Comments

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-id)

id

string

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-body-one-of-0)

body

string \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-parent-entity-type-one-of-0)

parentEntityType

string \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-parent-entity-id-one-of-0)

parentEntityID

integer \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-parent-comment-id-one-of-0)

parentCommentID

string \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-user-address-one-of-0)

userAddress

string \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-reply-address-one-of-0)

replyAddress

string \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-profile)

profile

object

Showchild attributes

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-reactions)

reactions

object\[\]

Showchild attributes

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-report-count-one-of-0)

reportCount

integer \| null

[​](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address#response-items-reaction-count-one-of-0)

reactionCount

integer \| null

[Get comments by comment id](https://docs.polymarket.com/api-reference/comments/get-comments-by-comment-id) [Get public profile by wallet address](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.