#!/usr/bin/env bash
#
# refresh-references.sh — refresh the local copy of the upstream MCP
# registry reference docs used by the mcp-registry-publish skill.
#
# Pulls four markdown files plus the current server.json schema (whose
# version is auto-detected from the markdown). Run this whenever the
# upstream docs at github.com/modelcontextprotocol/registry change.
#
# Usage: scripts/refresh-references.sh [--quiet]
#
# Exit codes:
#   0  all references refreshed
#   1  network failure (one or more files couldn't be fetched)
#   2  schema URL couldn't be auto-detected (upstream format changed)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REFS_DIR="$(cd "$SCRIPT_DIR/../references" && pwd)"

QUIET=0
for arg in "$@"; do
  case "$arg" in
    --quiet|-q) QUIET=1 ;;
    --help|-h)
      sed -n '2,15p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      exit 64
      ;;
  esac
done

log() { [[ "$QUIET" -eq 1 ]] || echo "$@"; }
err() { echo "$@" >&2; }

# raw_url|local_filename
MANIFEST=$(cat <<'EOF'
https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/README.md|reference-overview.md
https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/cli/commands.md|cli-commands.md
https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/server-json/official-registry-requirements.md|official-registry-requirements.md
https://raw.githubusercontent.com/modelcontextprotocol/registry/main/docs/reference/server-json/generic-server-json.md|generic-server-json.md
EOF
)

cd "$REFS_DIR"

FAIL=0
log "Refreshing markdown references in $REFS_DIR"
while IFS='|' read -r url filename; do
  [[ -z "$url" ]] && continue
  if curl -fsSL "$url" -o "$filename"; then
    log "  ✓ $filename"
  else
    err "  ✗ $filename  (fetch failed)"
    FAIL=$((FAIL + 1))
  fi
done <<< "$MANIFEST"

if [[ "$FAIL" -gt 0 ]]; then
  err "Aborting: $FAIL markdown file(s) failed to fetch."
  exit 1
fi

# Auto-detect the current schema URL from the just-refreshed
# generic-server-json.md. This way the script self-updates when the
# schema version bumps upstream — no manual edits to this file.
SCHEMA_URL="$(grep -oE 'https://static\.modelcontextprotocol\.io/schemas/[0-9-]+/server\.schema\.json' \
  generic-server-json.md | head -1)"

if [[ -z "$SCHEMA_URL" ]]; then
  err
  err "Error: could not auto-detect the server.schema.json URL in"
  err "generic-server-json.md. The upstream doc format may have"
  err "changed. Inspect the file manually and update this script."
  exit 2
fi

SCHEMA_VERSION="$(echo "$SCHEMA_URL" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}')"
log
log "Schema URL detected: $SCHEMA_URL"
log "Schema version:       $SCHEMA_VERSION"
log

if curl -fsSL "$SCHEMA_URL" -o "server.schema.json"; then
  log "  ✓ server.schema.json (version $SCHEMA_VERSION)"
else
  err "  ✗ server.schema.json  (fetch failed at $SCHEMA_URL)"
  exit 1
fi

# Drop a small marker file so the agent can see which schema version
# is currently checked in without parsing the JSON.
echo "$SCHEMA_VERSION" > .schema-version

log
log "✓ References refreshed: 4 markdown + 1 schema (version $SCHEMA_VERSION)"
