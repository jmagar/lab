# Marketplace

`marketplace` is an always-on synthetic service that surfaces the Claude Code plugin marketplace system — known marketplaces, published plugins, installed plugins, and their shipped artifacts — through the same CLI + MCP + API dispatch path as every other `lab` service.

## Why It Exists

Claude Code stores marketplace metadata under `~/.claude/plugins/`. That layout is machine-readable but not programmatically exposed: there is no CLI that lists plugins across all configured marketplaces with their installed state, and no API that returns the artifact payload of an installed plugin. `marketplace` fills that gap.

It also wraps the destructive `claude plugin …` shell commands (`marketplace add`, `install`, `uninstall`) behind the same confirmation gate as the rest of `lab`: MCP elicitation, CLI `-y`/`--yes`, HTTP `"confirm": true`.

## Why It Is a Service

`marketplace` is not a remote API integration, but it follows the same shape as real services:

- pure types live in `lab-apis/src/marketplace/types.rs`
- action catalog, dispatch, and params live in `crates/lab/src/dispatch/marketplace/`
- CLI shim in `crates/lab/src/cli/marketplace.rs`
- MCP tool registered in `build_default_registry()`
- HTTP API route group at `/v1/marketplace`

This keeps plugin management inside the one-tool-per-service model and avoids special-case plumbing.

## Compilation Rule

`marketplace` is always compiled. Like `extract`, it is a bootstrap/operator capability, not a feature-gated optional integration.

## Responsibilities

`marketplace` owns:

- reading `~/.claude/plugins/known_marketplaces.json`
- reading each marketplace's `marketplace.json` manifest
- reading `~/.claude/plugins/installed_plugins.json`
- enriching plugin listings with `owner`, `desc`, `tags`, `category`, installed state, timestamps
- walking an installed plugin's install path and returning artifact contents (skills, agents, scripts, configs)
- shelling out to `claude plugin marketplace add`, `claude plugin install`, `claude plugin uninstall` for destructive actions

It does **not** own:

- fetching marketplace manifests over the network — that happens inside `claude plugin marketplace add`
- Claude Code's plugin resolution logic — lab reads the recorded state, it does not re-derive it

## Data Sources

| Path | Purpose |
| --- | --- |
| `~/.claude/plugins/known_marketplaces.json` | Marketplace registry (id → source, autoUpdate, totalPlugins, lastUpdated, installLocation) |
| `<installLocation>/.claude-plugin/marketplace.json` | Manifest: `name`, `owner.name`, `metadata.description`, `plugins[]` |
| `<installLocation>/marketplace.json` | Fallback manifest location |
| `~/.claude/plugins/installed_plugins.json` | Installed state by `name@marketplace` → `installPath`, `installedAt`, `lastUpdated` |
| `<installPath>/**` | Plugin artifacts (returned by `plugin.artifacts`) |

Missing files are treated as empty — a fresh Claude Code install returns zero marketplaces without error.

## Actions

The complete marketplace action inventory is generated from `ActionSpec`:

- [generated action catalog](./generated/action-catalog.md)
- [generated action catalog JSON](./generated/action-catalog.json)

### Parameters

- `sources.add` — pass exactly one of `repo` (`owner/repo` slug) or `url` (git URL). Passing both returns `invalid_param`; passing neither returns `missing_param`.
- `plugins.list` — optional `marketplace`, `kind`, `installed`, and `query` filters. Filters are additive.
- `plugin.get`, `plugin.artifacts`, `plugin.workspace`, `plugin.deploy.preview`, `plugin.deploy`, `plugin.install`, `plugin.uninstall` — require `id` in `name@marketplace` form. Malformed ids return `invalid_param`.
- `plugin.save` — requires `id`, `path`, and `content`. `path` is always relative to the plugin workspace mirror.

## Return Shapes

### `Marketplace` (from `sources.list`)

