# Plugin / Marketplace / Extension Matrix — Claude / Codex / Gemini

Authoritative manifest schemas, file layouts, and validation rules for **building** distributable bundles across the three platforms. Sibling to `platform-matrix.md` (which covers loose-component parity) — this file covers **packaged distribution**.

When you need deeper detail than this matrix, jump to the per-platform reference doc cited in each section.

## Terminology map

The platforms call the same concept different things — keep this straight so you don't confuse the user:

| Concept | Claude | Codex | Gemini |
|---|---|---|---|
| Bundle of capabilities | "plugin" | "plugin" | "extension" |
| Catalog/distribution | "marketplace" | "marketplace" | (none — install from GitHub URL) |
| Per-bundle manifest | `.claude-plugin/plugin.json` | `.codex-plugin/plugin.json` | `gemini-extension.json` (at repo root) |
| Catalog manifest | `.claude-plugin/marketplace.json` | `.agents/plugins/marketplace.json` | (n/a) |
| User install command | `/plugin install <name>@<marketplace>` | `codex plugin install <name>@<marketplace>` | `gemini extensions install <github-url>` |

A single repo can be a Claude marketplace AND a Codex marketplace AND a Gemini extension at the same time — three manifests at three paths, no conflict.

---

## §1 — Claude plugin manifest (`.claude-plugin/plugin.json`)

Lives inside each plugin directory.

```json
{
  "name": "plugin-name",
  "version": "1.2.0",
  "description": "Brief plugin description",
  "author": {
    "name": "Author Name",
    "email": "author@example.com",
    "url": "https://github.com/author"
  },
  "homepage": "https://docs.example.com",
  "repository": "https://github.com/user/plugin",
  "license": "MIT",

  "skills":       "./skills/",
  "commands":     ["./commands/special.md"],
  "agents":       "./agents/",
  "hooks":        "./hooks.json",
  "mcpServers":   "./mcp.json",
  "outputStyles": "./styles/",
  "themes":       "./themes/",
  "lspServers":   "./.lsp.json",

  "dependencies": [
    "helper-lib",
    { "name": "secrets-vault", "version": "~2.1.0" }
  ],
  "userConfig": { "...": "see plugins-reference.md" },
  "channels":   [ "...see plugins-reference.md..." ]
}
```

| Field | Required | Notes |
|---|---|---|
| `name` | yes | identifier; must be unique within a marketplace |
| `version` | recommended | semver. If omitted on a git-hosted plugin, every commit is treated as a new version |
| `description` | yes | one-line for marketplace UI |
| `author` | yes (object form) | `{ name, email?, url? }` |
| component paths (`skills`, `commands`, `agents`, `hooks`, `mcpServers`, etc.) | optional | relative paths inside the plugin |
| `dependencies` | optional | other plugins this requires; supports semver |
| `userConfig` | optional | values Claude prompts for at install time — see `claude/plugins-reference.md` |

**Path convention:** all component paths are **relative to the plugin directory**, not the marketplace root. Use `${CLAUDE_PLUGIN_ROOT}` inside scripts to refer to the plugin root at runtime.

**Components with asymmetric platform support:**
- **`outputStyles`, `lspServers`, `channels`** — Claude only. No Codex or Gemini equivalent.
- **`themes`** — Claude AND Gemini both support themes; only Codex has no equivalent. See §5 for Gemini's `themes` array shape, which differs from Claude's (Gemini themes are inline JSON objects defining color tokens, not a separate themes/ directory).

Reference: `claude/plugins-reference.md`, `claude/create-plugins.md`.

## §2 — Claude marketplace manifest (`.claude-plugin/marketplace.json`)

Lives at the root of the marketplace repo, in `.claude-plugin/`.

```json
{
  "name": "company-tools",
  "owner": {
    "name": "DevTools Team",
    "email": "devtools@example.com"
  },
  "plugins": [
    {
      "name": "quality-review-plugin",
      "source": "./plugins/quality-review-plugin",
      "description": "Adds a /quality-review skill for quick code reviews",
      "version": "1.0.0"
    }
  ]
}
```

