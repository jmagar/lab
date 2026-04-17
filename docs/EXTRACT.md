# Extract

`extract` is a synthetic bootstrap service for discovering existing homelab credentials from appdata and writing them into `~/.lab/.env`.

## Why It Exists

Many users already have working service configs but do not want to manually copy URLs and API keys into a new tool. `extract` automates that bootstrap path.

## Why It Is a Service

`extract` is not a remote API integration, but it deliberately uses the same architecture shape as real services:

- `lab-apis` module
- typed client
- CLI shim
- MCP shim
- action catalog
- plugin metadata

This avoids special-case plumbing.

## Compilation Rule

`extract` is always compiled. It is treated like `doctor` and other bootstrap/operator capabilities rather than a feature-gated optional integration.

## Responsibilities

`extract` owns:

- URI parsing
- local filesystem reads
- SSH-backed remote reads
- parser orchestration
- normalized extracted credential reports
- safe `.env` merging and writing
- diffing discovered creds against the current env file

## URI Forms

Supported forms:

- `<host>:/absolute/path`
- `/absolute/path`
- `~/path`

SSH host resolution must follow the user’s SSH configuration rather than inventing a second host lookup mechanism.

## Parser Strategy

Each parser knows:

- where an app stores config under an appdata root
- how to turn that config into service credentials

Missing apps must be skipped quietly. Corrupt configs must produce warnings rather than abort the whole scan.

## Actions

Expected actions:

- `scan`
- `diff`
- `apply`
- `help`

`apply` is destructive because it writes `.env`.

`scan` and `diff` are read-only.

## `.env` Write Rules

The merge/write behavior is intentionally strict:

1. back up first
2. write atomically
3. preserve comments and ordering where possible
4. dedupe by key
5. keep existing conflicting values unless force is requested
6. quote safely
7. avoid partial writes
8. remain idempotent on repeated apply

The default conflict policy is preserve-existing, not overwrite.

## CLI Surface

Typical targeted usage:

```bash
lab extract /path/to/appdata
lab extract host:/path/to/appdata --diff
lab extract host:/path/to/appdata --apply
```

Fleet discovery usage:

```bash
lab extract
```

Bare `lab extract` reads `~/.ssh/config`, inspects reachable hosts for running
supported Docker containers, prefers Tailscale IPs when available, probes
candidate endpoints, and only attempts credential extraction for services with
verified working URLs.

Current fleet discovery supports:

- `radarr`
- `sonarr`
- `prowlarr`
- `sabnzbd`
- `qbittorrent`
- `plex`
- `tautulli`
- `overseerr`
- `linkding`

`apply` and `diff` remain targeted-only in this iteration. Fleet results are
discovery-oriented and are not written directly into `~/.lab/.env` because the
flat env key model is ambiguous when multiple hosts expose the same service.

The CLI remains a thin shim over the client.

## MCP and API Surface

The MCP tool and product API expose `extract.scan` within the normal one-tool-per-service model.

For `extract.scan`, omitting `params.uri` requests fleet discovery. Supplying
`params.uri` keeps the existing targeted behavior.

`extract.apply` and `extract.diff` are still CLI-only in this iteration. Those
mutating and diffing flows are not yet wired through the MCP or product API
dispatch surfaces.

Targeted scan responses now include both the new `target` metadata and the
legacy top-level `uri` field for compatibility. Fleet responses include
`target.mode = "fleet"` and omit the legacy `uri` field.

Browser-facing callers such as the web Setup page must request redacted scan
results so secret values are not delivered to the browser. Redacted responses
replace raw secret values with `secret_present`.

It must also participate in the normal discovery surfaces:

- `extract help`
- `lab://extract/actions`

## Safety

`extract` exists to save setup time, but it must default toward preserving existing config rather than overwriting it.
