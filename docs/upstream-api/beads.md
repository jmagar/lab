# Beads Source Contract

Retrieved: 2026-05-01

Sources:
- https://gastownhall.github.io/beads/
- local `bd --help`
- local `bd ready --help`
- local `bd list --help`

Beads is a git/Dolt-backed issue tracker exposed through the `bd` CLI. Public documentation describes `bd` as the stable operator interface and recommends JSON output for agents. Lab therefore implements Beads as a local CLI-backed service instead of binding directly to Dolt tables.

## V1 Actions

| Action | Contract | Hosted posture |
|---|---|---|
| `contract.status` | returns the Lab/Beads integration contract | safe |
| `health.status` | checks `bd` availability and lightweight workspace status | local-only |
| `version.get` | runs `bd --json --readonly version` | local-only |
| `context.get` | runs `bd --json --readonly context` | local-only |
| `status.summary` | runs `bd --json --readonly status` | local-only |
| `issue.list` | runs bounded `bd --json --readonly list` | local-only |
| `issue.ready` | runs bounded `bd --json --readonly ready` | local-only |
| `issue.show` | runs `bd --json --readonly show <id>` | local-only |
| `graph.show` | runs `bd --json --readonly graph <id>` | local-only |

## Deferred

Write operations are deferred: create, update, close, reopen, comment, dependency mutation, raw SQL, Dolt push/pull/commit, branch operations, import/export, and direct Dolt database access.

## Security

Every Lab v1 Beads command passes `--readonly`. `BEADS_BIN` can override the executable path, but no caller-supplied command arguments are executed outside the allowlisted action catalog.