| Field | Required | Notes |
|---|---|---|
| `name` | yes | marketplace identifier |
| `owner` | yes | `{ name, email? }` |
| `plugins[]` | yes | array; each entry needs at least `name` + `source` |
| `plugins[].source` | yes | `./relative/path` for in-repo plugins, or git URL |
| `plugins[].version` | optional | overrides per-plugin manifest version |

**Plugin caching gotcha:** when users install, Claude copies the plugin directory to a cache. Plugins **cannot** reference files outside their directory (`../shared-utils` won't work). Use symlinks if you need cross-plugin sharing — see `claude/plugins-reference.md § Plugin caching and file resolution`.

Reference: `claude/plugin-marketplaces.md`.

## §3 — Codex plugin manifest (`.codex-plugin/plugin.json`)

`.codex-plugin/plugin.json` is the required entry point. The other fields are optional, but published plugins commonly use them.

```json
{
  "name": "my-plugin",
  "version": "0.1.0",
  "description": "Bundle reusable skills and app integrations.",
  "author": {
    "name": "Your team",
    "email": "team@example.com",
    "url": "https://example.com"
  },
  "homepage": "https://example.com/plugins/my-plugin",
  "repository": "https://github.com/example/my-plugin",
  "license": "MIT",
  "keywords": ["research", "crm"],
  "skills": "./skills/",
  "mcpServers": "./.mcp.json",
  "apps": "./.app.json",
  "interface": {
    "displayName": "My Plugin",
    "shortDescription": "Reusable skills and apps",
    "longDescription": "Distribute skills and app integrations together.",
    "developerName": "Your team",
    "category": "Productivity",
    "capabilities": ["Read", "Write"],
    "websiteURL": "https://example.com",
    "privacyPolicyURL": "https://example.com/privacy",
    "termsOfServiceURL": "https://example.com/terms",
    "defaultPrompt": [
      "Use My Plugin to summarize new CRM notes.",
      "Use My Plugin to triage new customer follow-ups."
    ],
    "brandColor": "#10A37F",
    "composerIcon": "./assets/icon.png",
    "logo": "./assets/logo.png",
    "screenshots": ["./assets/screenshot-1.png"]
  }
}
```

Top-level fields:
- `name`, `version`, `description` — identify the plugin
- `author`, `homepage`, `repository`, `license`, `keywords` — publisher and discovery metadata
- `skills`, `mcpServers`, `apps` — relative `./...` paths to bundled components
- `interface` — install-surface metadata (see below)

`interface` object:
- `displayName`, `shortDescription`, `longDescription` — title and descriptive copy
- `developerName`, `category`, `capabilities[]` — publisher and capability metadata
- `websiteURL`, `privacyPolicyURL`, `termsOfServiceURL` — external links (also accept `*Url` lowercase aliases)
- `defaultPrompt` — starter prompts (string OR array, **max 3 items, ≤128 chars each** per upstream `MAX_DEFAULT_PROMPT_COUNT` / `MAX_DEFAULT_PROMPT_LEN`)
- `brandColor`, `composerIcon`, `logo`, `screenshots[]` — visual presentation

Path rules:
- Manifest paths must be relative to the plugin root and start with `./`
- Visual assets (`composerIcon`, `logo`, `screenshots`) conventionally live under `./assets/`
- `skills` for bundled skill folders; `apps` for `.app.json`; `mcpServers` for `.mcp.json`

**Note on parser vs. spec scope:** the local Rust parser at `codex-rs/core-plugins/src/manifest.rs` only deserializes a subset of these fields (`name`, `version`, `description`, `skills`, `mcpServers`, `apps`, `interface`). The remaining fields (`author`, `homepage`, `repository`, `license`, `keywords`) are valid published-manifest fields consumed by the marketplace UI and publishing tooling, not by the CLI's local runtime. Don't mistake "field absent from `RawPluginManifest`" for "field rejected by the platform" — `serde_json` ignores unknown keys by default, and the spec is broader than the local parser.

Reference: `codex/plugins-build.md`, `codex/plugins.md`, official spec at `developers.openai.com/codex/plugins`.

## §4 — Codex marketplace manifest (`.agents/plugins/marketplace.json`)

For a repo-scoped marketplace at `$REPO_ROOT/.agents/plugins/marketplace.json`. Plugins themselves live under `$REPO_ROOT/plugins/`.

```json
{
  "name": "local-repo",
  "plugins": [
    {
      "name": "my-plugin",
      "source": { "path": "./plugins/my-plugin" },
      "interface": { "displayName": "My Plugin" }
    }
  ]
}
```

| Field | Required | Notes |
|---|---|---|
| `name` | yes | marketplace identifier |
| `plugins[].name` | yes | plugin id |
| `plugins[].source.path` | yes | `./`-prefixed relative path from the marketplace file to the plugin dir |
| `plugins[].interface.displayName` | recommended | label shown in Codex's plugin picker |

After updating, users restart Codex and run `codex plugin marketplace add ./` (or the repo URL) to register the marketplace.

Reference: `codex/plugins-build.md`.

## §5 — Gemini extension manifest (`gemini-extension.json` at repo root)

The repo **is** the extension — manifest at the root, components in conventional subdirectories.

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My awesome extension",
  "mcpServers": {
    "my-server": {
      "command": "node",
      "args": ["${extensionPath}/my-server.js"],
      "cwd": "${extensionPath}"
    }
  },
  "contextFileName": "GEMINI.md",
  "excludeTools": ["run_shell_command"],
  "migratedTo": "https://github.com/new-owner/new-extension-repo",
  "plan": { "directory": ".gemini/plans" },
  "themes": [
    {
      "name": "shades-of-green",
      "type": "custom",
      "background": { "primary": "#1a362a" },
      "text": { "primary": "#a6e3a1", "secondary": "#6e8e7a", "link": "#89e689" },
      "status": { "success": "#76c076", "warning": "#d9e689", "error": "#b34e4e" },
      "border": { "default": "#4a6c5a" }
    }
  ]
}
```

| Field | Required | Notes |
|---|---|---|
| `name` | yes | Lowercase letters / numbers / dashes only. **Must match the extension's directory name.** Used for conflict resolution when extension command names clash with user/project commands. This is what users type to refer to the extension in the CLI. |
| `version` | yes | Semver. |
| `description` | recommended | Shown on `geminicli.com/extensions` and by `gemini extensions list`. |
| `migratedTo` | optional | URL of the new repository source. If set, the CLI auto-checks the new source for updates and migrates the install. |
| `mcpServers` | optional | Map of name → server config. Same shape as `settings.json` `mcpServers` **except `trust` is not supported**. Use `${extensionPath}` for paths. Split executable + args via `command` + `args`. |
| `contextFileName` | optional | Filename(s) loaded as context. String or string-array. If omitted and `GEMINI.md` exists in the extension dir, that file is loaded by default. |
| `excludeTools` | optional | Array of tool names. Supports per-command restrictions for tools that allow it: `["run_shell_command(rm -rf)"]` blocks only `rm -rf` rather than the entire tool. Distinct from per-MCP-server `excludeTools`. |
| `plan.directory` | optional | Where the extension stores planning artifacts. Falls back to user setting; final default is `~/.gemini/tmp/<project>/<session-id>/plans/`. |
| `settings` | optional | Array of installation-time settings the user provides (e.g. API keys). Each entry: `{ name, description, envVar, sensitive? }`. Values stored in a `.env` inside the extension dir; sensitive ones go to the system keychain. Update via `gemini extensions config <name> [setting] [--scope <scope>]`. |
| `themes` | optional | Array of inline theme objects with color tokens (`background`, `text`, `status`, `border`, `ui`). User-selectable via `/theme` or `ui.theme` in `settings.json`. Names are scoped — referenced as `<theme-name> (<extension-name>)`. |

`settings[]` entry shape:

```json
{
  "name": "API Key",
  "description": "Your API key for the service.",
  "envVar": "MY_API_KEY",
  "sensitive": true
}
```

- `name` — display name shown in the install/config UI
- `description` — explanation shown to the user
- `envVar` — the environment variable that holds the value at runtime
- `sensitive` — when `true`, the value is stored in the system keychain and obfuscated in the UI

Component subdirectories the extension can ship (these are NOT declared in the manifest — they're discovered by directory convention):

| Subdirectory | Purpose |
|---|---|
| `commands/` | custom slash commands (TOML) |
| `agents/` | subagent definition files (`.md`) |
| `hooks/hooks.json` | hooks file (note: subdir, not the root `settings.json`) |
| `skills/` | skill directories with `SKILL.md` |
| `policies/` | policy `.toml` files |

Variables available inside the extension manifest:
- `${extensionPath}` — absolute path to the extension's directory
- `${workspacePath}` — absolute path to the current workspace (when invoked)

**Distribution:** Gemini has no marketplace concept. Users run `gemini extensions install https://github.com/<org>/<repo>`. The repo IS the install target.

