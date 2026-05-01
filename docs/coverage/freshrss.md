# FreshRSS Coverage

Status: first-class v1 read-only Google Reader-compatible surface.

Actions: `subscription.list`, `tag.list`, `unread.counts`, `stream.items`, plus built-in `help` and `schema`.

Deferred: Fever API, public session creation, read-state mutation, starring, subscription/tag mutation, feed refresh triggers, write-token operations.

Security: `FRESHRSS_API_PASSWORD` and GoogleLogin tokens are secret. `stream.items` requires `n` and clamps to max 100.

