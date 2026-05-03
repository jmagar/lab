#!/usr/bin/env bash
#
# sync-skills.sh — wire .agents/skills as a symlink to .claude/skills
#
# This project treats .claude/ as the canonical home for all agent
# resources (skills, subagents, hooks, MCP, commands). Codex and Gemini
# read skills from .agents/skills/, so we keep that path as a symlink
# back into .claude/skills/. One source of truth, no manual mirroring.
#
# Idempotent: safe to run repeatedly. Refuses to overwrite a real
# .agents/skills directory to avoid silent data loss.
#
# Usage: scripts/sync-skills.sh [--quiet]
#
# Exit codes:
#   0  symlink in place (created, refreshed, or already correct)
#   1  .claude/skills/ missing
#   2  .agents/skills exists as a real directory (manual move required)

set -euo pipefail

# Resolve repo root from the script location. The script lives at
# .claude/skills/sync-skills/scripts/, so the repo root is four
# levels up.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
cd "$REPO_ROOT"

QUIET=0
for arg in "$@"; do
  case "$arg" in
    --quiet|-q) QUIET=1 ;;
    --help|-h)
      sed -n '2,18p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: $(basename "$0") [--quiet]" >&2
      exit 64
      ;;
  esac
done

log() { [[ "$QUIET" -eq 1 ]] || echo "$@"; }
err() { echo "$@" >&2; }

if [[ ! -d .claude/skills ]]; then
  err "Error: .claude/skills/ does not exist — nothing to sync."
  err "  This project authors skills under .claude/skills/ and mirrors"
  err "  them to .agents/skills/ via this script. Create a skill there"
  err "  first."
  exit 1
fi

if [[ -e .agents/skills ]] && [[ ! -L .agents/skills ]]; then
  err "Error: .agents/skills exists as a real directory, not a symlink."
  err "  This script will not overwrite it (data-loss guard)."
  err
  err "  To resolve:"
  err "    1. Move the contents of .agents/skills/ into .claude/skills/"
  err "    2. Remove .agents/skills/"
  err "    3. Re-run this script"
  err
  err "  (.claude/ is the source of truth — see CLAUDE.md.)"
  exit 2
fi

mkdir -p .agents

# Detect whether the symlink already points to the right place.
# Using a relative target so the link survives a clone-and-move.
TARGET="../.claude/skills"
if [[ -L .agents/skills ]]; then
  CURRENT="$(readlink .agents/skills)"
  if [[ "$CURRENT" == "$TARGET" ]]; then
    log "✓ .agents/skills -> .claude/skills (already in sync)"
    exit 0
  fi
fi

rm -f .agents/skills
ln -s "$TARGET" .agents/skills
log "✓ .agents/skills -> .claude/skills (Codex + Gemini read from here)"
