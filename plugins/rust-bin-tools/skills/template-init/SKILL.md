---
name: template-init
description: Post-cargo-generate first-run setup. Walks through domain replacement and tooling bootstrap.
disable-model-invocation: false
---

## Usage

Run `/template-init` immediately after `cargo generate` completes in a new project.

## Steps

### 1. Confirm surface selection
Ask the user which surfaces were selected during generation (CLI / API / MCP / web).
Note any surfaces that were excluded — those directories won't exist and references to them should be skipped.

### 2. Replace the example domain
The generated project contains a demo `example` domain. Ask the user for their first real domain name (snake_case).
Then:
- Rename `crates/core/src/example.rs` → `crates/core/src/{domain}.rs`
- Rename `crates/core/src/example/` → `crates/core/src/{domain}/`
- Rename `crates/app/src/cli/example.rs` → `crates/app/src/cli/{domain}.rs` (if CLI surface)
- Rename `crates/app/src/api/services/example.rs` → `crates/app/src/api/services/{domain}.rs` (if API surface)
- Update all `mod example` / `use example` references in `lib.rs`, `main.rs`, and surface files
- Update the MCP tool registration in `crates/app/src/mcp/` (if MCP surface)

If the user doesn't have a domain name yet, skip this step and remind them to use `/add-domain` later.

### 3. Update README.md
Replace the template placeholder content with:
- Project name and one-sentence description (ask user)
- Which surfaces are active
- Quick-start commands appropriate to the surfaces

### 4. Bootstrap tooling
```bash
just bootstrap        # installs cargo-nextest, lefthook, taplo, etc.
just install-hooks    # wires lefthook git hooks
```

### 5. Sync llms.txt references
Invoke the `sync-stack-llms` subagent to analyze the actual Cargo.toml and package.json dependencies and generate any missing llms.txt reference files.

### 6. Verify
```bash
just lint             # should pass on the fresh scaffold
just test             # should pass
```

If either fails, surface the errors before proceeding.

### 7. Initial commit
```bash
git add -A
git commit -m "chore: initialize project from rust-bin template"
```

## Notes
- cargo-generate already handled: project-name substitution, surface file inclusion
- This skill handles: domain renaming, README, tooling, and first-run verification
- After this skill completes, use `/add-domain` for subsequent domains
