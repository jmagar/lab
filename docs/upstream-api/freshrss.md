# FreshRSS Source Contract

Retrieved: 2026-05-01

Sources:
- https://freshrss.github.io/FreshRSS/en/developers/06_GoogleReader_API.html
- https://freshrss.github.io/FreshRSS/en/developers/06_Fever_API.html
- https://freshrss.github.io/FreshRSS/en/users/06_Mobile_access.html

FreshRSS exposes Google Reader-compatible and Fever-compatible APIs. Lab v1 uses Google Reader-compatible endpoints under `api/greader.php` and defers Fever.

## Auth

Lab requires `FRESHRSS_URL`, `FRESHRSS_USERNAME`, and `FRESHRSS_API_PASSWORD`. `accounts/ClientLogin` is an internal helper and its `Auth` token is never returned to MCP/API callers.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `subscription.list` | `/reader/api/0/subscription/list` | upstream bounded | safe |
| `tag.list` | `/reader/api/0/tag/list` | upstream bounded | safe |
| `unread.counts` | `/reader/api/0/unread-count` | upstream bounded | safe |
| `stream.items` | `/reader/api/0/stream/contents/reading-list` | required `n`, max 100, continuation returned by upstream | feed/article data |

## Deferred

Write tokens, read-state mutation, starring, subscription mutation, tag mutation, refresh triggers, Fever, and public `session.create` are deferred.

