#!/usr/bin/env bash
#
# session-context.sh — SessionStart hook helper.
#
# Prints active project context (beads, exec-plans, worktrees, PRs,
# product specs, sessions, ADRs) at the start of every Claude Code
# session. Each section explicitly prints "(none)" when empty so the
# session log doesn't have mysterious blank gaps.
#
# Wired into .claude/settings.json under hooks.SessionStart.

set -uo pipefail

# Print a section header and either the captured output or a "(none)"
# fallback. The fallback fires when the command produced no output OR
# exited non-zero — both are reasonable empty-states.
section() {
  local title="$1"; shift
  local empty_msg="${EMPTY_MSG:-(none)}"
  local out
  out="$("$@" 2>/dev/null || true)"
  echo "=== $title ==="
  if [[ -z "$out" ]]; then
    echo "$empty_msg"
  else
    echo "$out"
  fi
  echo
}

# --- bd ---------------------------------------------------------------

bd_inprogress() { bd list -s in_progress --no-pager 2>/dev/null | head -10; }
bd_ready()      { bd ready --no-pager 2>/dev/null | head -10; }

# `bd list` prints "No issues found." when empty — strip that so
# section() can detect the actual empty state.
bd_inprogress_filtered() {
  local out
  out="$(bd_inprogress)"
  [[ "$out" == "No issues found." ]] && return 0
  echo "$out"
}

bd_ready_filtered() {
  local out
  out="$(bd_ready)"
  [[ "$out" == "No issues found." ]] && return 0
  echo "$out"
}

# --- exec-plans -------------------------------------------------------

list_dir() {
  local dir="$1"
  [[ -d "$dir" ]] || return 0
  ls -1 "$dir" 2>/dev/null | grep -v '^\.gitkeep$' | head -10
}

active_plans()   { list_dir docs/exec-plans/active; }
proposed_plans() { list_dir docs/exec-plans/proposed; }

# --- worktrees --------------------------------------------------------

# `git worktree list` always prints the main checkout first; trim it.
non_main_worktrees() {
  command -v git >/dev/null 2>&1 || return 0
  git worktree list 2>/dev/null | tail -n +2 | head -10
}

# --- gh ---------------------------------------------------------------

open_prs() {
  if ! command -v gh >/dev/null 2>&1; then
    EMPTY_MSG="(gh not installed)" return 0
  fi
  gh pr list --limit 10 2>/dev/null
}

# --- docs (always have at least one entry if the dir exists) ---------

list_glob() {
  local pattern="$1"
  local exclude="${2:-}"
  # shellcheck disable=SC2086 # intentional glob expansion
  if [[ -n "$exclude" ]]; then
    ls -1t $pattern 2>/dev/null | grep -v "$exclude" | head -5
  else
    ls -1t $pattern 2>/dev/null | head -5
  fi
}

product_specs()  { list_glob 'docs/product-specs/*.md' index; }
recent_sessions() { list_glob 'docs/sessions/*.md' README | head -3; }
recent_adrs()     { list_glob 'docs/design-docs/2*.md' '' | head -3; }

# --- render -----------------------------------------------------------

section "In-progress beads"      bd_inprogress_filtered
section "Ready beads"            bd_ready_filtered
section "Active exec-plans"      active_plans
section "Proposed exec-plans"    proposed_plans

EMPTY_MSG="(no extra worktrees)" section "Worktrees (excluding main)" non_main_worktrees

# Override empty-message for gh based on what's installed.
if command -v gh >/dev/null 2>&1; then
  EMPTY_MSG="(no open PRs or gh not authenticated)" section "Open PRs" open_prs
else
  section "Open PRs" : <<<""
fi

section "Product specs"   product_specs
section "Recent sessions" recent_sessions
section "Recent ADRs"     recent_adrs
