# Uptime Kuma Source Contract

Retrieved: 2026-05-01

Sources:
- https://github.com/louislam/uptime-kuma/wiki/API-Documentation
- https://github.com/louislam/uptime-kuma

Uptime Kuma monitor data is primarily exposed through an internal Socket.IO API rather than a stable REST API. Lab treats this as a first-class but deferred live-read integration: metadata, env, registry, dispatch, docs, and health are wired now, while live monitor reads require a bounded socket actor.

## Auth

Lab uses:

```env
UPTIME_KUMA_URL=http://localhost:3001
UPTIME_KUMA_USERNAME=admin
UPTIME_KUMA_PASSWORD=replace-me
```

`UPTIME_KUMA_PASSWORD` is secret. The current health probe checks the web UI root without logging in.

## V1 Actions

| Action | Upstream mechanism | Status | Hosted posture |
|---|---|---|---|
| `contract.status` | local contract | implemented | safe |
| `server.health` | web-root HTTP probe | implemented | safe |
| `monitor.list` | Socket.IO | deferred | sensitive |
| `monitor.get` | Socket.IO | deferred | sensitive |
| `heartbeat.list` | Socket.IO | deferred | sensitive |
| `status.summary` | Socket.IO | deferred | safe summary after redaction |
| `notification.list` | Socket.IO | deferred | sensitive |

## Security

Monitor payloads can include URLs, headers, notification targets, incident messages, and topology hints. The live actor must enforce bounded reads, redaction, and no mutation by default before these actions are allowed to return upstream data.
