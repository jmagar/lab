#!/usr/bin/env bash
#
# validate-server-json.sh — local validation of a server.json before
# running `mcp-publisher publish`.
#
# Catches the documented top-3 publish failures (per the upstream
# quickstart) BEFORE you do an irreversible npm/pypi/docker publish:
#
#   1. server.json schema violations (missing/typed-wrong fields)
#   2. server.json `name` != package manifest `mcpName` (or equivalent)
#   3. server.json `version` != published package version
#
# Plus a sanity check on the namespace requirement:
#   4. `name` starts with `io.github.<github-username>/` for GitHub auth
#
# Usage:
#   validate-server-json.sh [SERVER_JSON]
#   validate-server-json.sh [SERVER_JSON] --package-manifest PATH
#
# Defaults:
#   SERVER_JSON              ./server.json
#   --package-manifest       auto-detected (./package.json or
#                            ./pyproject.toml in the same dir as
#                            SERVER_JSON, in that order)
#
# Exit codes:
#   0  all checks passed
#   1  schema validation failed
#   2  cross-manifest mismatch (name or version)
#   3  namespace rule violated
#   4  required file missing
#   5  python3 + jsonschema not available

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHEMA_PATH="$SCRIPT_DIR/../references/server.schema.json"

# ---- arg parsing -----------------------------------------------------

SERVER_JSON=""
PACKAGE_MANIFEST=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --package-manifest)
      PACKAGE_MANIFEST="$2"; shift 2 ;;
    --help|-h)
      sed -n '2,30p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    -*)
      echo "Unknown flag: $1" >&2; exit 64 ;;
    *)
      [[ -z "$SERVER_JSON" ]] && SERVER_JSON="$1" || { echo "Too many positional args" >&2; exit 64; }
      shift ;;
  esac
done

SERVER_JSON="${SERVER_JSON:-./server.json}"

# ---- preflight -------------------------------------------------------

if [[ ! -f "$SERVER_JSON" ]]; then
  echo "Error: $SERVER_JSON not found." >&2
  exit 4
fi

if [[ ! -f "$SCHEMA_PATH" ]]; then
  echo "Error: schema not found at $SCHEMA_PATH" >&2
  echo "  Run scripts/refresh-references.sh to fetch it." >&2
  exit 4
fi

if ! command -v python3 >/dev/null; then
  echo "Error: python3 is required for schema validation." >&2
  exit 5
fi

# Auto-detect the package manifest if not provided.
if [[ -z "$PACKAGE_MANIFEST" ]]; then
  server_dir="$(cd "$(dirname "$SERVER_JSON")" && pwd)"
  for candidate in package.json pyproject.toml; do
    if [[ -f "$server_dir/$candidate" ]]; then
      PACKAGE_MANIFEST="$server_dir/$candidate"
      break
    fi
  done
fi

echo "→ server.json:        $SERVER_JSON"
echo "→ schema:             $SCHEMA_PATH"
if [[ -n "$PACKAGE_MANIFEST" ]]; then
  echo "→ package manifest:   $PACKAGE_MANIFEST"
else
  echo "→ package manifest:   (none found — cross-manifest checks will be skipped)"
fi
echo

# ---- §1 schema validation -------------------------------------------
# Each validator returns:
#   exit 0  → server.json is valid
#   exit 1  → server.json is invalid (output contains errors)
#   exit 2  → validator not available, try the next one

echo "§1 Schema validation"
SCHEMA_VERSION_FILE="$SCRIPT_DIR/../references/.schema-version"
SCHEMA_VERSION="$(cat "$SCHEMA_VERSION_FILE" 2>/dev/null || echo unknown)"

run_python_jsonschema() {
  python3 - "$SERVER_JSON" "$SCHEMA_PATH" <<'PY'
import json, sys
try:
    import jsonschema
except ImportError:
    sys.exit(2)
server_path, schema_path = sys.argv[1], sys.argv[2]
with open(server_path) as f: server = json.load(f)
with open(schema_path) as f: schema = json.load(f)
v = jsonschema.Draft7Validator(schema)
errors = sorted(v.iter_errors(server), key=lambda e: list(e.absolute_path))
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
  check-jsonschema --schemafile "$SCHEMA_PATH" "$SERVER_JSON"
}

run_ajv_cli() {
  command -v npx >/dev/null || return 2
  # Filter out the "unknown format 'uri' ignored" noise — the schema
  # uses uri formats that ajv intentionally doesn't validate by default.
  npx --yes ajv-cli@5 validate \
    --spec=draft7 \
    --strict=false \
    -s "$SCHEMA_PATH" \
    -d "$SERVER_JSON" 2>&1 | grep -v 'unknown format "uri" ignored'
  return ${PIPESTATUS[0]}
}

