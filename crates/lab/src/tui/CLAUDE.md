# tui/ — Ratatui plugin manager

The TUI is the **discovery, configuration, and installation UX** for `lab`. CLI is for automation, TUI is for humans browsing what's available and flipping things on/off.

## Current State

`run()` is a stub — the TUI is not yet functional end-to-end. `metadata.rs` currently only has `radarr` wired. The architecture below describes the intended design; implementation is in progress.

## Two tabs

The TUI has two top-level tabs (see `docs/TUI.md` for full spec):

| Tab | Responsibility |
|-----|---------------|
| **Services** | Browse compiled-in `lab` services, configure env vars, toggle `.mcp.json` wiring, surface health dots |
| **Plugins** | Browse/add marketplaces (Claude Code, Codex) and Gemini CLI extensions; install/remove plugins |

## Services tab — implementation notes

- Enumerate all compiled-in services by reading `PluginMeta` constants via `metadata.rs`.
- Group by `Category` (Media, Servarr, Indexer, Download, Notes, Documents, Network, Notifications, Ai, Bootstrap).
- Show env var requirements (`required_env` / `optional_env`), masking values where `EnvVar.secret == true`.
- Toggle enabled/disabled by writing the `lab` entry's `--services` array in `.mcp.json`.
- Surface `lab doctor` results inline (per-service health dots).

## Plugins tab — implementation notes

- Fetch marketplace catalogs from local files (already-added) or remotely via raw GitHub URLs (preview).
- Support Claude Code (`claude plugin` CLI), Codex (file-based), and Gemini CLI (`gemini extensions` CLI).
- Never invent install logic for Claude Code or Gemini CLI — delegate to their CLIs.

## Metadata aggregation

`metadata.rs` collects every service's `pub const META: PluginMeta` into a static list at compile time. Adding a new service means updating `metadata.rs` — **not** editing the TUI render loop. The render code is data-driven over the metadata list.

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
| "What services are available?" | TUI — Services tab |
| "Enable radarr, set env vars" | TUI — Services tab |
| "Add a Claude Code marketplace" | TUI — Plugins tab |
| "Run `radarr movie.search matrix` in a script" | CLI |
| "CI health check of all enabled services" | `lab doctor --json` |

If you're adding a flow that automates something, it belongs in the CLI. If you're adding a browse/pick/configure flow, it belongs here.

## Secrets handling

`EnvVar.secret == true` values must be masked in all render paths: the list view, the detail pane, and any copy-to-clipboard action. Reveal requires an explicit keypress (`r`) and a banner.
