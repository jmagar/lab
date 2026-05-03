#!/usr/bin/env bash
#
# refresh-schemas.sh — re-fetch the three upstream-published JSON
# Schemas used by validate-settings.sh.
#
# The five plugin-manifest schemas (claude-plugin, claude-marketplace,
# codex-plugin, codex-marketplace, gemini-extension) are SYNTHESIZED
# in-repo from `plugin-matrix.md` and are NOT touched by this script.
# Edit them by hand when the matrix's field tables change.
#
# Usage: refresh-schemas.sh [--quiet]
#
# Exit codes:
#   0  all schemas refreshed
#   1  one or more fetches failed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_DIR="$(cd "$SCRIPT_DIR/../references/schemas" && pwd)"

QUIET=0
for arg in "$@"; do
  case "$arg" in
    --quiet|-q) QUIET=1 ;;
    --help|-h)
      sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *)
      echo "Unknown argument: $arg" >&2
      exit 64 ;;
  esac
done

log() { [[ "$QUIET" -eq 1 ]] || echo "$@"; }
err() { echo "$@" >&2; }

# url|local-filename
MANIFEST=$(cat <<'EOF'
https://developers.openai.com/codex/config-schema.json|codex-config.schema.json
https://raw.githubusercontent.com/google-gemini/gemini-cli/main/schemas/settings.schema.json|gemini-settings.schema.json
https://json.schemastore.org/claude-code-settings.json|claude-settings.schema.json
https://json.schemastore.org/claude-code-plugin-manifest.json|claude-plugin.schema.json
https://json.schemastore.org/claude-code-marketplace.json|claude-marketplace.schema.json
EOF
)

cd "$SCHEMA_DIR"

FAIL=0
log "Refreshing upstream schemas in $SCHEMA_DIR"
while IFS='|' read -r url filename; do
  [[ -z "$url" ]] && continue
  if curl -fsSL "$url" -o "$filename"; then
    log "  ✓ $filename"
  else
    err "  ✗ $filename  (fetch failed: $url)"
    FAIL=$((FAIL + 1))
  fi
done <<< "$MANIFEST"

if [[ "$FAIL" -gt 0 ]]; then
  err "Aborting: $FAIL schema(s) failed to fetch."
  exit 1
fi

log
log "✓ Schemas refreshed: 5 upstream-published"
log "  (Codex/Gemini plugin-manifest schemas remain synthesized — edit manually)"