VALIDATOR_USED=""
for fn in run_python_jsonschema run_check_jsonschema run_ajv_cli; do
  # `|| rc=$?` prevents set -e from killing us on validator failure / unavailability.
  output="$($fn 2>&1)" && rc=0 || rc=$?
  if [[ "$rc" -eq 2 ]]; then
    continue
  fi
  VALIDATOR_USED="${fn#run_}"
  if [[ "$rc" -eq 0 ]]; then
    echo "  ✓ valid against schema $SCHEMA_VERSION (validator: $VALIDATOR_USED)"
  else
    [[ -n "$output" ]] && echo "$output"
    echo
    echo "  Schema validation failed (validator: $VALIDATOR_USED)."
    exit 1
  fi
  break
done

if [[ -z "$VALIDATOR_USED" ]]; then
  echo "  ! No JSON Schema validator available. Install one of:"
  echo "      pip install --user jsonschema           # smallest"
  echo "      pipx install check-jsonschema           # cleanest output"
  echo "      (npx auto-installs ajv-cli if Node is present)"
  exit 5
fi

# ---- §2 cross-manifest checks (name + version) ----------------------

# Pull values from server.json once (jq if available, else python).
if command -v jq >/dev/null; then
  SERVER_NAME="$(jq -r '.name // ""' "$SERVER_JSON")"
  SERVER_VERSION="$(jq -r '.version // ""' "$SERVER_JSON")"
else
  SERVER_NAME="$(python3 -c "import json; print(json.load(open('$SERVER_JSON')).get('name',''))")"
  SERVER_VERSION="$(python3 -c "import json; print(json.load(open('$SERVER_JSON')).get('version',''))")"
fi

echo
echo "§2 Cross-manifest checks"

if [[ -z "$PACKAGE_MANIFEST" ]]; then
  echo "  ⊘ skipped (no package.json or pyproject.toml found)"
else
  case "$PACKAGE_MANIFEST" in
    *package.json)
      if command -v jq >/dev/null; then
        PKG_MCPNAME="$(jq -r '.mcpName // ""' "$PACKAGE_MANIFEST")"
        PKG_VERSION="$(jq -r '.version // ""' "$PACKAGE_MANIFEST")"
      else
        PKG_MCPNAME="$(python3 -c "import json; print(json.load(open('$PACKAGE_MANIFEST')).get('mcpName',''))")"
        PKG_VERSION="$(python3 -c "import json; print(json.load(open('$PACKAGE_MANIFEST')).get('version',''))")"
      fi
      ;;
    *pyproject.toml)
      # Quick-and-dirty extraction; requires tomllib (Python 3.11+).
      PKG_MCPNAME="$(python3 -c "
import sys
try: import tomllib
except ImportError: sys.exit(0)
with open('$PACKAGE_MANIFEST','rb') as f: d=tomllib.load(f)
mcp=d.get('tool',{}).get('mcp',{})
print(mcp.get('name',''))
")"
      PKG_VERSION="$(python3 -c "
import sys
try: import tomllib
except ImportError: sys.exit(0)
with open('$PACKAGE_MANIFEST','rb') as f: d=tomllib.load(f)
print(d.get('project',{}).get('version',''))
")"
      ;;
  esac

  FAIL_X=0
  if [[ -z "$PKG_MCPNAME" ]]; then
    echo "  ✗ package manifest is missing the mcpName / [tool.mcp].name field"
    FAIL_X=1
  elif [[ "$PKG_MCPNAME" != "$SERVER_NAME" ]]; then
    echo "  ✗ name mismatch:"
    echo "      server.json .name        = $SERVER_NAME"
    echo "      package manifest mcpName = $PKG_MCPNAME"
    FAIL_X=1
  else
    echo "  ✓ name matches: $SERVER_NAME"
  fi

  if [[ -z "$PKG_VERSION" ]]; then
    echo "  ✗ package manifest is missing the version field"
    FAIL_X=1
  elif [[ "$PKG_VERSION" != "$SERVER_VERSION" ]]; then
    echo "  ✗ version mismatch:"
    echo "      server.json .version       = $SERVER_VERSION"
    echo "      package manifest .version  = $PKG_VERSION"
    FAIL_X=1
  else
    echo "  ✓ version matches: $SERVER_VERSION"
  fi

  if [[ "$FAIL_X" -eq 1 ]]; then
    exit 2
  fi
fi

# ---- §3 namespace rule ----------------------------------------------

echo
echo "§3 Namespace rule (GitHub auth)"
if [[ "$SERVER_NAME" =~ ^io\.github\.[A-Za-z0-9_-]+/[A-Za-z0-9._-]+$ ]]; then
  echo "  ✓ name follows io.github.<user>/<server> pattern"
else
  echo "  ✗ name '$SERVER_NAME' does not match the required pattern."
  echo "    For GitHub-based auth, server names MUST start with"
  echo "    io.github.<your-github-username>/<server-name>"
  exit 3
fi

echo
echo "✓ All checks passed. Safe to run \`mcp-publisher publish\`."
