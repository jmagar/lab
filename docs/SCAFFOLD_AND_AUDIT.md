# Scaffold & Audit: Complete Reference

This document is the single authoritative reference for the `lab scaffold` and `lab audit`
systems — how they work, what they produce, how they verify, and every invariant that
governs them.

---

## Table of Contents

1. [Purpose and Philosophy](#1-purpose-and-philosophy)
2. [Scaffold: Overview](#2-scaffold-overview)
3. [Scaffold: CLI Interface](#3-scaffold-cli-interface)
4. [Scaffold: Types and Validation](#4-scaffold-types-and-validation)
5. [Scaffold: What Gets Generated](#5-scaffold-what-gets-generated)
6. [Scaffold: Template System](#6-scaffold-template-system)
7. [Scaffold: Patchers](#7-scaffold-patchers)
8. [Scaffold: Write Safety](#8-scaffold-write-safety)
9. [Scaffold: Idempotency](#9-scaffold-idempotency)
10. [Audit: Overview](#10-audit-overview)
11. [Audit: CLI Interface](#11-audit-cli-interface)
12. [Audit: Check Categories](#12-audit-check-categories)
13. [Audit: Check Reference](#13-audit-check-reference)
14. [Audit: Output and Exit Codes](#14-audit-output-and-exit-codes)
15. [The `lab_admin` MCP Tool](#15-the-lab_admin-mcp-tool)
16. [End-to-End Onboarding Workflow](#16-end-to-end-onboarding-workflow)
17. [Anchor Tokens and Pattern Contracts](#17-anchor-tokens-and-pattern-contracts)
18. [Invariants and Hard Rules](#18-invariants-and-hard-rules)
19. [Extending the System](#19-extending-the-system)
20. [Known Gaps and Limitations](#20-known-gaps-and-limitations)

---

## 1. Purpose and Philosophy

### Why scaffold exists

Adding a new service to `lab` requires touching 12+ files across two crates. Without automation,
each onboarding is a multi-hour exercise in copying patterns and a vector for subtle omissions.
The scaffold system codifies every required file and registration point into a single command.

### Why audit exists

The scaffold generates the *skeleton*. The engineer then fills it in. Audit answers the
question "is this service actually wired up correctly?" by mechanically checking every
registration point, file, and structural contract without human guesswork.

### The three-step contract

```
lab scaffold service <name>   →  generates skeleton files and repo patches
# ... engineer implements the service ...
lab audit onboarding <name>   →  verifies all wiring is in place
cargo test --all-features     →  confirms it compiles and passes tests
```

These three steps, in order, are the only recognized path to declaring a service "online".

---

## 2. Scaffold: Overview

Entry point: `crates/lab/src/scaffold.rs`

`scaffold_service()` is the single public function. It:

1. Validates the service name (see §4).
2. Builds a list of `FileOp` entries — one per file to create or patch.
3. In dry-run mode: returns the plan without touching the filesystem.
4. In write mode: writes each file via a temp-file-then-persist atomic write, skipping files
   whose content is already identical.

```
scaffold_service(config, dry_run) -> ScaffoldResult
│
├── validate_service_name()          ← rejects invalid names early
├── service_file_ops()               ← 13 new-file FileOps
├── patcher::compute_patches()       ← 11 patch FileOps (reads existing files)
│
└── if !dry_run:
    └── write_file() per FileOp      ← atomic: tempfile → persist
```

The result carries:

- `planned` — every `FileOp` regardless of dry-run
- `created` — paths that were newly written
- `modified` — paths that existed and changed

---

## 3. Scaffold: CLI Interface

Source: `crates/lab/src/cli/scaffold.rs`

```
lab scaffold service <name> [--kind <kind>] [--dry-run] [-y]
```

### Flags

| Flag | Short | Default | Effect |
|------|-------|---------|--------|
| `--kind` | — | `http` | `http` or `non-http` (see §4 for ScaffoldKind) |
| `--dry-run` | — | false | Plan only; print what would be created/modified |
| `--yes` | `-y` | false | Suppress interactive confirmation |

### Confirmation behavior

- **`--dry-run`**: no confirmation needed; never writes files.
- **`--yes` supplied**: writes immediately without prompting.
- **On a TTY without `--yes`**: prints `"scaffold will write files for service '<name>'. Continue? [y/N]"` and waits.
- **Non-TTY without `--yes`**: exits with `"pass -y / --yes to confirm scaffold writes"`.

### Output modes

| Mode | Trigger | Content |
|------|---------|---------|
| Human | default | `render_scaffold_result()` — service name, mode, created/modified lists |
| JSON | `--json` | Serialized `ScaffoldResult` struct |

### Example session

```bash
# Preview what will be generated
lab scaffold service mysvc --dry-run

# Generate for real
lab scaffold service mysvc --yes

# Generate a non-HTTP service (no HTTP client, no AppState field)
lab scaffold service mylocal --kind non-http --yes
```

---

## 4. Scaffold: Types and Validation

Source: `crates/lab/src/scaffold/service.rs`

### Service name validation

`validate_service_name(name)` rejects anything that does not match `^[a-z][a-z0-9_]{1,63}$`:

- Must start with an ASCII lowercase letter
- Remaining characters: ASCII lowercase, digits, or `_`
- Minimum length: 2 characters (one `first` + at least one more in `rest`)
- Maximum length: 65 characters (1 + 63 + 1 implicit first)
- Hyphens, uppercase, digits-first, and path traversal are all rejected

This validation fires before any path construction, preventing injection via the service name.

### ScaffoldKind

```rust
pub enum ScaffoldKind {
    Http,     // default — generates HttpClient, AppState field, API routes
    NonHttp,  // for local/SSH/shell services with no external HTTP endpoint
}
```

`NonHttp` is serialized as `"non-http"` in JSON and accepted as `--kind non-http` on the CLI.
Both kinds generate the same file tree — the distinction is currently cosmetic in the scaffolded
code but signals intent for engineers who review the generated output.

### ScaffoldConfig

```rust
pub struct ScaffoldConfig {
    pub service: String,    // validated name
    pub kind: ScaffoldKind,
    pub repo_root: PathBuf, // absolute path to the workspace root
}
```

`repo_root` is always set to `std::env::current_dir()` by the CLI. The MCP dispatch layer
accepts an optional `repo_root` param to allow explicit override (relevant when CWD is not
the repo root).

### FileOp

```rust
pub struct FileOp {
    pub path: PathBuf,   // relative to repo_root
    pub content: String, // full rendered content
}
```

### ScaffoldResult

```rust
pub struct ScaffoldResult {
    pub service: String,
    pub kind: ScaffoldKind,
    pub dry_run: bool,
    pub planned: Vec<FileOp>,   // all ops (dry-run and write)
    pub created: Vec<PathBuf>,  // only on !dry_run, newly created
    pub modified: Vec<PathBuf>, // only on !dry_run, updated in place
}
```

### ScaffoldError variants

| Variant | When |
|---------|------|
| `InvalidServiceName` | Name fails regex validation |
| `InvalidTarget` | Path escapes repo root (via `..` or symlink) |
| `Io` | Filesystem error during read or write |
| `Toml` | TOML parse or serialization failure in a patcher |

---

## 5. Scaffold: What Gets Generated

Scaffolding one service produces **24 file operations**: 13 new files plus 11 repo patches.

### New files (13)

| Relative path | Template source | Purpose |
|---------------|-----------------|---------|
| `crates/lab-apis/src/<svc>.rs` | `lab_apis_service.tpl` | Module entry: META, ServiceClient impl |
| `crates/lab-apis/src/<svc>/client.rs` | `lab_apis_client.tpl` | HTTP client struct and methods |
| `crates/lab-apis/src/<svc>/types.rs` | `lab_apis_types.tpl` | Request/response types |
| `crates/lab-apis/src/<svc>/error.rs` | `lab_apis_error.tpl` | Typed error enum |
| `crates/lab/src/dispatch/<svc>.rs` | `dispatch_entrypoint.tpl` | Dispatch module entrypoint, re-exports |
| `crates/lab/src/dispatch/<svc>/catalog.rs` | `dispatch_catalog.tpl` | Action catalog (ACTIONS const) |
| `crates/lab/src/dispatch/<svc>/client.rs` | `dispatch_client.tpl` | Env/instance resolution, client construction |
| `crates/lab/src/dispatch/<svc>/dispatch.rs` | `dispatch_dispatch.tpl` | Action router + dispatch_with_client |
| `crates/lab/src/dispatch/<svc>/params.rs` | `dispatch_params.tpl` | Param coercion helpers |
| `crates/lab/src/cli/<svc>.rs` | `adapter_cli.tpl` | CLI adapter (clap Args + run()) |
| `crates/lab/src/mcp/services/<svc>.rs` | `adapter_mcp.tpl` | MCP adapter (thin forwarder) |
| `crates/lab/src/api/services/<svc>.rs` | `adapter_api.tpl` | API adapter (axum route group) |
| `docs/coverage/<svc>.md` | `coverage_doc.tpl` | Coverage documentation stub |

### Patched files (11)

| File patched | Patcher function | What gets inserted |
|-------------|------------------|--------------------|
| `crates/lab-apis/src/lib.rs` | `patch_lib_rs` | `#[cfg(feature = "<svc>")] pub mod <svc>;` |
| `crates/lab-apis/Cargo.toml` | `patch_lab_apis_cargo` | `<svc> = []` in `[features]` |
| `crates/lab/Cargo.toml` | `patch_lab_cargo` | `<svc> = ["lab-apis/<svc>"]` in `[features]` |
| `crates/lab/src/dispatch.rs` | `patch_dispatch_rs` | `#[cfg(feature = "<svc>")] pub mod <svc>;` |
| `crates/lab/src/cli.rs` | `patch_cli_rs` | Module decl + enum variant + dispatch arm |
| `crates/lab/src/mcp/services.rs` | `patch_mcp_services_rs` | `#[cfg(feature = "<svc>")] pub mod <svc>;` |
| `crates/lab/src/registry.rs` | `patch_mcp_registry_rs` | `register_service!(reg, "<svc>", <svc>);` |
| `crates/lab/src/api/services.rs` | `patch_api_services_rs` | `#[cfg(feature = "<svc>")] pub mod <svc>;` |
| `crates/lab/src/api/router.rs` | `patch_api_router_rs` | Feature-gated + registry-checked route mount |
| `crates/lab/src/api/state.rs` | `patch_api_state_rs` | AppState field + `from_env()` load arm |
| `crates/lab/src/tui/metadata.rs` | `patch_tui_metadata_rs` | `// scaffolded service: <svc>` comment |

---

## 6. Scaffold: Template System

Source: `crates/lab/src/scaffold/templates/`

Templates are `.tpl` files loaded via `include_str!()` at compile time. The template engine
is entirely string-replacement: no external crate, no conditionals, no loops. The substitutions
are:

| Placeholder | Value | Example (`service = "myapi"`) |
|-------------|-------|-------------------------------|
| `{{service}}` | service name as-is | `myapi` |
| `{{Service}}` | PascalCase of name | `Myapi` |
| `{{SERVICE}}` | SCREAMING_SNAKE_CASE | `MYAPI` |

`pascal_case()` converts underscore-delimited names to PascalCase: `my_api` → `MyApi`,
`myapi` → `Myapi`.

### Template index

#### `lab_apis_service.tpl`
Generates the `lab-apis` module entry file. Contains:
- `pub mod client; pub mod error; pub mod types;`
- `pub use client::{Service}Client;`
- `pub const META: PluginMeta` with `Category::Bootstrap` placeholder
- `impl ServiceClient` with `health()` returning `ServiceStatus::degraded("scaffolded service placeholder")`

#### `lab_apis_client.tpl`
Generates `{Service}Client` struct wrapping `HttpClient`. Includes:
- `new(base_url, auth) -> Result<Self, {Service}Error>`
- placeholder `health()` method

#### `lab_apis_types.tpl`
Minimal: one `Placeholder` struct deriving `Debug, Clone, Default, Serialize, Deserialize`.

#### `lab_apis_error.tpl`
One enum variant: `Api(#[from] ApiError)`. Engineers replace this with service-specific error variants.

#### `dispatch_entrypoint.tpl`
The dispatch module entry (`dispatch/{svc}.rs`). Re-exports the four canonical symbols:
```rust
pub use catalog::ACTIONS;
pub use client::{client_from_env, not_configured_error, require_client};
pub use dispatch::{dispatch, dispatch_with_client};
```

#### `dispatch_catalog.tpl`
Initializes `ACTIONS` with only `help` and `schema` entries. Engineers add real actions here.

#### `dispatch_client.tpl`
Follows the canonical `client.rs` template exactly:
- `client_from_env()` — reads `{SERVICE}_URL`, constructs client, returns `Option<{Service}Client>`
- `require_client()` — wraps `client_from_env()` in `ok_or_else(not_configured_error)`
- `not_configured_error()` — returns `ToolError::Sdk { sdk_kind: "internal_error", ... }`

Auth is scaffolded as `Auth::None`; engineers replace with the correct auth variant
(`ApiKey`, `Bearer`, `Basic`, etc.) per the upstream API spec.

#### `dispatch_dispatch.tpl`
The split `dispatch` / `dispatch_with_client` pattern:
- `dispatch(action, params)` — intercepts `help` and `schema`; delegates rest to `dispatch_with_client(&require_client()?, ...)`
- `dispatch_with_client(client, action, params)` — currently returns `UnknownAction` for everything; engineers add match arms

This split enables the API surface to call `dispatch_with_client` with a pre-built client
from `AppState`, avoiding a redundant `client_from_env()` call per request.

#### `dispatch_params.tpl`
Empty stub with a doc comment. Engineers add typed param coercion helpers here.

#### `adapter_cli.tpl`
Tier-2 CLI adapter: raw `action: String` + `params: Vec<String>` (key=value), delegates to
`run_confirmable_action_command()`. Engineers replace this with a typed Tier-1 adapter
(`radarr.rs` is the reference) once the service has a stable action set.

#### `adapter_mcp.tpl`
Six lines: re-exports `ACTIONS` from dispatch, then `dispatch()` forwards to
`crate::dispatch::{svc}::dispatch()`. No logic lives here.

#### `adapter_api.tpl`
Axum route group: one `POST /` route that calls `handle_action()` from the shared API helper.
Uses `dispatch_with_client()` with the pre-built client from `AppState`. Includes the
`x-request-id` header extraction pattern.

#### `coverage_doc.tpl`
One-liner stub: `# {svc} onboarding coverage` + `TODO: document...`

---

## 7. Scaffold: Patchers

Source: `crates/lab/src/scaffold/patcher/`

Two patcher modules handle two different concerns:

- `source.rs` — patches Rust source files (string surgery)
- `toml.rs` — patches Cargo.toml `[features]` sections (TOML parse + serialize)

### Source patchers (`source.rs`)

All source patchers use two primitives:

**`insert_before_eof(content, insert)`**
Appends `insert` to the end of `content`. Idempotent: skips if already present.

**`insert_once(content, needle, replacement)`**
Finds `needle` in `content` and replaces it with `replacement`. Idempotent: skips if
`replacement` is already present. Returns an error if `needle` is not found.

The strategy: each patcher anchors on a token guaranteed to be present in the file
(e.g. `"pub mod serve;\n"` in `cli.rs`, `"    reg\n}"` in `registry.rs`). The replacement
prepends the new content before the anchor, leaving the anchor in place. This means the
file grows from the bottom of each section upward.

#### `patch_lib_rs`
Appends `#[cfg(feature = "<svc>")] pub mod <svc>;` before EOF.

#### `patch_dispatch_rs`
Anchors on `"pub mod helpers;\n"`. Inserts the feature-gated mod declaration after `helpers`.

#### `patch_cli_rs`
Three insertions (all sequential — each feeds the result of the previous as input):

1. Inserts `pub mod <svc>;` after `pub mod serve;`
2. Inserts the `#[cfg(feature = "<svc>")] <Service>(<svc>::<Service>Args),` enum variant
   before the `Apprise` variant in the `Command` enum (alphabetical anchor)
3. Inserts the dispatch arm `Command::<Service>(args) => <svc>::run(args, format).await,`
   before the `Apprise` dispatch arm

The sequential chaining is critical — each step operates on the output of the previous,
not the original file.

#### `patch_mcp_services_rs`
Appends the feature-gated mod before EOF.

#### `patch_mcp_registry_rs`
Anchors on `"    reg\n}"` (the closing of `build_default_registry()`). Inserts
`register_service!(reg, "<svc>", <svc>);` immediately before `reg`.

#### `patch_api_services_rs`
Appends the feature-gated mod before EOF.

#### `patch_api_router_rs`
Anchors on `"    let x_request_id = HeaderName::from_static(\"x-request-id\");"`.
Inserts the full two-guard route mount before that line:

```rust
#[cfg(feature = "<svc>")]
if state.registry.services().iter().any(|s| s.name == "<svc>") {
    v1 = v1.nest("/<svc>", services::<svc>::routes(state.clone()));
}
```

Both guards are required. The `#[cfg(feature)]` gate ensures the handler module compiles;
the runtime registry check ensures that `lab serve --services X` cannot be bypassed.

#### `patch_api_state_rs`
Two insertions:

1. The struct field: `#[cfg(feature = "<svc>")] pub <svc>: Option<Arc<lab_apis::<svc>::<Service>Client>>,`
   inserted after the `prowlarr` field
2. The load arm: `#[cfg(feature = "<svc>")] <svc>: crate::dispatch::<svc>::client_from_env().map(Arc::new),`
   inserted after the `prowlarr` load arm

`prowlarr` is used as the anchor because it is always present and near the end of the service
list. New services appear immediately after it.

#### `patch_tui_metadata_rs`
Appends `// scaffolded service: <svc>` before EOF. This exact string is the audit check
token for TUI registration (see §13).

### TOML patchers (`toml.rs`)

Both `patch_lab_apis_cargo` and `patch_lab_cargo` delegate to `patch_feature_section()`.

**Algorithm:**
1. `split_feature_section(content)` — scans the file line-by-line to locate `[features]` and the
   next section header. Returns `(prefix, body, suffix)` where `prefix` = everything through
   the `[features]` header, `body` = the key-value lines, `suffix` = rest.
2. Parse `body` with `toml::from_str`.
3. If the key already exists: return the original content unchanged (idempotent).
4. Build a `reordered` map: copy all existing entries, inserting the new entry just before `all`
   (or appending if `all` is absent).
5. Serialize `reordered` with `toml::to_string_pretty`.
6. Strip the `[features]` header from the serialized output (it was added for the parse step).
7. Reassemble: `prefix + rendered_body + suffix`.

**Note on key ordering:** The `toml` crate (v1.x without `preserve_order`) uses `BTreeMap` for
its map type, which sorts keys alphabetically. The `reordered` insertion logic attempts to place
the new key before `all`, but the final serialized output is alphabetically sorted. This is
semantically correct for Cargo but may not match the hand-maintained style of the surrounding
file. The surrounding sections (comments, non-features sections) are preserved exactly via the
`prefix`/`suffix` split.

---

## 8. Scaffold: Write Safety

Source: `validate_path_within_root()` in `crates/lab/src/scaffold.rs`

Every `FileOp` path is validated before any filesystem write. Two checks run:

### Check 1: No `..` components

The relative path is walked component-by-component. Any `Component::ParentDir` (`..`) causes
immediate rejection with `ScaffoldError::InvalidTarget`. This prevents path traversal via the
service name or any other input.

Since service names are validated to match `^[a-z][a-z0-9_]{1,63}$`, the only way a `..`
could appear is through adversarial construction of the `FileOp` path itself (not the name).

### Check 2: No symlink escape via existing ancestors

After the `..` check, the validator walks up the absolute path looking for the first existing
ancestor directory. When found, that directory is canonicalized and the result is verified to
start with the canonicalized repo root. If a symlink in the directory tree would redirect outside
the root, this check catches it.

```
abs = repo_root.join(op.path)
walk up abs until parent.exists()
    → canonicalize(parent)
    → verify canonical.starts_with(canonical_root)
```

### Atomic writes

Each file is written via `tempfile::NamedTempFile` in the same directory as the target, then
persisted atomically. This prevents partial writes from leaving corrupt files if the process is
interrupted.

```
NamedTempFile::new_in(target_parent)
→ write_all(content)
→ flush()
→ persist(target_path)   // atomic rename
```

---

## 9. Scaffold: Idempotency

Running `scaffold_service` twice on the same service name is safe:

- **New files**: `write_file` reads existing content and returns `Ok(false)` (unchanged) if
  content is byte-identical. No new entries appear in `created` or `modified`.
- **Patches**: each `insert_once` and `insert_before_eof` checks for the presence of the
  replacement string before inserting. If already present, the file is not rewritten.
- **TOML patches**: `patch_feature_section` returns the original content unchanged if the key
  already exists.

The net result: repeated scaffolding is a no-op. `ScaffoldResult.created` and `.modified`
will both be empty on the second run.

---

## 10. Audit: Overview

Source: `crates/lab/src/audit/`

`audit_services(names, repo_root)` runs a battery of static checks against the repository
filesystem for one or more service names. No network calls, no compilation — pure filesystem
and string matching.

```
audit_services(names, repo_root) -> AuditReport
│
├── SharedContext::load(repo_root)   ← reads 10 shared files once
│
└── per name:
    └── audit_service(name, shared, repo_root) -> ServiceReport
        ├── checks::files::run()         ← 12 file-existence checks
        ├── checks::registration::run()  ← 11 registration/token checks
        ├── checks::dispatch::run()      ← 2 dispatch layout checks
        ├── checks::tests::run()         ← 4 file-existence checks (non-fatal)
        └── checks::docs::run()          ← 1 coverage doc check
```

### SharedContext

Ten shared files are read once and cached in a `HashMap<PathBuf, String>`:

```
crates/lab-apis/Cargo.toml
crates/lab/Cargo.toml
crates/lab-apis/src/lib.rs
crates/lab/src/cli.rs
crates/lab/src/mcp/services.rs
crates/lab/src/registry.rs
crates/lab/src/api/services.rs
crates/lab/src/api/router.rs
crates/lab/src/api/state.rs
crates/lab/src/tui/metadata.rs
```

If loading fails (e.g. wrong CWD), a warning is logged and an empty context is used.
Registration checks that depend on these files will produce `Fail` results in that case.

---

## 11. Audit: CLI Interface

Source: `crates/lab/src/cli/audit.rs`

```
lab audit onboarding <service>... [--json]
```

One or more service names are required. Multiple names audit in series; each gets a
`ServiceReport` in the output.

```bash
# Audit one service
lab audit onboarding myapi

# Audit several
lab audit onboarding myapi another_svc

# JSON output for scripting
lab audit onboarding myapi --json
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | All checks pass or skip — no failures |
| 1 | At least one check returned `Fail` |

`Fail` counts as a failure; `Skip` does not. A service that is partially implemented will
have many `Skip` results but zero `Fail` results and exit 0 — skips are informational.

### Output format (human)

`render_audit_report()` in `output.rs`:

```
myapi:
  - crates/lab-apis/src/myapi.rs: pass
  - crates/lab-apis/src/myapi/client.rs: pass
  - feature.lab-apis: pass
  - mcp.registry.rs: fail: missing token `register_service!(reg, "myapi"`
  - docs.coverage: skip: not yet present: docs/coverage/myapi.md
```

### Output format (JSON)

Serialized `AuditReport`:

```json
{
  "services": [
    {
      "service": "myapi",
      "checks": [
        ["crates/lab-apis/src/myapi.rs", "Pass"],
        ["mcp.registry.rs", {"Fail": "missing token `register_service!(reg, \"myapi\"`"}],
        ["docs.coverage", {"Skip": "not yet present: ..."}]
      ]
    }
  ]
}
```

---

## 12. Audit: Check Categories

Five check modules run in sequence per service. Each returns `Vec<(String, CheckResult)>`.

| Module | Source | Count | What it checks |
|--------|--------|-------|----------------|
| `files` | `checks/files.rs` | 12 | Required files exist on disk |
| `registration` | `checks/registration.rs` | 11 | Tokens present in shared registration files |
| `dispatch` | `checks/dispatch.rs` | 2 | Dispatch layout and client contract |
| `tests` | `checks/tests.rs` | 4 | Optional file presence (SDK test, impl files) |
| `docs` | `checks/docs.rs` | 1 | Coverage doc exists and is non-empty |

**Total: 30 checks per service.**

### CheckResult semantics

```rust
pub enum CheckResult {
    Pass,           // the check is satisfied
    Fail(String),   // the check is not satisfied — message explains why
    Skip(String),   // check cannot be evaluated — treated as neutral
}
```

`Skip` is used for checks that are optional or whose target may not exist yet without
being a bug. Specifically: the four `tests` checks (SDK test file, impl files) always `Skip`
when the files are absent, because these are expected to be missing for newly scaffolded
services.

There is one `Skip`-upgraded `Fail` in registration: `api.state.load` downgrades from `Fail`
to `Skip` because the state client mapping pattern is not yet wired for all services.

---

## 13. Audit: Check Reference

### files (12 checks)

All 12 are `Fail` if the path does not exist — no skipping.

| Check name (= relative path) | Path |
|------------------------------|------|
| `crates/lab-apis/src/<svc>.rs` | Module entry file |
| `crates/lab-apis/src/<svc>/client.rs` | HTTP client |
| `crates/lab-apis/src/<svc>/types.rs` | Types |
| `crates/lab-apis/src/<svc>/error.rs` | Error enum |
| `crates/lab/src/dispatch/<svc>.rs` | Dispatch entrypoint |
| `crates/lab/src/dispatch/<svc>/catalog.rs` | Action catalog |
| `crates/lab/src/dispatch/<svc>/client.rs` | Env/client construction |
| `crates/lab/src/dispatch/<svc>/dispatch.rs` | Action router |
| `crates/lab/src/dispatch/<svc>/params.rs` | Param helpers |
| `crates/lab/src/cli/<svc>.rs` | CLI adapter |
| `crates/lab/src/mcp/services/<svc>.rs` | MCP adapter |
| `crates/lab/src/api/services/<svc>.rs` | API adapter |

### registration (11 checks)

Token-based: `contains_check(file_content, token)` — case-sensitive substring match.

| Check name | File searched | Token searched |
|------------|---------------|----------------|
| `feature.lab-apis` | `crates/lab-apis/Cargo.toml` | `<svc> = [` |
| `feature.lab` | `crates/lab/Cargo.toml` | `<svc> = ["lab-apis/<svc>"]` |
| `lib.rs` | `crates/lab-apis/src/lib.rs` | `#[cfg(feature = "<svc>")]\npub mod <svc>;` |
| `cli.rs` | `crates/lab/src/cli.rs` | `pub mod <svc>;` |
| `mcp.services.rs` | `crates/lab/src/mcp/services.rs` | `#[cfg(feature = "<svc>")]\npub mod <svc>;` |
| `mcp.registry.rs` | `crates/lab/src/registry.rs` | `register_service!(reg, "<svc>"` |
| `api.services.rs` | `crates/lab/src/api/services.rs` | `#[cfg(feature = "<svc>")]\npub mod <svc>;` |
| `api.router.rs` | `crates/lab/src/api/router.rs` | `nest("/<svc>"` |
| `api.state.rs` | `crates/lab/src/api/state.rs` | `pub <svc>: Option<` |
| `tui.metadata.rs` | `crates/lab/src/tui/metadata.rs` | `// scaffolded service: <svc>` |
| `api.state.load` | `crates/lab/src/api/state.rs` | `<svc>: crate::dispatch::<svc>::client_from_env()` |

**Note on `mcp.registry.rs`:** The token is `register_service!(reg, "<svc>"` (the macro call
prefix), not just the bare name. This prevents false passes from the name appearing in comments
or unrelated strings in the registry file.

**Note on `tui.metadata.rs`:** The token is the exact comment `// scaffolded service: <svc>`
that `patch_tui_metadata_rs` inserts. Engineers who manually wire TUI metadata must add this
comment (or replace it with real TUI registration code after the comment).

**Note on `api.state.load`:** This check downgrades to `Skip` when it fails, because not all
services have their client mapping fully threaded into `AppState` yet. It is informational only.

### dispatch (2 checks)

Reads the dispatch entrypoint and client files, checks for required token groups.

| Check name | File | Required tokens |
|------------|------|-----------------|
| `dispatch.entrypoint` | `dispatch/<svc>.rs` | `pub use catalog::ACTIONS;` AND `pub use client::client_from_env;` AND `pub use dispatch::{dispatch, dispatch_with_client};` |
| `dispatch.client` | `dispatch/<svc>/client.rs` | `client_from_env()` AND `require_client()` AND `not_configured_error()` |

Both use `contains_all`: all tokens must be present. Missing any one token causes `Fail`.

### tests (4 checks, all Skip-on-missing)

Uses `file_or_skip`: returns `Pass` if the file exists, `Skip` (never `Fail`) if absent.

| Check name | File checked | What it actually represents |
|------------|-------------|------------------------------|
| `tests.sdk` | `crates/lab-apis/tests/<svc>_client.rs` | SDK integration test (a real test file) |
| `impl.dispatch` | `crates/lab/src/dispatch/<svc>/dispatch.rs` | Dispatch implementation file exists |
| `impl.mcp` | `crates/lab/src/mcp/services/<svc>.rs` | MCP adapter exists |
| `impl.api` | `crates/lab/src/api/services/<svc>.rs` | API adapter exists |

Note: `impl.dispatch`, `impl.mcp`, and `impl.api` check implementation files (not test files)
despite being in the `tests` check module. They were renamed from `tests.dispatch/mcp/api` to
`impl.*` to reflect what they actually verify.

### docs (1 check)

| Check name | File | Pass condition |
|------------|------|----------------|
| `docs.coverage` | `docs/coverage/<svc>.md` | File exists AND is non-empty (after trim) |

---

## 14. Audit: Output and Exit Codes

Source: `crates/lab/src/output.rs` and `crates/lab/src/cli/audit.rs`

### Human rendering

`render_audit_report(report)` writes one section per service:

```
<service>:
  - <check_name>: pass
  - <check_name>: fail: <message>
  - <check_name>: skip: <message>
```

### JSON rendering

`print(&report, OutputFormat::Json)` serializes the full `AuditReport`. `CheckResult` serializes as:

```json
"Pass"
{"Fail": "missing token `...`"}
{"Skip": "not yet present: ..."}
```

### Exit code logic

```rust
fn exit_code(report: &AuditReport) -> ExitCode {
    if report.has_failures() {
        ExitCode::FAILURE   // 1
    } else {
        ExitCode::SUCCESS   // 0
    }
}
```

`has_failures()` returns true if any `ServiceReport` contains any `CheckResult::Fail`. Skips
are neutral.

---

## 15. The `lab_admin` MCP Tool

The audit capability is also exposed as an MCP tool named `lab_admin`, enabling AI agents to
audit services without a CLI session.

### Feature flag

`lab-admin` is a local-only feature flag with no `lab-apis` counterpart:

```toml
# Intentionally local-only — no lab-apis counterpart; performs filesystem audits only.
lab-admin   = []
```

It is included in `all` and activated at runtime only when `LAB_ADMIN_ENABLED=1` is set.

### Runtime activation

```rust
#[cfg(feature = "lab-admin")]
if lab_admin_enabled() {   // checks LAB_ADMIN_ENABLED == "1"
    reg.register(RegisteredService {
        name: "lab_admin",
        category: "bootstrap",
        ...
    });
}
```

The exact value `"1"` is required. Setting `LAB_ADMIN_ENABLED` to any other value (including
`"true"`) does not activate the tool.

### Action catalog

```
lab_admin.help                  → action catalog
lab_admin.schema { action }     → parameter schema for one action
lab_admin.onboarding.audit { services, repo_root? } → AuditReport
```

### `onboarding.audit` parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `services` | `string[]` | yes | Service names to audit |
| `repo_root` | `string` | no | Absolute path to repo root; defaults to `current_dir()` |

If `services` is absent: `MissingParam` error.
If `services` is present but not an array: `InvalidParam` error.
If `services` elements are not strings: `InvalidParam` error per element.

`repo_root` should be supplied explicitly when calling via MCP from a context where the
process CWD may not be the workspace root (e.g. a remote MCP server session).

### Dispatch path

Source: `crates/lab/src/dispatch/lab_admin/`

```
dispatch(action, params)
├── "help"              → help_payload("lab_admin", ACTIONS)
├── "schema"            → action_schema(ACTIONS, params["action"])
├── "onboarding.audit"  → onboarding_audit(params)
│   ├── parse_services(params)            ← typed param coercion
│   ├── resolve repo_root                 ← params["repo_root"] or current_dir()
│   └── spawn_blocking(audit_services)    ← async wrapper for sync filesystem scan
└── unknown             → UnknownAction error with valid action list
```

`audit_services` is synchronous (blocking filesystem reads) and is wrapped in
`tokio::task::spawn_blocking` to avoid stalling the async runtime.

### Module layout

```
crates/lab/src/dispatch/lab_admin.rs          ← module entry, re-exports
crates/lab/src/dispatch/lab_admin/
    catalog.rs    ← ACTIONS const
    client.rs     ← empty; no HTTP client (documented stub)
    dispatch.rs   ← dispatch() and onboarding_audit()
    params.rs     ← parse_services()
```

The `client.rs` file is a documented stub satisfying the required dispatch service layout
contract (every migrated service must have `client.rs`), while making it explicit that
`lab_admin` has no external HTTP service.

---

## 16. End-to-End Onboarding Workflow

### Step 1: Scaffold

```bash
# Preview first
lab scaffold service myapi --dry-run

# Generate for real
lab scaffold service myapi --yes
```

24 file operations run. Check `created` and `modified` in the output.

### Step 2: Implement the SDK client

Edit `crates/lab-apis/src/myapi/`:

- `types.rs` — replace `Placeholder` with real request/response types derived from the API spec
- `error.rs` — add service-specific error variants
- `client.rs` — implement methods, each making an HTTP call via `self.http`
- `myapi.rs` — update `META` (category, description, env vars, default port), complete `health()`

Update `dispatch/myapi/client.rs`:
- Replace `Auth::None` with the correct auth variant
- Add `{SERVICE}_API_KEY` or `{SERVICE}_TOKEN` env read if needed

### Step 3: Implement the dispatch layer

Edit `crates/lab/src/dispatch/myapi/`:

- `catalog.rs` — add `ActionSpec` entries for each real operation
- `params.rs` — add param coercion helpers
- `dispatch.rs` — add match arms in `dispatch_with_client`

### Step 4: Implement the adapters

- **CLI** (`cli/myapi.rs`): start with the scaffolded Tier-2 adapter; upgrade to typed
  Tier-1 once the action set is stable
- **MCP** (`mcp/services/myapi.rs`): the scaffolded adapter is complete — no changes needed
  unless you need custom elicitation
- **API** (`api/services/myapi.rs`): verify `state.clients.myapi` field name matches `AppState`
  field generated by `patch_api_state_rs`

### Step 5: Audit

```bash
lab audit onboarding myapi
```

Fix every `Fail`. `Skip` results are informational. All registration checks should be `Pass`.

### Step 6: Build and test

```bash
cargo test --all-features
cargo clippy --all-features -- -D warnings
```

All-features is the authoritative gate. Narrow feature builds are diagnostic only.

### Step 7: Write a coverage doc

Edit `docs/coverage/myapi.md`. The `docs.coverage` audit check passes once this file
exists and is non-empty.

---

## 17. Anchor Tokens and Pattern Contracts

The scaffold patchers rely on stable anchor tokens in existing files. Breaking these anchors
causes `insert_once` to fail with a `ScaffoldError::Toml { message: "patch anchor not found" }`.

### Stable anchors (must not be removed or reformatted)

| File | Anchor | Used by |
|------|--------|---------|
| `crates/lab/src/cli.rs` | `pub mod serve;\n` | `patch_cli_rs` (mod insertion) |
| `crates/lab/src/cli.rs` | `#[cfg(feature = "apprise")]\n    Apprise(apprise::AppriseArgs),\n` | `patch_cli_rs` (enum variant) |
| `crates/lab/src/cli.rs` | `#[cfg(feature = "apprise")]\n        Command::Apprise(args) => apprise::run(args, format).await,\n` | `patch_cli_rs` (dispatch arm) |
| `crates/lab/src/dispatch.rs` | `pub mod helpers;\n` | `patch_dispatch_rs` |
| `crates/lab/src/registry.rs` | `    reg\n}` | `patch_mcp_registry_rs` |
| `crates/lab/src/api/router.rs` | `    let x_request_id = HeaderName::from_static("x-request-id");` | `patch_api_router_rs` |
| `crates/lab/src/api/state.rs` | `#[cfg(feature = "prowlarr")]\n    pub prowlarr: ...` | `patch_api_state_rs` (field) |
| `crates/lab/src/api/state.rs` | `#[cfg(feature = "prowlarr")]\n            prowlarr: crate::dispatch::prowlarr::...` | `patch_api_state_rs` (load) |

### Audit check tokens (must be present after onboarding)

| What to check | Required token |
|---------------|----------------|
| Feature in lab-apis | `<svc> = [` in `crates/lab-apis/Cargo.toml` |
| Feature in lab | `<svc> = ["lab-apis/<svc>"]` in `crates/lab/Cargo.toml` |
| lib.rs gated mod | `#[cfg(feature = "<svc>")]\npub mod <svc>;` in `lib.rs` |
| MCP services mod | `#[cfg(feature = "<svc>")]\npub mod <svc>;` in `mcp/services.rs` |
| MCP registry | `register_service!(reg, "<svc>"` in `registry.rs` |
| API services mod | `#[cfg(feature = "<svc>")]\npub mod <svc>;` in `api/services.rs` |
| API router mount | `nest("/<svc>"` in `api/router.rs` |
| AppState field | `pub <svc>: Option<` in `api/state.rs` |
| TUI presence | `// scaffolded service: <svc>` in `tui/metadata.rs` |
| Dispatch re-exports | `pub use catalog::ACTIONS;` etc. in `dispatch/<svc>.rs` |
| Client contract | `client_from_env()` + `require_client()` + `not_configured_error()` in `dispatch/<svc>/client.rs` |

---

## 18. Invariants and Hard Rules

These are enforced by code and tests. Violations cause either compile errors or audit failures.

### Service name

- Must match `^[a-z][a-z0-9_]{1,63}$`
- Enforced before any path construction
- Rejection is immediate and non-recoverable (no fallback)

### Module style

- No `mod.rs` files — modern Rust module style only: `foo.rs` sibling to `foo/`
- Scaffold templates enforce this: `myapi.rs` + `myapi/client.rs`, not `myapi/mod.rs`

### Crate boundary

- `lab-apis` never contains `clap`, `rmcp`, `ratatui`, `anyhow`, or `tabled`
- Business logic never lives in `cli/`, `mcp/services/`, or `api/services/`
- Config loading (env reads) never happens inside `lab-apis` — always in `dispatch/*/client.rs`

### Two-guard API router mount

The generated router patch includes both:
1. `#[cfg(feature = "<svc>")]` — compile-time gate
2. `if state.registry.services().iter().any(...)` — runtime registry gate

Both must be present. The runtime gate enforces `lab serve --services` filtering.

### Dispatch split

Every dispatch module exposes:
- `dispatch(action, params)` — intercepts `help`/`schema`, delegates rest
- `dispatch_with_client(client, action, params)` — pure action routing, no env reads

MCP and CLI call `dispatch`; the API handler calls `dispatch_with_client` with the
pre-built `AppState` client.

### Client contract

Every `dispatch/<svc>/client.rs` must expose exactly these three functions:
- `client_from_env() -> Option<SvcClient>` — pure, no side effects, no logging
- `require_client() -> Result<SvcClient, ToolError>` — wraps `client_from_env`
- `not_configured_error() -> ToolError` — returns `ToolError::Sdk { sdk_kind: "internal_error", ... }`

The audit `dispatch.client` check verifies all three are present.

### Atomic writes

All scaffold writes use `tempfile::NamedTempFile` → `persist()`. Never direct `File::create`.

### Idempotency

Every patcher checks for pre-existing content before modifying. Running scaffold twice is
always safe.

### Path safety

Every write is gated by `validate_path_within_root`. Both `..` components and symlink escape
are checked. Path traversal via a crafted service name is impossible (name validation runs first).

---

## 19. Extending the System

### Adding a new check to the audit

1. Add a function to the appropriate module under `crates/lab/src/audit/checks/`
   (or create a new module).
2. Push `(name, result)` pairs to `out`.
3. Call the function from `audit_service()` in `onboarding.rs`.
4. Document the new check in §13 of this file.

### Adding a new file to the scaffold

1. Add a new `.tpl` file in `crates/lab/src/scaffold/templates/`.
2. Add a `pub fn <name>_template(service: &str) -> String` in the appropriate `.rs` file
   under `templates/`.
3. Add a `FileOp` entry in `service_file_ops()` in `scaffold.rs`.
4. If the file will be audited, add the path to the `files::run()` list in `checks/files.rs`.

### Adding a new patch target

1. Add a `pub fn patch_<file>_rs(name: &str, content: &str) -> Result<String>` to
   `patcher/source.rs` (or `patcher/toml.rs` for TOML files).
2. Call it from `patcher::compute_patches()`.
3. Add the corresponding token check to `registration::run()` if the patch produces
   a verifiable token.
4. Update §5 (What Gets Generated), §7 (Patchers), §13 (Check Reference), and §17
   (Anchor Tokens) in this document.

### Adding a new template variable

1. Define the transformation in `templates/*.rs`.
2. Apply it uniformly across all `.tpl` files that need it.
3. Keep substitution to pure string operations — do not add external templating dependencies.

---

## 20. Known Gaps and Limitations

| Gap | Location | Notes |
|-----|----------|-------|
| `surface = "mcp"` hardcoded | `dispatch/lab_admin/dispatch.rs` | Systemic: dispatch layer has no surface context. All surfaces log `"mcp"`. Requires threading a surface param through the full call chain. |
| `api.state.load` check always skips on fail | `checks/registration.rs` | The AppState client mapping pattern is not yet complete for all services. Downgraded to `Skip` intentionally. |
| TUI not fully wired | `tui/metadata.rs` | Only `radarr` has real TUI metadata. Scaffold inserts a comment token; real TUI wiring is manual. |
| `schema` action not implemented | All service dispatchers | `help` is implemented everywhere; `schema` is not. Returns nothing in dispatch_inner for non-`lab_admin` services. |
| TOML feature key order | `patcher/toml.rs` | The `toml` crate (without `preserve_order`) sorts keys alphabetically in the `[features]` section. The surrounding file structure is preserved but feature keys may reorder. |
| `NonHttp` kind has no behavioral difference | `scaffold/service.rs` | `ScaffoldKind::NonHttp` is recorded and serialized but produces identical file output to `Http`. It signals intent only. |
| `repo_root` from CWD in CLI and audit | `cli/audit.rs`, `cli/scaffold.rs` | Both use `std::env::current_dir()`. Running from a subdirectory will produce wrong paths. Always run from the workspace root. |
