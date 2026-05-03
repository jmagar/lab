#!/usr/bin/env bash
#
# sync-claude-mds.sh — mirror every CLAUDE.md to AGENTS.md and GEMINI.md
#
# This project treats CLAUDE.md as the canonical source of agent
# instructions. Codex reads AGENTS.md, Gemini reads GEMINI.md — same
# content, different filenames. Rather than maintain three copies, we
# keep CLAUDE.md as the real file and symlink the other two beside it.
#
# Walks the entire repo (skipping vendor and build dirs) so that
# nested CLAUDE.md files (e.g. docs/CLAUDE.md, package-level
# CLAUDE.mds) are mirrored too — each gets its own AGENTS.md and
# GEMINI.md siblings.
#
# Idempotent: safe to run repeatedly. Refuses to overwrite a real
# AGENTS.md or GEMINI.md file to avoid silent data loss; replaces
# stale symlinks pointing elsewhere.
#
# Usage: scripts/sync-claude-mds.sh [--quiet] [--dry-run]
#
# Exit codes:
#   0  every CLAUDE.md has correct AGENTS.md + GEMINI.md siblings
#   1  no CLAUDE.md found in the repo
#   2  one or more siblings exist as real files (manual move required)

set -euo pipefail

# Resolve repo root from the script location. The script lives at
# .claude/skills/sync-claude-mds/scripts/, so the repo root is four
# levels up.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --quiet|-q) QUIET=1 ;;
    --dry-run|-n) DRY_RUN=1 ;;
    --help|-h)
      sed -n '2,21p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: $(basename "$0") [--quiet] [--dry-run]" >&2
      exit 64
      ;;
  esac
done

log()    { [[ "$QUIET" -eq 1 ]] || echo "$@"; }
err()    { echo "$@" >&2; }
action() { [[ "$DRY_RUN" -eq 1 ]] && echo "would: $*" || eval "$*"; }

# Find every CLAUDE.md, skipping vendor / build / VCS noise.
# Using -prune so we don't even descend into them.
mapfile -t CLAUDE_FILES < <(
  find . \
    \( -path ./.git -o \
       -path ./node_modules -o \
       -path '*/node_modules' -o \
       -path ./target -o \
       -path '*/target' -o \
       -path ./.next -o \
       -path '*/.next' -o \
       -path ./dist -o \
       -path '*/dist' -o \
       -path ./build -o \
       -path '*/build' \
    \) -prune -o \
    -type f -name CLAUDE.md -print | sort
)

if [[ ${#CLAUDE_FILES[@]} -eq 0 ]]; then
  err "Error: no CLAUDE.md found anywhere in the repo."
  exit 1
fi

# Track failures so we exit with the right code at the end without
# bailing on the first occurrence — better UX to surface every
# problem in one pass.
FAIL_COUNT=0
LINK_COUNT=0
NOOP_COUNT=0

mirror_one() {
  local claude_path="$1"
  local dir="$(dirname "$claude_path")"
  local name="$2"  # AGENTS.md or GEMINI.md
  local target_path="$dir/$name"

  # If a real file (not a symlink) is sitting here, refuse to clobber
  # it. The user must move its content into CLAUDE.md first.
  if [[ -e "$target_path" ]] && [[ ! -L "$target_path" ]]; then
    err "Error: $target_path exists as a real file."
    err "  Will not overwrite (data-loss guard). Move its content into"
    err "  $claude_path, delete $target_path, then re-run."
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi

  # Already a symlink — check if it points at CLAUDE.md (relative).
  if [[ -L "$target_path" ]]; then
    local current
    current="$(readlink "$target_path")"
    if [[ "$current" == "CLAUDE.md" ]]; then
      log "  ✓ $target_path -> CLAUDE.md (already in sync)"
      NOOP_COUNT=$((NOOP_COUNT + 1))
      return
    fi
    # Stale symlink pointing elsewhere — replace it.
    action "rm -f \"$target_path\""
  fi

  action "ln -s CLAUDE.md \"$target_path\""
  log "  ✓ $target_path -> CLAUDE.md"
  LINK_COUNT=$((LINK_COUNT + 1))
}

log "Found ${#CLAUDE_FILES[@]} CLAUDE.md file(s):"
for claude_path in "${CLAUDE_FILES[@]}"; do
  log "→ $claude_path"
  mirror_one "$claude_path" AGENTS.md
  mirror_one "$claude_path" GEMINI.md
done

log
log "Summary: $LINK_COUNT created/refreshed · $NOOP_COUNT already in sync · $FAIL_COUNT blocked"

if [[ "$FAIL_COUNT" -gt 0 ]]; then
  exit 2
fi
