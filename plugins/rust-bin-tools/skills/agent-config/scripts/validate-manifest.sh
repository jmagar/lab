#!/usr/bin/env bash
#
# validate-manifest.sh — schema-validate a plugin/marketplace
# manifest from any of the three platforms, auto-routing by path.
#
# Targets:
#   .claude-plugin/plugin.json       → claude-plugin.schema.json
#   .claude-plugin/marketplace.json  → claude-marketplace.schema.json
#   .codex-plugin/plugin.json        → codex-plugin.schema.json
#   .agents/plugins/marketplace.json → codex-marketplace.schema.json
#   gemini-extension.json (any path) → gemini-extension.schema.json
#
# These schemas are *synthesized* from the field tables in
# `agent-config/references/plugin-matrix.md` because none of the three
# platforms publish official JSON Schema files for these manifests.
# For Claude, this is a complement to `claude plugin validate` (which
# is the authoritative validator); for Codex and Gemini, this is the
# primary local pre-validation.
#
# Usage:
#   validate-manifest.sh PATH
#
# Exit codes:
#   0  valid
#   1  schema validation failed
#   3  path didn't match any known manifest type
#   4  manifest file or schema not found
#   5  no JSON Schema validator available

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_DIR="$(cd "$SCRIPT_DIR/../references/schemas" && pwd)"

if [[ "${1:-}" =~ ^(--help|-h)$ ]]; then
  sed -n '2,28p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit 0
fi

MANIFEST="${1:-}"
if [[ -z "$MANIFEST" ]]; then
  echo "Usage: $(basename "$0") PATH-TO-MANIFEST" >&2
  exit 64
fi

if [[ ! -f "$MANIFEST" ]]; then
  echo "Error: $MANIFEST not found." >&2
  exit 4
fi

# ---- route by path/basename -----------------------------------------

base="$(basename "$MANIFEST")"
SCHEMA=""
KIND=""

case "$MANIFEST" in
  *.claude-plugin/plugin.json|*/.claude-plugin/plugin.json)
    SCHEMA="$SCHEMA_DIR/claude-plugin.schema.json"
    KIND="Claude plugin"
    ;;
  *.claude-plugin/marketplace.json|*/.claude-plugin/marketplace.json)
    SCHEMA="$SCHEMA_DIR/claude-marketplace.schema.json"
    KIND="Claude marketplace"
    ;;
  *.codex-plugin/plugin.json|*/.codex-plugin/plugin.json)
    SCHEMA="$SCHEMA_DIR/codex-plugin.schema.json"
    KIND="Codex plugin"
    ;;
  *.agents/plugins/marketplace.json|*/.agents/plugins/marketplace.json)
    SCHEMA="$SCHEMA_DIR/codex-marketplace.schema.json"
    KIND="Codex marketplace"
    ;;
  *)
    if [[ "$base" == "gemini-extension.json" ]]; then
      SCHEMA="$SCHEMA_DIR/gemini-extension.schema.json"
      KIND="Gemini extension"
    fi
    ;;
esac

if [[ -z "$SCHEMA" ]]; then
  echo "Error: $MANIFEST does not match any known manifest path." >&2
  echo "  Recognized:" >&2
  echo "    .claude-plugin/plugin.json, .claude-plugin/marketplace.json," >&2
  echo "    .codex-plugin/plugin.json, .agents/plugins/marketplace.json," >&2
  echo "    gemini-extension.json" >&2
  exit 3
fi

if [[ ! -f "$SCHEMA" ]]; then
  echo "Error: schema not found at $SCHEMA" >&2
  exit 4
fi

echo "→ manifest:  $MANIFEST"
echo "→ kind:      $KIND"
echo "→ schema:    $SCHEMA"
echo

# ---- validators (same fallback chain as validate-server-json.sh) ----

run_python_jsonschema() {
  python3 - "$MANIFEST" "$SCHEMA" <<'PY'
import json, sys
try:
    import jsonschema
except ImportError:
    sys.exit(2)
with open(sys.argv[1]) as f: doc = json.load(f)
with open(sys.argv[2]) as f: schema = json.load(f)
v = jsonschema.Draft7Validator(schema)
errors = sorted(v.iter_errors(doc), key=lambda e: list(e.absolute_path))
if not errors:
    sys.exit(0)
for e in errors:
    loc = "/".join(str(p) for p in e.absolute_path) or "(root)"
    print(f"  ✗ {loc}: {e.message}")
sys.exit(1)
PY
}

run_check_jsonschema() {
  command -v check-jsonschema >/dev/null || return 2
  check-jsonschema --schemafile "$SCHEMA" "$MANIFEST"
}

run_ajv_cli() {
  command -v npx >/dev/null || return 2
  npx --yes ajv-cli@5 validate \
    --spec=draft7 --strict=false \
    -s "$SCHEMA" -d "$MANIFEST" 2>&1 \
    | grep -v 'unknown format "uri" ignored'
  return ${PIPESTATUS[0]}
}

VALIDATOR_USED=""
for fn in run_python_jsonschema run_check_jsonschema run_ajv_cli; do
  output="$($fn 2>&1)" && rc=0 || rc=$?
  if [[ "$rc" -eq 2 ]]; then
    continue
  fi
  VALIDATOR_USED="${fn#run_}"
  if [[ "$rc" -eq 0 ]]; then
    echo "  ✓ valid (validator: $VALIDATOR_USED)"
  else
    [[ -n "$output" ]] && echo "$output"
    echo
    echo "  Schema validation failed (validator: $VALIDATOR_USED)."
    exit 1
  fi
  break
done

if [[ -z "$VALIDATOR_USED" ]]; then
  echo "  ! No JSON Schema validator available. Install one of:" >&2
  echo "      pip install --user jsonschema" >&2
  echo "      pipx install check-jsonschema" >&2
  echo "      (npx auto-installs ajv-cli if Node is present)" >&2
  exit 5
fi

if [[ "$KIND" == Claude* ]]; then
  echo
  echo "  Note: schema validation passed against the upstream JSON Schema"
  echo "  (json.schemastore.org). For full semantic validation that the"
  echo "  marketplace machinery also runs at install time, also run:"
  echo "      claude plugin validate $(dirname "$(dirname "$MANIFEST")")"
fi