```json
{
  "id": "jmagar-lab",
  "name": "jmagar-lab",
  "owner": "Jacob Magar",
  "ghUser": "jmagar",
  "repo": "jmagar/lab-plugins",
  "source": "github",
  "url": null,
  "path": null,
  "desc": "Homelab control plane — skills and tooling for lab",
  "autoUpdate": true,
  "totalPlugins": 9,
  "lastUpdated": "2026-04-21T19:13:23.871Z"
}
```

### `Plugin` (from `plugins.list`, `plugin.get`)

```json
{
  "id": "aurora-design@jmagar-lab",
  "name": "aurora-design",
  "mkt": "jmagar-lab",
  "ver": "0.3.1",
  "desc": "Aurora design system primitives",
  "tags": ["ui", "design-system", "shadcn"],
  "installed": true,
  "hasUpdate": null,
  "installedAt": "2026-04-10T12:04:11Z",
  "updatedAt": "2026-04-19T08:22:40Z"
}
```

`tags` is a deduplicated merge of the plugin's `category` (prepended when present), `tags`, and `keywords` arrays from `marketplace.json`.

### `Artifact` (from `plugin.artifacts`)

```json
{
  "path": "skills/foo/SKILL.md",
  "lang": "markdown",
  "content": "# Foo skill\n..."
}
```

`lang` is inferred from the file extension: `json`, `yaml`, `markdown`, `bash`, `toml`, or `text`. The walker:

- skips dotfiles, `node_modules`, and `target`
- skips files larger than 256 KiB
- caps output at 200 files per plugin
- returns paths relative to the plugin's install path

### `PluginWorkspace` (from `plugin.workspace`)

```json
{
  "pluginId": "aurora-design@jmagar-lab",
  "workspaceRoot": "/Users/jmagar/.lab/stash/plugins/aurora-design-jmagar-lab",
  "deployTarget": "/Users/jmagar/.claude/skills/aurora-design",
  "hasDirtyFiles": false,
  "files": [
    {
      "path": "SKILL.md",
      "lang": "markdown",
      "content": "---\nname: aurora-design\n..."
    }
  ]
}
```

The workspace mirror is created under `<workspace.root>/plugins/` on first load from the marketplace source tree and preserved separately from the installed Claude Code target. By default `workspace.root` is `~/.lab/stash`, and it also backs the read-only attachment picker. Legacy mirrors under `~/.claude/plugins/workspaces/` are migrated to `<workspace.root>/plugins/` on first access when the new mirror does not already exist.

### `SaveResult` (from `plugin.save`)

```json
{
  "pluginId": "aurora-design@jmagar-lab",
  "path": "SKILL.md",
  "savedAt": "2026-04-23T03:11:29.000Z"
}
```

### `DeployPreviewResult` (from `plugin.deploy.preview`)

```json
{
  "pluginId": "aurora-design@jmagar-lab",
  "targetPath": "/Users/jmagar/.claude/skills/aurora-design",
  "changed": ["SKILL.md"],
  "skipped": [],
  "removed": ["old.txt"],
  "failed": []
}
```

### `DeployResult` (from `plugin.deploy`)

```json
{
  "pluginId": "aurora-design@jmagar-lab",
  "targetPath": "/Users/jmagar/.claude/skills/aurora-design",
  "changed": ["SKILL.md"],
  "skipped": [],
  "removed": ["old.txt"],
  "failed": []
}
```

### Shell wrappers (`sources.add`, `plugin.install`, `plugin.uninstall`)

```json
{ "ok": true, "id": "foo@bar", "stdout": "...", "stderr": "..." }
```

A non-zero exit from `claude plugin …` becomes a `server_error` envelope with the captured `stderr` in the message.

## Error Envelopes

`marketplace` returns the canonical `ToolError` kinds from `docs/ERRORS.md`:

| Kind | When |
| --- | --- |
| `unknown_action` | Action name not in the catalog |
| `missing_param` | Required param absent (e.g., `id` for `plugin.get`) |
| `invalid_param` | Malformed input (e.g., `id` without `@`, both `repo` and `url` supplied) |
| `not_found` | Plugin id not present in any manifest, or `plugin.artifacts` called on an uninstalled plugin |
| `decode_error` | JSON parse failure on a plugin registry file |
| `internal_error` | `HOME` unset, filesystem I/O error, `claude` binary not spawnable |
| `server_error` | `claude plugin …` exited non-zero |
| `confirmation_required` | Destructive action without `confirm: true` / `-y` / elicitation accept |

