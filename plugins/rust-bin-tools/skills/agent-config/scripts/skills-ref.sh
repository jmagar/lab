#!/usr/bin/env bash
#
# skills-ref.sh — wrapper around `npx skills-ref` with batch / --all
# discovery for skills under .claude/skills/.
#
# Upstream:  https://www.npmjs.com/package/skills-ref
# Subcommands match upstream:
#   validate <skill_path>              — single skill, "Valid skill: ..." on success
#   read-properties <skill_path>       — single skill, JSON {name, description}
#   to-prompt <skill_paths...>         — multi-skill, XML <available_skills> block
#
# Wrapper extras:
#   --all                              — discover every SKILL.md under .claude/skills/
#                                        and feed it into the requested subcommand
#   validate-all                       — shortcut for `validate --all`; aggregates
#                                        results, exits non-zero if any failed
#   read-properties-all                — shortcut for `read-properties --all`;
#                                        emits a single JSON array of all skills
#
# Discovery scope: `.claude/skills/<name>/SKILL.md`. The `.agents/skills/`
# directory is a symlink to `.claude/skills/`, so it's not re-scanned.
#
# Usage:
#   skills-ref.sh <command> [args]
#   skills-ref.sh validate <path>
#   skills-ref.sh validate --all
#   skills-ref.sh validate-all                  # alias
#   skills-ref.sh read-properties <path>
#   skills-ref.sh read-properties --all         # JSON array
#   skills-ref.sh read-properties-all           # alias
#   skills-ref.sh to-prompt <path>...
#   skills-ref.sh to-prompt --all               # all skills in one prompt block
#
# Exit codes:
#   0  success (or all batch items succeeded)
#   1  one or more validations failed (validate / validate-all)
#   2  npx / skills-ref invocation failed
#  64  bad arguments

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
SKILLS_DIR="$REPO_ROOT/.claude/skills"

# Claude Code extends the open agent-skills frontmatter allowlist with
# fields that are valid only on Claude. `skills-ref` enforces the strict
# upstream allowlist (allowed-tools, compatibility, description, license,
# metadata, name) and rejects anything else. We forgive these specific
# extensions because they're documented Claude-skill features:
CLAUDE_SKILL_EXTENSIONS=(
  "user-invocable"            # boolean — controls whether the user can invoke via /name
  "disable-model-invocation"  # boolean — newer name for the inverse concept
)
STRICT=0  # set by --strict on validate / validate-all to disable forgiveness

# Returns 0 if every name in arg1 (comma-separated) is in CLAUDE_SKILL_EXTENSIONS.
# Returns 1 if any name is foreign (a real validation failure beyond Claude's surface).
all_claude_approved() {
  local fields="$1" name approved
  IFS=',' read -ra names <<< "$fields"
  for raw in "${names[@]}"; do
    name="$(echo "$raw" | xargs)"  # trim whitespace
    [[ -z "$name" ]] && continue
    approved=0
    for ext in "${CLAUDE_SKILL_EXTENSIONS[@]}"; do
      [[ "$name" == "$ext" ]] && { approved=1; break; }
    done
    [[ "$approved" -eq 0 ]] && return 1
  done
  return 0
}

# Run skills-ref validate with Claude-extension forgiveness.
# Echoes one of:
#   "OK"                              — validates per upstream's strict rules
#   "OK_CLAUDE_EXT:<comma-fields>"   — fails upstream, but only on forgiven Claude fields
#   "FAIL:<error-text>"              — fails for some other reason (truly invalid)
validate_with_forgiveness() {
  local skill_dir="$1" output rc fields
  output="$(npx --yes skills-ref validate "$skill_dir" 2>&1)" && rc=0 || rc=$?

  if [[ "$rc" -eq 0 ]]; then
    echo "OK"
    return 0
  fi

  # If --strict, never forgive.
  if [[ "$STRICT" -eq 1 ]]; then
    echo "FAIL:$output"
    return 1
  fi

  # Extract "Unexpected fields in frontmatter: X, Y, Z." (period after Z).
  fields="$(echo "$output" | grep -oE 'Unexpected fields in frontmatter: [^.]+' | sed 's/^Unexpected fields in frontmatter: //' | head -1)"

  if [[ -n "$fields" ]] && all_claude_approved "$fields"; then
    echo "OK_CLAUDE_EXT:$fields"
    return 0
  fi

  echo "FAIL:$output"
  return 1
}

# ---- helpers --------------------------------------------------------

