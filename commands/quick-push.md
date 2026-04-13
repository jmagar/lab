---
allowed-tools: Bash, TodoWrite
description: Git add all, commit with Claude, and push to current/new feature branch
---

## Context

- Current branch: !`git branch --show-current`
- Git status: !`git status --short`
- Remote info: !`git remote -v | head -1`

## Your task

Work through these steps in order:

### 1. Orient
- Check if we're on a feature branch (not main/master)
- If on main/master, create a new feature branch with a descriptive name based on the changes
- Run `git diff --stat HEAD` to understand the scope of changes
- Run `git log --oneline -5` to understand recent commit conventions

### 2. Bump version (before staging)

Detect the project type and bump the version based on the commit message you'll generate.

**Bump rules** (based on the commit message prefix you'll draft in step 3):
- `feat!:` or `BREAKING CHANGE` → **major** (X+1.0.0)
- `feat` or `feat(...)` → **minor** (X.Y+1.0)
- Everything else (`fix`, `chore`, `refactor`, `test`, `docs`, etc.) → **patch** (X.Y.Z+1)

**Process:**
1. Read the current version from the primary manifest (first match: `Cargo.toml`, `package.json`, `pyproject.toml`)
2. Draft the commit message mentally (you'll finalize it in step 4)
3. Determine bump type from the commit prefix
4. Calculate the new version
5. **Update the version in ALL of these files that exist in the repo:**
   - `Cargo.toml` — `version = "X.Y.Z"` in `[package]`
   - `package.json` — `"version": "X.Y.Z"`
   - `pyproject.toml` — `version = "X.Y.Z"` in `[project]`
   - `.claude-plugin/plugin.json` — `"version": "X.Y.Z"`
   - `.codex-plugin/plugin.json` — `"version": "X.Y.Z"`
   - `gemini-extension.json` — `"version": "X.Y.Z"`
6. If Rust: run `cargo check` to update `Cargo.lock` (it records the version)
7. Report: `Version: X.Y.Z → A.B.C (bump type)` and list which files were updated

**Skip conditions:**
- Version is `0.0.0` or `0.0.1` (project not yet versioned)
- No manifest file found
- The `--no-bump` flag was passed as a skill argument

### 3. Update CHANGELOG.md (before staging)
If a `CHANGELOG.md` exists in the repo root:
- Find the most recently documented commit in the changelog (look for commit hashes in the table)
- Run `git log --oneline <last_documented_sha>..HEAD` to get undocumented commits
- If there are new commits, update the changelog:
  - Add new rows to the commit summary table (newest first)
  - Update the Highlights section with grouped summaries
  - Keep the existing structure and style
- If no CHANGELOG.md exists, skip this step

### 4. Stage, commit, and push
- Stage all changes with `git add .`
- Create a meaningful commit message following the repo's conventions
- Always include Claude's co-authorship signature
- Push to remote:
  - New branch: `git push -u origin <branch>`
  - Existing branch: `git push`

### 5. Post-push: save session context
After the push succeeds, invoke the `save-to-md` skill to capture session context to Neo4j and Qdrant.

The `save-to-md` skill handles most Neo4j work, but after it completes, **always** create these additional entities and relations to wire the commit graph:

**Entities to create:**
- One entity per commit pushed this session (type: `commit`), with observations: SHA, message, branch, timestamp, files changed
- The repo itself if not already in the graph (type: `repository`), with observations: remote URL, current branch
- The session document (type: `session_doc`), with observations: file path, collection embedded into, embed job ID

**Relations to create:**
```
commit  → repository    : PUSHED_TO
commit  → session_doc   : DOCUMENTED_IN
session_doc → repository : BELONGS_TO
commit  → commit        : PRECEDED_BY   (chain commits in push order, newest → oldest)
```

Use the actual SHA, branch name, repo name, and session doc path from the work done in steps 1–5. Do not use placeholders.

---

**Notes:**
- If creating a new branch, name it based on the changes (e.g., `feat/add-user-auth`, `fix/navbar-styling`)
- The changelog update is part of the commit — it goes in the same commit as the other changes
- End with a summary of what was pushed and the branch name
