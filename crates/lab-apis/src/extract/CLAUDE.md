# extract/ — Synthetic bootstrap service

`extract` is **not a real API client.** It's a synthetic service that scans local filesystems and SSH hosts for existing service credentials (config files, `.env` files, Docker Compose, etc.), parses them, and offers to write the discovered values into `~/.lab/.env`. It mirrors the real-service module shape so the CLI/MCP dispatchers treat it uniformly.

## Always compiled

Unlike feature-gated upstream integrations, `extract` has no feature flag — it's always on. It's the bootstrap: how a new user goes from zero to a populated `.env` in one command.

## Module layout

```
extract.rs              # entry point with META (Category::Bootstrap)
extract/
├── client.rs           # ExtractClient — scan(), apply()
├── types.rs            # ScanTarget, DiscoveredService, ServiceCreds, WritePlan
├── error.rs            # ExtractError
├── transport/          # LocalFs + Ssh (russh) abstraction
└── parsers/            # one parser per recognizable artifact
```

## Transport

- **`LocalFs`** — scans a local path.
- **`Ssh`** — uses `russh` + `russh-keys` (not shelling out to `ssh`). Supports URIs like `user@host:/etc/docker/compose` — the `host:/path` shape.
- Never shell out. Never use `tokio::process::Command` for ssh.

## `extract.apply` — `.env` write algorithm

The 8-rule algorithm below is **implemented** in `crates/lab/src/config/env_merge.rs` (lab-bg3e.3). Both `extract.apply` (when implemented) and `setup.draft.commit` delegate to that module — do not re-implement.

`extract.apply` is marked **`destructive: true`** because it mutates `~/.lab/.env`. The writer follows these 8 rules, no exceptions:

1. **Backup first.** Copy `.env` → `.env.bak.<timestamp>` before any write.
2. **Atomic write.** Write to `.env.tmp`, `fsync`, then `rename`.
3. **Preserve order** of existing keys.
4. **Preserve comments and blank lines.**
5. **Dedupe by key.** One entry per key in the output.
6. **Conflict policy.** Default: skip-and-warn on existing keys with different values. `--force` overwrites. `--dry-run` reports without writing.
7. **Quote values** containing whitespace, `#`, or shell metacharacters. Use double quotes with `\"` / `\\` escaping.
8. **Idempotence.** Re-running `apply` with the same inputs must be a no-op.

## Parsers

Each parser handles one artifact type (servarr `config.xml`, qBittorrent `qBittorrent.conf`, Docker Compose `.yml`, plain `.env`, etc.). Parsers return `Vec<ServiceCreds>`. Add new parsers by dropping a file in `parsers/` and registering it in `parsers.rs` — no dispatch wiring needed.

## Category

`Category::Bootstrap` — synthetic bootstrap/operator services. `extract` shares
this category with peers such as `deploy`, `doctor`, and `device_runtime`; do
not treat it as the only member.
