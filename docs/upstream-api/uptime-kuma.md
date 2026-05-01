# Uptime Kuma Source Contract

Retrieved: 2026-05-01

Sources:
- https://github.com/louislam/uptime-kuma/wiki/API-Documentation
- https://github.com/louislam/uptime-kuma

Uptime Kuma monitor data is primarily exposed through an internal Socket.IO API rather than a stable REST API. Lab treats this as a first-class Socket.IO integration. Side-effecting operations are exposed only through destructive Lab actions.

## Auth

Lab uses:

```env
UPTIME_KUMA_URL=http://localhost:3001
UPTIME_KUMA_USERNAME=admin
UPTIME_KUMA_PASSWORD=replace-me
```

`UPTIME_KUMA_PASSWORD` is secret. `server.health` checks the web UI root. Socket.IO actions log in with username/password and keep the resulting token in memory only.

## V1 Actions

| Action | Upstream mechanism | Status | Hosted posture |
|---|---|---|---|
| `contract.status` | local contract | implemented | safe |
| `server.health` | web-root HTTP probe | implemented | safe |
| `monitor.list` | `getMonitorList` Socket.IO ack | implemented | sensitive |
| `monitor.get` | `getMonitor` Socket.IO ack | implemented | sensitive |
| `monitor.create` | `add` Socket.IO ack | implemented, destructive | mutates upstream |
| `monitor.update` | `editMonitor` Socket.IO ack | implemented, destructive | mutates upstream |
| `monitor.delete` | `deleteMonitor` Socket.IO ack | implemented, destructive | mutates upstream |
| `monitor.pause` | `pauseMonitor` Socket.IO ack | implemented, destructive | mutates upstream |
| `monitor.resume` | `resumeMonitor` Socket.IO ack | implemented, destructive | mutates upstream |
| `heartbeat.list` | `getMonitorBeats` Socket.IO ack | implemented, 1-168h window | sensitive |
| `status.summary` | `getStats` Socket.IO ack | implemented | safe summary after redaction |
| `notification.list` | `getNotificationList` Socket.IO ack | implemented | sensitive |
| `notification.test` | `testNotification` Socket.IO ack | implemented, destructive | sends a real notification |
| `notification.add` | `addNotification` Socket.IO ack | implemented, destructive | mutates upstream |

## Security

Monitor payloads can include URLs, headers, notification targets, incident messages, and topology hints. Lab redacts sensitive keys from Socket.IO responses. Monitor mutations, notification tests, and notification creation are marked destructive. `notification.add` rejects template markers such as `{{`, `${`, `<%`, and `{%`, rejects URL userinfo, and rejects notification webhook/endpoint/url fields targeting localhost, RFC 1918, link-local, metadata, or other local/private addresses.