Reference: `gemini/extensions.md`, `gemini/extensions-reference.md`, `gemini/extensions-writing-extensions.md`.

---

## §6 — Multi-platform packaging strategy

When the user wants ONE repo to serve all three platforms, here's the layout that works:

```
<repo>/
├── .claude-plugin/
│   ├── marketplace.json          ← Claude marketplace catalog
│   └── (no plugin.json here unless this dir is also a single-plugin)
├── plugins/                      ← Claude marketplace plugins live here
│   ├── plugin-a/
│   │   ├── .claude-plugin/plugin.json
│   │   ├── .codex-plugin/plugin.json   ← if also a Codex plugin
│   │   ├── skills/
│   │   ├── agents/
│   │   └── ...
│   └── plugin-b/
│       └── ...
├── .agents/
│   └── plugins/
│       └── marketplace.json      ← Codex marketplace catalog (points at the same plugins/<name>/ dirs)
├── gemini-extension.json         ← This repo as a Gemini extension
├── commands/                     ← Gemini extension commands
├── agents/                       ← Gemini extension agents
├── hooks/hooks.json              ← Gemini extension hooks
├── README.md                     ← Install instructions for ALL THREE platforms
└── ...
```

Tradeoffs to surface to the user:

- A Codex marketplace can re-use the same `plugins/<plugin-name>/` directories Claude uses, **but the per-plugin manifest path differs** (`.claude-plugin/plugin.json` vs `.codex-plugin/plugin.json`). You may need both manifests in each plugin dir.
- Gemini's "the repo IS the extension" model means **the repo can ship at most ONE Gemini extension**. If your Claude marketplace serves 5 plugins, Gemini gets one consolidated extension or 5 separate repos — not 5 extensions in this repo.
- README must document install commands for every targeted platform:
  ```
  # Claude
  /plugin marketplace add github.com/owner/repo
  /plugin install plugin-a@my-marketplace

  # Codex
  codex plugin marketplace add owner/repo
  codex plugin install plugin-a@my-marketplace

  # Gemini (one extension per repo)
  gemini extensions install https://github.com/owner/repo
  ```

