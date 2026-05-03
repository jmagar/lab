---
name: check-llms-drift
description: Audit the docs/references llms.txt cache for drift — stale files, orphaned entries, and (optionally) version-bumped upstream releases. Each *-llms.txt is self-describing via YAML frontmatter; this skill reads that frontmatter and cross-references it against Cargo.lock / package.json. Used by sync-stack-llms before generating new entries.
disable-model-invocation: false
---

The audit logic lives in a script — `scripts/check-llms-drift.py` — that does the mechanical work (frontmatter parsing across `docs/references/*-llms.txt`, lockfile cross-referencing, registry version queries). Don't re-implement any of this in conversation; just run the script and reason about the **findings**.

## Run it

```bash
.claude/skills/check-llms-drift/scripts/check-llms-drift.py
```

Useful flags:
- `--max-age-days N` — file age threshold for the STALE label (default 90)
- `--check-versions` — also query crates.io / npm for newer versions (slower, parallelized)
- `--quiet` / `-q` — suppress OK rows; only show drift
- `--help` — full usage

Exit codes: `0` no drift · `1` drift detected · `2` `docs/references/` missing · `3` bad arguments.

## Output contract

```
STALE       axum-llms.txt        last refreshed 2024-11-03 (175d ago)
ORPHAN      wiremock-llms.txt    not in Cargo.lock, Cargo.toml, or package.json
VERSION     tokio-llms.txt       pinned 1.36, latest 1.40 (minor, via crates)
OK          rmcp-llms.txt        last refreshed 2026-04-15 (13d ago)

Summary: 1 stale, 1 orphan, 1 version-drift, 30 ok
```

## Interpreting findings

| Label | What it means | Action |
|---|---|---|
| `STALE` | The file's `generated:` frontmatter date (or git mtime, if absent) is more than `--max-age-days` old. | Refresh via `manage-llms-txt refresh <name>`. |
| `ORPHAN` | The entry's name doesn't appear in any lockfile/manifest in the repo (Cargo.lock, Cargo.toml, package.json, pnpm-lock.yaml). | Either the dep was removed and the cached file is dead (`manage-llms-txt remove <owner/repo>`), OR the dep was never installed in the first place — confirm with `git log` on the file before deleting. |
| `VERSION` | Pinned version is at least one minor version behind the registry's `newest_version`. Patch-only diffs are not flagged. | Bump the dep, then refresh the llms.txt file so the cached docs reflect the new version. |
| `OK` | File is current and the dep is still pinned somewhere. | Nothing to do. |

## Notes for the agent

- **Don't fabricate findings** the script didn't emit. The script is the source of truth; you're a reader.
- **A finding is a flag, not a verdict.** ORPHAN can mean "remove this" OR "the dep was renamed" — read the manifest entry's `description` to decide.
- **VERSION findings need network access.** If `--check-versions` was passed but the script reports zero VERSION rows, it may mean either no drift OR the registries were unreachable. The other findings are still valid in either case.
- **The script is idempotent and side-effect-free.** Safe to run repeatedly during a session.

## Related

- `sync-stack-llms` agent — uses this skill before adding new entries, so the drift sweep happens automatically as part of its workflow.
- `manage-llms-txt` skill — the writer counterpart that creates / refreshes / removes the `*-llms.txt` files (and stamps the frontmatter this skill reads).