## CLI Surface

```bash
lab marketplace sources.list                    # default human table
lab marketplace sources.list --json             # machine-readable
lab marketplace plugins.list --params '{"marketplace":"jmagar-lab"}'
lab marketplace plugin.get   --params '{"id":"aurora-design@jmagar-lab"}'
lab marketplace plugin.artifacts --params '{"id":"aurora-design@jmagar-lab"}'

lab marketplace sources.add      --params '{"repo":"obra/superpowers-marketplace"}' -y
lab marketplace plugin.install   --params '{"id":"aurora-design@jmagar-lab"}' -y
lab marketplace plugin.uninstall --params '{"id":"aurora-design@jmagar-lab"}' -y
```

Destructive actions require `-y` / `--yes` (or `--no-confirm`) to run non-interactively. `--dry-run` prints what would be dispatched without touching `~/.claude/`.

## MCP Surface

One tool, `marketplace`, accepting the standard `{ action, params }` envelope:

```jsonc
marketplace({ "action": "sources.list" })
marketplace({ "action": "plugins.list", "params": { "marketplace": "jmagar-lab" } })
marketplace({ "action": "plugin.get",   "params": { "id": "aurora-design@jmagar-lab" } })

// destructive — clients with elicitation get a confirm prompt;
// clients without elicitation must pass confirm: true explicitly.
marketplace({ "action": "plugin.install",
              "params": { "id": "aurora-design@jmagar-lab", "confirm": true } })
```

Discovery resources:

- `lab://marketplace/actions` — action catalog
- `lab://catalog` — includes `marketplace` alongside every other service

## HTTP API Surface

Route group: `/v1/marketplace` (bearer auth required).

```bash
# List marketplaces
curl -s -X POST http://127.0.0.1:8765/v1/marketplace \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"sources.list"}'

# Install a plugin (destructive — must confirm)
curl -s -X POST http://127.0.0.1:8765/v1/marketplace \
  -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"plugin.install","params":{"id":"aurora-design@jmagar-lab","confirm":true}}'
```

Status mapping follows `docs/ERRORS.md`: `not_found` → 404, `invalid_param`/`missing_param` → 422, `confirmation_required` → 400, `server_error` → 502.

## Web UI

The gateway admin UI at `/marketplace` consumes the HTTP API and presents:

- the nine (or however many are configured) marketplaces as cards with owner, plugin count, and description
- every plugin across every marketplace with installed badges
- an "Add marketplace" flow that calls `sources.add`
- a dedicated plugin detail route at `/marketplace/plugin?id=<pluginId>`
- an editable `Files` tab backed by a workspace mirror, explicit `Save`, deploy preview, and explicit `Deploy`
- install / uninstall buttons gated behind the shared destructive confirmation flow

The frontend never talks to `~/.claude/` directly — every read and write goes through `/v1/marketplace`.

Deploy preview and deploy use a cheap-first sync model:
- file metadata is checked before file contents are read
- unchanged files are short-circuited without a full content read
- only potentially changed files are compared more deeply
- this keeps large plugin workspaces responsive while preserving explicit `Save` and `Deploy` semantics

## Safety

- No action reads or writes anywhere outside `~/.claude/plugins/` for the observational commands.
- Destructive actions delegate to the `claude` CLI binary; `lab` captures stdout/stderr but does not modify the plugin registry itself.
- `plugin.artifacts` enforces a 256 KiB per-file cap and 200-file ceiling to keep responses bounded — large plugins return a truncated artifact list rather than a multi-MB payload.
- Spawning the `claude` binary happens via `tokio::process::Command` with explicit argv (no shell interpolation); plugin ids and repo slugs are never passed through a shell.
