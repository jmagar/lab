#!/usr/bin/env bash
#
# validate-settings.sh — validate Codex/Gemini/Claude config and
# settings files against their published JSON Schemas.
#
# Targets and routing:
#   .codex/config.toml         → codex-config.schema.json (TOML, draft-07)
#   .gemini/settings.json      → gemini-settings.schema.json (draft 2020-12)
#   .claude/settings.json      → claude-settings.schema.json (draft-07)
#   .claude/settings.local.json → claude-settings.schema.json (same)
#
# These three schemas are upstream-published — refresh them with
# `refresh-schemas.sh` (no synthesis, unlike the plugin-manifest
# schemas).
#
# TOML files are parsed to JSON via Python's tomllib before validation.
# Requires Python 3.11+ for tomllib.
#
# Usage: validate-settings.sh PATH
#
# Exit codes:
#   0  valid
#   1  schema validation failed
#   3  path didn't match any known settings file type
#   4  manifest file or schema not found
#   5  no JSON Schema validator available (install jsonschema, check-jsonschema, or have npx)
#   6  TOML file but Python 3.11+ tomllib not available

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_DIR="$(cd "$SCRIPT_DIR/../references/schemas" && pwd)"

if [[ "${1:-}" =~ ^(--help|-h)$ ]]; then
  sed -n '2,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit 0
fi

TARGET="${1:-}"
if [[ -z "$TARGET" ]]; then
  echo "Usage: $(basename "$0") PATH-TO-CONFIG-FILE" >&2
  exit 64
fi

if [[ ! -f "$TARGET" ]]; then
  echo "Error: $TARGET not found." >&2
  exit 4
fi

# ---- route by path/basename -----------------------------------------

base="$(basename "$TARGET")"
SCHEMA=""
KIND=""
DRAFT=""
IS_TOML=0

case "$base" in
  config.toml)
    if [[ "$TARGET" == *.codex/config.toml || "$TARGET" == */.codex/config.toml ]]; then
      SCHEMA="$SCHEMA_DIR/codex-config.schema.json"
      KIND="Codex config (TOML)"
      DRAFT="draft7"
      IS_TOML=1
    fi
    ;;
  settings.json)
    if   [[ "$TARGET" == *.gemini/settings.json || "$TARGET" == */.gemini/settings.json ]]; then
      SCHEMA="$SCHEMA_DIR/gemini-settings.schema.json"
      KIND="Gemini settings"
      DRAFT="draft2020"
    elif [[ "$TARGET" == *.claude/settings.json || "$TARGET" == */.claude/settings.json ]]; then
      SCHEMA="$SCHEMA_DIR/claude-settings.schema.json"
      KIND="Claude settings"
      DRAFT="draft7"
    fi
    ;;
  settings.local.json)
    if [[ "$TARGET" == *.claude/settings.local.json || "$TARGET" == */.claude/settings.local.json ]]; then
      SCHEMA="$SCHEMA_DIR/claude-settings.schema.json"
      KIND="Claude settings (local)"
      DRAFT="draft7"
    fi
    ;;
esac

if [[ -z "$SCHEMA" ]]; then
  echo "Error: $TARGET does not match any known settings file." >&2
  echo "  Recognized:" >&2
  echo "    .codex/config.toml" >&2
  echo "    .gemini/settings.json" >&2
  echo "    .claude/settings.json" >&2
  echo "    .claude/settings.local.json" >&2
  exit 3
fi

if [[ ! -f "$SCHEMA" ]]; then
  echo "Error: schema not found at $SCHEMA" >&2
  echo "  Run scripts/refresh-schemas.sh to fetch it." >&2
  exit 4
fi

echo "→ target:    $TARGET"
echo "→ kind:      $KIND"
echo "→ schema:    $SCHEMA"
echo

# ---- TOML preprocessing (if needed) ---------------------------------
# Convert TOML → JSON in a temp file, then validate the JSON.

DOC_TO_VALIDATE="$TARGET"
TMP_JSON=""
cleanup() { [[ -n "$TMP_JSON" ]] && rm -f "$TMP_JSON"; }
trap cleanup EXIT

if [[ "$IS_TOML" -eq 1 ]]; then
  TMP_JSON="$(mktemp --suffix=.json)"
  if ! python3 - "$TARGET" "$TMP_JSON" <<'PY' 2>&1
import sys
try:
    import tomllib
except ImportError:
    sys.exit(6)
with open(sys.argv[1], 'rb') as f: doc = tomllib.load(f)
import json
with open(sys.argv[2], 'w') as f: json.dump(doc, f)
PY
  then
    rc=$?
    if [[ "$rc" -eq 6 ]]; then
      echo "  ! Python 3.11+ tomllib not available; cannot validate TOML." >&2
      exit 6
    fi
    echo "  ✗ TOML parse failed." >&2
    exit 1
  fi
  DOC_TO_VALIDATE="$TMP_JSON"
fi

# ---- validators (same fallback chain) -------------------------------

run_python_jsonschema() {
  python3 - "$DOC_TO_VALIDATE" "$SCHEMA" <<'PY'
import json, sys
try:
    import jsonschema
except ImportError:
    sys.exit(2)
with open(sys.argv[1]) as f: doc = json.load(f)
with open(sys.argv[2]) as f: schema = json.load(f)
draft_uri = schema.get("$schema", "")
if "2020-12" in draft_uri:
    Validator = jsonschema.Draft202012Validator
elif "2019-09" in draft_uri:
    Validator = jsonschema.Draft201909Validator
else:
    Validator = jsonschema.Draft7Validator
v = Validator(schema)
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
  check-jsonschema --schemafile "$SCHEMA" "$DOC_TO_VALIDATE"
}

run_ajv_cli() {
  command -v npx >/dev/null || return 2
  npx --yes ajv-cli@5 validate \
    --spec="$DRAFT" --strict=false \
    -s "$SCHEMA" -d "$DOC_TO_VALIDATE" 2>&1 \
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
    echo "  ✓ valid (validator: $VALIDATOR_USED, draft: $DRAFT)"
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
