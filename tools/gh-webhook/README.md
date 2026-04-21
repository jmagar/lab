# gh-webhook

Receives GitHub webhooks, debounces PR comment bursts, appends short pings to a JSONL file that a Claude Code monitor tails. Replaces polling in the `gh-address-comments` skill.

## Layout

- One axum server bound to `127.0.0.1:7891`
- Published via Tailscale Funnel at `https://<host>.ts.net/gh-webhook`
- Writes to `$GH_WEBHOOK_DATA_DIR` (default `~/.gh-webhook`)
- Logs to stderr (JSON when `GH_WEBHOOK_LOG_FORMAT=json`)

## Env

| Var | Required | Purpose |
|-----|----------|---------|
| `GH_WEBHOOK_SECRET` | yes | HMAC-SHA256 shared secret |
| `GH_WEBHOOK_GITHUB_TOKEN` | yes | Fine-grained PAT (Metadata:Read, PR:Read, Issues:Read) |
| `GH_WEBHOOK_BIND` | no | Default `127.0.0.1:7891` |
| `GH_WEBHOOK_DATA_DIR` | no | Default `$HOME/.gh-webhook` |
| `GH_WEBHOOK_DEBOUNCE_SECS` | no | Default `30` |
| `GH_WEBHOOK_LOG_FORMAT` | no | `json` or unset |

See `systemd/gh-webhook.service` for the hardened unit.