---

## §7 — Validation checklist (REQUIRED before declaring done)

### Per Claude plugin
- [ ] `.claude-plugin/plugin.json` exists and is valid JSON (`jq . <path>`)
- [ ] `name` field present; matches the marketplace entry's `name`
- [ ] All component paths in the manifest resolve (the directories/files exist)
- [ ] No `../` paths reaching outside the plugin directory
- [ ] If `userConfig` is declared, all keys are documented
- [ ] **`claude plugin validate <plugin-dir>` exits 0** — official Claude validator catches schema and structural issues the manual checks above can miss

### Per Claude marketplace
- [ ] `.claude-plugin/marketplace.json` exists and is valid JSON
- [ ] `name`, `owner`, `plugins[]` all present
- [ ] Every `plugins[].source` resolves
- [ ] Every plugin directory contains a `.claude-plugin/plugin.json`
- [ ] **`claude plugin validate <marketplace-root>` exits 0** — run before pushing the marketplace anywhere; this is the same validator the marketplace machinery uses on install

### Per Codex plugin
- [ ] `.codex-plugin/plugin.json` exists and is valid JSON
- [ ] `name`, `version`, `description` present
- [ ] Components in conventional subdirs (`agents/`, `skills/`, `hooks/`, `commands/`, `mcp/`)

### Per Codex marketplace
- [ ] `.agents/plugins/marketplace.json` exists at repo root
- [ ] Every `plugins[].source.path` is `./`-prefixed and resolves
- [ ] `interface.displayName` set on every entry

