[Skip to main content](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Profiles

Get public profile by wallet address

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get public profile by wallet address

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/public-profile
```

200

400

404

Copy

Ask AI

```
{
  "createdAt": "2023-11-07T05:31:56Z",
  "proxyWallet": "<string>",
  "profileImage": "<string>",
  "displayUsernamePublic": true,
  "bio": "<string>",
  "pseudonym": "<string>",
  "name": "<string>",
  "users": [\
    {\
      "id": "<string>",\
      "creator": true,\
      "mod": true\
    }\
  ],
  "xUsername": "<string>",
  "verifiedBadge": true
}
```

GET

/

public-profile

Try it

Get public profile by wallet address

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/public-profile
```

200

400

404

Copy

Ask AI

```
{
  "createdAt": "2023-11-07T05:31:56Z",
  "proxyWallet": "<string>",
  "profileImage": "<string>",
  "displayUsernamePublic": true,
  "bio": "<string>",
  "pseudonym": "<string>",
  "name": "<string>",
  "users": [\
    {\
      "id": "<string>",\
      "creator": true,\
      "mod": true\
    }\
  ],
  "xUsername": "<string>",
  "verifiedBadge": true
}
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#parameter-address)

address

string

required

The wallet address (proxy wallet or user address)

#### Response

200

application/json

Public profile information

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-created-at-one-of-0)

createdAt

string<date-time> \| null

ISO 8601 timestamp of when the profile was created

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-proxy-wallet-one-of-0)

proxyWallet

string \| null

The proxy wallet address

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-profile-image-one-of-0)

profileImage

string<uri> \| null

URL to the profile image

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-display-username-public-one-of-0)

displayUsernamePublic

boolean \| null

Whether the username is displayed publicly

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-bio-one-of-0)

bio

string \| null

Profile bio

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-pseudonym-one-of-0)

pseudonym

string \| null

Auto-generated pseudonym

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-name-one-of-0)

name

string \| null

User-chosen display name

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-users-one-of-0)

users

object\[\] \| null

Array of associated user objects

Showchild attributes

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-x-username-one-of-0)

xUsername

string \| null

X (Twitter) username

[​](https://docs.polymarket.com/api-reference/profiles/get-public-profile-by-wallet-address#response-verified-badge-one-of-0)

verifiedBadge

boolean \| null

Whether the profile has a verified badge

[Get comments by user address](https://docs.polymarket.com/api-reference/comments/get-comments-by-user-address) [Search markets, events, and profiles](https://docs.polymarket.com/api-reference/search/search-markets-events-and-profiles)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.