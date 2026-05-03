# `manage-llms-txt` skill

**Last updated:** 2026-04-28

Unified manager for the project's `docs/references/*-llms.txt` reference cache. Wraps `repomix` (the upstream content fetcher) under one ergonomic interface, so creating, refreshing, listing, and removing entries are all single commands instead of multi-step hand-edits.

The cache is **self-describing**: each `*-llms.txt` file carries its own YAML frontmatter (`repo`, `include`, `description`, `generated`). The directory itself is the manifest — no separate manifest file to drift out of sync.

## Directory layout

```
manage-llms-txt/
├── SKILL.md                              ← description + invocation modes
├── README.md (this file)
└── scripts/
    └── manage-llms-txt.py                ← all five operations
```

## Why this skill exists

Previously, adding a new llms.txt entry required:

1. Run `repomix` (or `just generate-llms`) with the right flags
2. Hand-edit a `MANIFEST` heredoc inside `scripts/refresh-llms.sh`
3. Re-sort the heredoc alphabetically
4. Rebuild `docs/references/index.md` from the heredoc

Every step had failure modes: forgetting to update the manifest, breaking heredoc syntax, getting the include glob wrong, leaving the index out of sync. This skill collapses all four into `manage-llms-txt <owner/repo>`, atomic on success and a no-op on failure, with the manifest information embedded in the file itself instead of an external bash script.

## Operations

### Create

```bash
manage-llms-txt jmagar/lab                                 # all defaults
manage-llms-txt jmagar/lab --include "README.md,docs/**"   # narrower fetch
manage-llms-txt jmagar/lab --description "Our shared lab"  # custom blurb
manage-llms-txt jmagar/lab --name lab-platform             # custom filename basename
manage-llms-txt create jmagar/lab                          # explicit subcommand
```

The script:

1. Validates the `owner/repo` shape and ensures no duplicate file already exists
2. Derives the filename from the repo's basename (e.g. `jmagar/lab` → `lab-llms.txt`)
3. Auto-fetches the GitHub repo description via `api.github.com/repos/<owner/repo>`; falls back to `"<name> reference"` if the API is unreachable
4. Runs `repomix` with the include glob (default `README.md,docs/**/*.md,docs/**/*.mdx,src/**`)
5. **Only on success** prepends YAML frontmatter to the file and rebuilds the index — failed fetches leave the directory untouched

### Refresh

```bash
manage-llms-txt refresh                # all entries
manage-llms-txt refresh jmagar/lab     # by owner/repo (looked up via frontmatter)
manage-llms-txt refresh lab            # by basename (-llms.txt is appended)
manage-llms-txt refresh lab-llms.txt   # by full filename
```

Refresh re-runs `repomix` against the file's stored `repo` + `include`, then re-stamps the frontmatter (preserving the original `repo` / `include` / `description`, updating `generated` to today). The body is replaced; the header survives. Per-entry refresh writes atomically — a failed refresh of one file doesn't affect any other.

### List

```bash
manage-llms-txt list
```

Discovers every `*-llms.txt` file in `docs/references/`, parses each one's frontmatter, and prints `filename · repo · description · (generated)`. Useful for spotting what's already covered before adding a new entry, or grepping by description.

### Remove

```bash
manage-llms-txt remove jmagar/lab        # asks for confirmation
manage-llms-txt remove jmagar/lab --yes  # skip prompt (CI / scripted usage)
```

Deletes the `.txt` file in `docs/references/` and rebuilds the index. The `--yes` flag exists for automation but interactive use should leave it off — accidentally deleting the wrong reference is recoverable from git but annoying.

### Rebuild index

```bash
manage-llms-txt rebuild-index
```

Rewrites `docs/references/index.md` from the discovered frontmatter. Mostly useful after manual file edits; the create / refresh / remove paths already rebuild it automatically.

## Atomic-update guarantees

The script is structured so partial failures don't corrupt anything:

- **Create**: validates first, fetches to a tmp file, atomically renames into place with frontmatter prepended. If any earlier step fails, no file is created.
- **Refresh**: re-runs `repomix` to a tmp file, then atomically replaces the original with the new content + preserved frontmatter. On failure, the original is untouched.
- **Remove**: confirms first, deletes the file second, rebuilds index third. If the user aborts at the prompt (exit 1), nothing is touched.

## Defaults — and when to override them

| Default | Override flag | When to override |
|---|---|---|
| Filename basename = repo basename | `--name <basename>` | Repo is generically named (e.g. `jmagar/cli` would want `--name jmagar-cli` to avoid a generic `cli-llms.txt`) |
| Include glob = `README.md,docs/**/*.md,docs/**/*.mdx,src/**` | `--include "<glob>"` | Repo organizes content differently — see the SKILL.md "Useful include-glob templates" table |
| Description = GitHub repo description | `--description "<text>"` | GitHub description is empty / outdated, or you want a project-specific blurb |

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | repomix failed; or `remove` was declined at the prompt |
| `2` | Bad arguments; or refresh/remove target not in the manifest |
| `3` | `npx` / `repomix` not on PATH |

Distinct codes let CI gates and the `sync-stack-llms` agent branch on the failure mode (e.g. retry on 1, fix arguments on 2, install tooling on 3).

## Sibling skills and agents

| Sibling | Relationship |
|---|---|
| `check-llms-drift` skill | Read-only audit (STALE / ORPHAN / VERSION). Run it BEFORE this skill's mutating operations to know what needs fixing. It reads the same frontmatter this skill writes. |
| `sync-stack-llms` agent | Loads both this skill and `check-llms-drift` to keep the references in sync with `Cargo.toml` / `package.json` automatically. |

## Maintenance

The script's logic is **all in one file** (`scripts/manage-llms-txt.py`) — Python stdlib only, no dependencies beyond `npx` (for repomix) and Python 3.10+. The frontmatter format is a tiny YAML subset (only flat key/value pairs, optional double-quoting); see `parse_frontmatter()` and `format_frontmatter()` in the script.

When `repomix`'s CLI surface changes, update the `_run([...])` invocation in `cmd_create()` / `cmd_refresh()` to match. As of writing, the invocation matches the include-glob convention used by the existing `*-llms.txt` snapshots.