### Per Gemini extension
- [ ] `gemini-extension.json` at repo root, valid JSON
- [ ] `name` and `version` present
- [ ] If `mcpServers` declared, `${extensionPath}` used in command paths
- [ ] Component subdirs (`commands/`, `agents/`, `hooks/`, `skills/`) match what the manifest implies

### Cross-platform (multi-platform repos only)
- [ ] README documents install commands for every platform the repo targets
- [ ] If a component (e.g. an agent) is portable, it appears in the Claude plugin AND under the Codex+Gemini-compatible locations
- [ ] Platform-only components (Claude `outputStyles`/`themes`, Codex `PermissionRequest` hooks, Gemini `BeforeModel` hooks) are documented as such

**Failing any of these means the bundle isn't done.** A malformed `marketplace.json` is silently rejected by the platform — the user installs the plugin, sees nothing, and has no error. Validation is non-negotiable.

### Schema validation

`agent-config/references/schemas/` contains JSON Schemas for both the **packaged distribution** manifests (synthesized from this document) and the **runtime settings** files (fetched from upstream publishers):

| File | Schema | Source |
|---|---|---|
| `.claude-plugin/plugin.json` | `claude-plugin.schema.json` | upstream — `json.schemastore.org/claude-code-plugin-manifest.json` |
| `.claude-plugin/marketplace.json` | `claude-marketplace.schema.json` | upstream — `json.schemastore.org/claude-code-marketplace.json` |
| `.codex-plugin/plugin.json` | `codex-plugin.schema.json` | derived from upstream `RawPluginManifest` Rust type (`codex-rs/core-plugins/src/manifest.rs` in `openai/codex`) |
| `.agents/plugins/marketplace.json` | `codex-marketplace.schema.json` | derived from upstream `RawMarketplaceManifest` Rust type (`codex-rs/core-plugins/src/marketplace.rs` in `openai/codex`) |
| `gemini-extension.json` | `gemini-extension.schema.json` | derived from upstream `ExtensionConfig` TypeScript interface (`packages/cli/src/config/extension.ts` in `google-gemini/gemini-cli`) |
| `.claude/settings.json` / `settings.local.json` | `claude-settings.schema.json` | upstream — `json.schemastore.org/claude-code-settings.json` |
| `.codex/config.toml` | `codex-config.schema.json` | upstream — `developers.openai.com/codex/config-schema.json` |
| `.gemini/settings.json` | `gemini-settings.schema.json` | upstream — `github.com/google-gemini/gemini-cli/main/schemas/settings.schema.json` |

Two auto-routing validator scripts:

```bash
# Plugin / marketplace / extension manifests:
.claude/skills/agent-config/scripts/validate-manifest.sh <path>

# Runtime settings / config files (incl. TOML → JSON conversion):
.claude/skills/agent-config/scripts/validate-settings.sh <path>
```

**Synthesized vs upstream:**
- **5 schemas are upstream-published**: Claude plugin + marketplace, Claude/Codex/Gemini settings & config. Refresh them with:
  ```bash
  .claude/skills/agent-config/scripts/refresh-schemas.sh
  ```
- **3 schemas are derived from upstream type definitions** in-repo (Codex plugin, Codex marketplace, Gemini extension manifest). Codex and Gemini don't publish JSON Schemas, but their CLI source code defines the types these manifests are parsed against. The schemas in `references/schemas/` mirror those upstream types, including non-obvious constraints the field tables in §3–§5 don't capture (e.g. Codex's `MAX_DEFAULT_PROMPT_COUNT = 3` cap on `interface.defaultPrompt`, Codex's `source` enum tagged variants `local`/`url`/`git-subdir`, Gemini's `contextFileName` accepting either string OR string array).
- When upstream types change, update the matching schema by re-reading the type definition. `refresh-schemas.sh` does NOT touch these — they're hand-derived, not URL-fetched.

**Authoritative vs schema validation for Claude plugins:**
- The upstream Claude `plugin.json` and `marketplace.json` schemas are rich (cover hooks, dependencies, etc.) — schema validation alone catches most structural issues.
- `claude plugin validate <dir>` performs full semantic validation that the marketplace machinery runs at install time. Always run it as a complement when shipping a plugin or marketplace.
