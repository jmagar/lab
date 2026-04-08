# tui/ — Ratatui plugin manager

The TUI is the **discovery and installation UX** for `lab`. CLI is for automation, TUI is for humans browsing what's available and flipping services on/off.

## Responsibilities

- Enumerate all compiled-in services by reading `PluginMeta` constants.
- Group by `Category` (Media, Servarr, Indexer, Download, Notes, Documents, Network, Notifications, Ai, Bootstrap).
- Show env var requirements (`required_env` / `optional_env`), masking values where `EnvVar.secret == true`.
- Install/uninstall services into the user's `.mcp.json` — adding or removing entries in the `--services` array.
- Surface `lab doctor` results inline (per-service health dots).

## Metadata aggregation

`metadata.rs` collects every service's `pub const META: PluginMeta` into a static list at compile time. Adding a new service means updating `metadata.rs` to include its module — **not** editing the TUI render loop. The render code is data-driven over the metadata list.

## `.mcp.json` patching

The TUI is the only part of `lab` that writes `.mcp.json`. Rules:

1. **Backup first.** Copy `.mcp.json` → `.mcp.json.bak.<timestamp>`.
2. **Atomic write.** Write to temp, rename.
3. **Preserve unrelated keys.** Parse, mutate only the `lab` entry's `--services` array, serialize back.
4. **Dedupe** the services array on write.
5. **Refuse to write** if the file has invalid JSON — don't silently overwrite user edits.

## TUI vs CLI divide

| Task | Tool |
|------|------|
| "What services are available?" | TUI |
| "Enable radarr with env vars prompted" | TUI |
| "Run `radarr movie.search matrix` in a script" | CLI |
| "CI health check of all enabled services" | `lab doctor --json` |

If you're adding a flow that automates something, it belongs in the CLI. If you're adding a browse/pick/configure flow, it belongs here.

## Secrets handling

`EnvVar.secret == true` values must be masked in all render paths: the list view, the detail pane, and any copy-to-clipboard action. Reveal requires an explicit keypress (`r`) and a banner.
