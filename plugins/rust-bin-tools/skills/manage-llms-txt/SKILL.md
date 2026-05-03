---
name: manage-llms-txt
description: Manage llms.txt reference files in docs/references/. Create new entries from GitHub repos, refresh existing ones, list current entries, or remove obsolete ones — all via the bundled script. Use this skill whenever the user mentions creating an llms.txt for a specific GitHub repo (`/manage-llms-txt jmagar/lab`, `add a docs reference for owner/repo`, `pull docs for nextjs-form-validator`), refreshing the references (`refresh the llms.txt files`, `update our docs cache`, `re-pull all the references`), or auditing what's there (`what llms.txt files do we have`, `list the references`). Also handles removal (`remove the llms.txt for foo/bar`). The script is the source of truth — invoke it; don't re-derive its logic in conversation.
---

This skill wraps `repomix` (which fetches GitHub content as an llms.txt snapshot) under one ergonomic interface for create / refresh / list / remove. Each `*-llms.txt` file is **self-describing**: its YAML frontmatter records `repo`, `include`, `description`, and `generated` date, so the directory itself is the manifest.

## Run

```bash
.claude/skills/manage-llms-txt/scripts/manage-llms-txt.py <command> [args]
```

## Modes

### Create — `manage-llms-txt <owner/repo>`

Generates a fresh `<basename>-llms.txt` from the GitHub repo, prepends YAML frontmatter (`repo`, `include`, `description`, `generated`), and rebuilds `docs/references/index.md`. The basename defaults to the repo's name (the part after `/`); the description defaults to the GitHub repo's description (fetched via the public API); the include glob defaults to `README.md,docs/**/*.md,docs/**/*.mdx,src/**`.

Override any of those:

```bash
manage-llms-txt jmagar/lab                                 # all defaults
manage-llms-txt jmagar/lab --include "README.md,docs/**"   # narrower scope
manage-llms-txt jmagar/lab --description "Our shared lab"  # custom blurb
manage-llms-txt jmagar/lab --name lab-platform             # different filename
```

The explicit `create` subcommand works too: `manage-llms-txt create jmagar/lab ...`.

The script refuses to overwrite an existing entry (use `refresh` for that) and refuses if `repomix` produces an empty file. Frontmatter is only written *after* a successful fetch — failed creates leave the directory untouched.

### Refresh — `manage-llms-txt refresh [target]`

```bash
manage-llms-txt refresh                # all entries
manage-llms-txt refresh jmagar/lab     # by owner/repo (looked up via frontmatter)
manage-llms-txt refresh lab-llms.txt   # by filename
manage-llms-txt refresh lab            # by basename (the `-llms.txt` is implied)
```

Refresh re-runs `repomix` against the file's stored `repo` + `include`, then re-stamps the frontmatter (preserving `repo` / `include` / `description`, updating `generated` to today). The body is replaced; the header survives.

### List — `manage-llms-txt list`

Pretty-prints the discovered manifest (filename · repo · description · generated date), one entry per line, with a count at the end. Entries are discovered by scanning `docs/references/*-llms.txt` for valid frontmatter.

### Remove — `manage-llms-txt remove <owner/repo>`

Asks for confirmation, then deletes the `.txt` file and rebuilds the index. Pass `--yes`/`-y` to skip the prompt (useful from scripts; not recommended interactively).

### Rebuild index — `manage-llms-txt rebuild-index`

Rewrites `docs/references/index.md` from the discovered frontmatter. Mostly useful after manually editing files; create / refresh / remove already rebuild it.

## When to invoke

Trigger this skill whenever the user wants to:

- Add a new docs cache for a GitHub repo (e.g. "pull docs for `tokio-rs/console`")
- Refresh stale references (e.g. "refresh the llms.txt files", "update our docs cache")
- Audit existing references (e.g. "what llms.txt files do we have")
- Clean up obsolete entries (e.g. "drop the llms.txt for `foo/bar`")

Don't try to re-implement the repomix invocation in conversation — the script handles all that, including the gotchas (description fetch via GitHub API, empty-file detection, atomic header preservation across refreshes, index rebuild).

## Exit codes

- `0` — success
- `1` — repomix failed, or `remove` was declined
- `2` — bad arguments, or target not found in manifest
- `3` — `npx` / `repomix` not available

## Related

- `check-llms-drift` skill — audit which entries are stale, orphaned, or version-drifted (the read-only counterpart to this skill's mutating operations)
- `sync-stack-llms` agent — uses both `check-llms-drift` and this skill to keep the references in sync with the project's actual dependencies
