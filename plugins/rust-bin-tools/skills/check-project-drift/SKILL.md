---
name: check-project-drift
description: Five drift checks for detecting where the codebase has outpaced documentation — knowledge map integrity, spec.md vs xtask parity, exec-plan lifecycle drift, stale active plans, ADR coverage gaps. Preloaded by drift-detector.
user-invocable: false
---

Run all four checks in order. Report everything — do not fix.

## Check 1 — Knowledge map integrity (CLAUDE.md)

Extract every file path from CLAUDE.md's knowledge map. Verify each exists:
```bash
test -e {path} && echo "OK {path}" || echo "MISSING {path}"
```

Also scan for undocumented additions:
- `.md` files in `docs/` not listed in the map
- New `xtask/src/` subcommand files not reflected in spec.md or CLAUDE.md

Labels: `MISSING` (referenced but gone), `UNDOCUMENTED` (exists, not in map).

## Check 2 — spec.md vs xtask parity

Subcommands from source:
```bash
grep -r "\"check-\|\"gen-\|\"audit-\|\"verify-" xtask/src/ 2>/dev/null | grep -oE '"[a-z][a-z-]+"' | sort -u
```

Subcommands from spec.md:
```bash
grep -oE 'cargo xtask [a-z][a-z-]+' spec.md | awk '{print $3}' | sort -u
```

Labels: `UNDOCUMENTED` (in source, missing from spec.md), `PHANTOM` (in spec.md, gone from source).

## Check 3 — Exec-plan lifecycle drift

Plans should flow `proposed/` → `active/` → `completed/`. Each transition can drift:

### 3a. Proposed plans where work has already started (should be `active/`)

For each file in `docs/exec-plans/proposed/`:
1. Extract the slug (filename without `.md`)
2. Check git log for activity referencing the slug or its keywords:
   ```bash
   git log --oneline --since="60 days ago" | grep -iE "{slug-keywords}"
   ```
3. Check whether files the plan says to create now exist
4. Check whether a branch matching the slug exists or was recently merged

Label: `LIKELY_ACTIVE` (with evidence — commit SHAs, file paths).

### 3b. Active plans where work appears done (should be `completed/`)

For each file in `docs/exec-plans/active/`:
1. Extract the target files, crates, or features it describes
2. Check if the work looks already complete:
   - Files the plan says to create already exist with non-stub content
   - The described problem is already solved in the codebase
   - Commit messages contain "closes", "completes", or the plan's slug
3. Check for stale references — paths in the plan that no longer exist

Labels: `LIKELY_DONE` (with evidence), `STALE_REFS` (references paths that no longer exist).

### 3c. Long-stalled proposed plans

Flag plans in `proposed/` older than 90 days with no matching git activity. These should either be moved to `active/`, abandoned (annotated with `Status: abandoned`), or restated.

Label: `STALLED` (age + no activity).

## Check 4 — ADR coverage gaps

Read `docs/design-docs/index.md`. Then check whether major framework choices and architectural patterns have a corresponding ADR:
```bash
grep -l "axum\|rmcp\|tokio\|figment\|utoipa" docs/design-docs/*.md 2>/dev/null
```

Look for:
- Significant dependencies in `spec.md § Framework Choices` with no ADR
- Core patterns (two-tier errors, action+params contract, single-port topology) with no recorded rationale

Label: `NO_ADR`.

## Output format

```
=== Check 1: Knowledge Map ===
MISSING       docs/sessions/README.md        (referenced in CLAUDE.md, file gone)
UNDOCUMENTED  docs/research/2025-03-cve.md   (exists, not in knowledge map)

=== Check 2: spec.md vs xtask ===
UNDOCUMENTED  check-msrv                     (in source, not in spec.md)
PHANTOM       check-old-lint                 (in spec.md, not in source)

=== Check 3: Exec-plan lifecycle ===
LIKELY_ACTIVE proposed/2026-04-15-cache-layer.md   (3 commits matching "cache-layer", target file crates/core/src/cache.rs exists)
LIKELY_DONE   active/2025-01-15-auth-flow.md       (auth middleware exists at crates/app/src/api/auth/)
STALE_REFS    active/2025-02-20-mcp-tools.md       (references crates/mcp/ which doesn't exist)
STALLED       proposed/2025-09-01-search-ui.md     (94 days old, no git activity matching slug)

=== Check 4: ADR Coverage ===
NO_ADR        axum                           (framework choice, no ADR in design-docs/)
NO_ADR        action+params contract         (core pattern, no ADR found)

=== Summary ===
{n} knowledge map issues, {n} xtask parity issues, {n} stale plans, {n} ADR gaps
```