usage() { sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

discover_skills() {
  # Print absolute paths to every SKILL.md under .claude/skills/, one per line.
  if [[ ! -d "$SKILLS_DIR" ]]; then
    echo "Error: $SKILLS_DIR does not exist" >&2
    return 1
  fi
  find "$SKILLS_DIR" -mindepth 2 -maxdepth 2 -name SKILL.md | sort
}

require_npx() {
  if ! command -v npx >/dev/null; then
    echo "Error: npx is required (install Node.js)." >&2
    exit 2
  fi
}

# ---- subcommands ----------------------------------------------------

cmd_validate_one() {
  require_npx
  local target="$1" result rc
  # If a SKILL.md was passed, walk up to the directory.
  [[ -f "$target" ]] && target="$(dirname "$target")"
  result="$(validate_with_forgiveness "$target")" && rc=0 || rc=$?
  case "$result" in
    OK)
      echo "Valid skill: $target" ;;
    OK_CLAUDE_EXT:*)
      local fields="${result#OK_CLAUDE_EXT:}"
      echo "Valid skill (Claude extension: $fields): $target" ;;
    FAIL:*)
      echo "${result#FAIL:}" >&2 ;;
  esac
  return "$rc"
}

cmd_validate_all() {
  require_npx
  local ok=0 ok_ext=0 fail=0 result
  while IFS= read -r skill_md; do
    local skill_dir
    skill_dir="$(dirname "$skill_md")"
    result="$(validate_with_forgiveness "$skill_dir")" || true
    case "$result" in
      OK)
        ok=$((ok + 1))
        echo "  ✓ $(basename "$skill_dir")" ;;
      OK_CLAUDE_EXT:*)
        ok_ext=$((ok_ext + 1))
        echo "  ✓ $(basename "$skill_dir") (Claude: ${result#OK_CLAUDE_EXT:})" ;;
      FAIL:*)
        fail=$((fail + 1))
        echo "  ✗ $(basename "$skill_dir")"
        echo "${result#FAIL:}" | sed 's/^/      /' ;;
    esac
  done < <(discover_skills)

  echo
  if [[ "$ok_ext" -gt 0 ]]; then
    echo "Summary: $ok strict-valid · $ok_ext valid-with-Claude-extensions · $fail invalid"
  else
    echo "Summary: $ok valid · $fail invalid"
  fi

  if [[ "$ok_ext" -gt 0 ]] && [[ "$STRICT" -eq 0 ]]; then
    echo
    echo "Note: $ok_ext skill(s) use Claude-only frontmatter (user-invocable or"
    echo "disable-model-invocation) which the open agent-skills standard rejects."
    echo "These are intentionally Claude-only and are forgiven by default."
    echo "Pass --strict to fail on them too."
  fi
  [[ "$fail" -eq 0 ]]
}

cmd_read_one() {
  require_npx
  local target="$1"
  npx --yes skills-ref read-properties "$target"
}

cmd_read_all() {
  require_npx
  # Emit a JSON array of all skills' properties.
  local first=1
  echo "["
  while IFS= read -r skill_md; do
    local skill_dir
    skill_dir="$(dirname "$skill_md")"
    local props
    props="$(npx --yes skills-ref read-properties "$skill_dir" 2>/dev/null)"
    if [[ -z "$props" ]]; then continue; fi
    if [[ "$first" -eq 1 ]]; then
      first=0
    else
      echo ","
    fi
    # Indent each line of the JSON object two spaces for readability.
    echo "$props" | sed 's/^/  /'
  done < <(discover_skills)
  echo
  echo "]"
}

cmd_to_prompt() {
  require_npx
  if [[ "${1:-}" == "--all" ]]; then
    mapfile -t paths < <(discover_skills | xargs -n1 dirname)
    [[ ${#paths[@]} -eq 0 ]] && { echo "No skills found in $SKILLS_DIR" >&2; return 0; }
    npx --yes skills-ref to-prompt "${paths[@]}"
  else
    [[ $# -eq 0 ]] && { echo "Usage: $(basename "$0") to-prompt <path>... | --all" >&2; exit 64; }
    npx --yes skills-ref to-prompt "$@"
  fi
}

# ---- arg dispatch ---------------------------------------------------

case "${1:-}" in
  --help|-h|help|"")
    usage; exit 0 ;;
  validate)
    shift
    # Allow --strict before or after the path.
    [[ "${1:-}" == "--strict" ]] && { STRICT=1; shift; }
    if [[ "${1:-}" == "--all" ]]; then
      [[ "${2:-}" == "--strict" ]] && STRICT=1
      cmd_validate_all
    else
      [[ "${2:-}" == "--strict" ]] && STRICT=1
      [[ $# -eq 0 ]] && { echo "Usage: $(basename "$0") validate <path> [--strict] | --all [--strict]" >&2; exit 64; }
      cmd_validate_one "$1"
    fi
    ;;
  validate-all)
    shift
    [[ "${1:-}" == "--strict" ]] && STRICT=1
    cmd_validate_all
    ;;
  read-properties)
    shift
    if [[ "${1:-}" == "--all" ]]; then
      cmd_read_all
    else
      [[ $# -eq 0 ]] && { echo "Usage: $(basename "$0") read-properties <path> | --all" >&2; exit 64; }
      cmd_read_one "$1"
    fi
    ;;
  read-properties-all)
    cmd_read_all
    ;;
  to-prompt)
    shift
    cmd_to_prompt "$@"
    ;;
  *)
    echo "Unknown command: $1" >&2
    echo >&2
    usage >&2
    exit 64
    ;;
esac
