# Uptime Kuma Source Contract

Retrieved: 2026-05-01

Sources:
- https://github.com/louislam/uptime-kuma/wiki/API-Documentation
- https://github.com/louislam/uptime-kuma

Uptime Kuma monitor data is primarily exposed through an internal Socket.IO API rather than a stable REST API. Lab treats this as a first-class read-oriented integration with side-effecting operations deferred.

## Auth

Lab uses:

```env
UPTIME_KUMA_URL=http://localhost:3001
UPTIME_KUMA_USERNAME=admin
UPTIME_KUMA_PASSWORD=replace-me
```

`UPTIME_KUMA_PASSWORD` is secret. `server.health` checks the web UI root. Socket.IO read actions log in with username/password and keep the resulting token in memory only.

## V1 Actions

| Action | Upstream mechanism | Status | Hosted posture |
|---|---|---|---|
| `contract.status` | local contract | implemented | safe |
| `server.health` | web-root HTTP probe | implemented | safe |
| `monitor.list` | `getMonitorList` Socket.IO ack | implemented | sensitive |
| `monitor.get` | `getMonitor` Socket.IO ack | implemented | sensitive |
| `heartbeat.list` | `getMonitorBeats` Socket.IO ack | implemented, 1-168h window | sensitive |
| `status.summary` | `getStats` Socket.IO ack | implemented | safe summary after redaction |
| `notification.list` | `getNotificationList` Socket.IO ack | implemented | sensitive |

## Security

Monitor payloads can include URLs, headers, notification targets, incident messages, and topology hints. Lab therefore exposes read actions only and does not expose monitor mutation, notification creation, or notification test sends in the default catalog.
