#!/usr/bin/env bash
#
# download-docs.sh — download every URL listed in docs.md to the right
# folder under .claude/skills/agent-config/references/.
#
# Routing:
#   code.claude.com/docs/en/<path>.md         → references/claude/<path>.md      (slashes flattened to dashes)
#   developers.openai.com/codex.md            → references/codex/codex.md
#   developers.openai.com/codex/<path>.md     → references/codex/<path>.md       (slashes flattened to dashes)
#   geminicli.com/docs.md                     → references/gemini/docs.md
#   geminicli.com/docs/<path>.md              → references/gemini/<path>.md      (slashes flattened to dashes)
#   agentskills.io/<path>.md                  → references/agent-skills/<path>.md (slashes flattened to dashes)
#
# Idempotent: existing files are skipped unless --force is given.
#
# Usage: download-docs.sh [--force] [--quiet] [--parallel N] [--docs PATH]
#
# Exit codes:
#   0  all downloads succeeded (or skipped because already present)
#   1  one or more downloads failed
#   2  routing failed for one or more URLs (unknown host pattern)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REFS_DIR="$(cd "$SCRIPT_DIR/../references" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DOCS_MD="$REFS_DIR/docs-urls.md"

FORCE=0
QUIET=0
PARALLEL=8
while [[ $# -gt 0 ]]; do
  case "$1" in
    --force)        FORCE=1; shift ;;
    --quiet|-q)     QUIET=1; shift ;;
    --parallel)     PARALLEL="$2"; shift 2 ;;
    --docs)         DOCS_MD="$2"; shift 2 ;;
    --help|-h)
      sed -n '2,21p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 64 ;;
  esac
done

log() { [[ "$QUIET" -eq 1 ]] || echo "$@"; }
err() { echo "$@" >&2; }

if [[ ! -f "$DOCS_MD" ]]; then
  err "Error: docs.md not found at $DOCS_MD"
  exit 1
fi

# Build URL→target list using Python for clarity (the routing has
# enough special cases that pure-bash would be brittle). Output is
# tab-separated `url\ttarget_path` lines, one per URL we know how to
# route. URLs we can't route are reported as warnings.

mapfile -t LINES < <(python3 - "$DOCS_MD" "$REFS_DIR" <<'PY'
import re, sys
from urllib.parse import urlparse

docs_md, refs_dir = sys.argv[1], sys.argv[2]
url_re = re.compile(r"https?://\S+")

def route(url):
    u = urlparse(url)
    host = u.netloc
    path = u.path.lstrip("/")
    # Drop a trailing .md; we'll re-add it after slashes→dashes.
    if path.endswith(".md"):
        path = path[:-3]

    if host == "code.claude.com" and path.startswith("docs/en/"):
        rel = path[len("docs/en/"):]
        if not rel:
            return ("claude", "index.md")
        return ("claude", rel.replace("/", "-") + ".md")

    if host == "developers.openai.com":
        if path == "codex":
            return ("codex", "codex.md")
        if path.startswith("codex/"):
            return ("codex", path[len("codex/"):].replace("/", "-") + ".md")

    if host == "geminicli.com":
        if path == "docs":
            return ("gemini", "docs.md")
        if path.startswith("docs/"):
            return ("gemini", path[len("docs/"):].replace("/", "-") + ".md")

    if host == "agentskills.io":
        if not path:
            return ("agent-skills", "index.md")
        return ("agent-skills", path.replace("/", "-") + ".md")

    return None

with open(docs_md) as f:
    text = f.read()

unrouted = []
seen = set()
for url in url_re.findall(text):
    if url in seen:
        continue
    seen.add(url)
    target = route(url)
    if target is None:
        unrouted.append(url)
        continue
    folder, fname = target
    print(f"{url}\t{folder}/{fname}")

for url in unrouted:
    print(f"UNROUTED\t{url}", file=sys.stderr)
PY
)

# Separate routed pairs from unrouted lines (which went to stderr above
# and are already on the user's screen). LINES has the routed pairs.

if [[ ${#LINES[@]} -eq 0 ]]; then
  err "Error: no routable URLs found in $DOCS_MD"
  exit 2
fi

log "Found ${#LINES[@]} routable URLs in $DOCS_MD"
log "Target root: $REFS_DIR"
log "Parallel: $PARALLEL$([ $FORCE -eq 1 ] && echo ' (forced re-download)')"
log

# Compute work: skip files that already exist unless --force.
TODO_FILE="$(mktemp)"
trap 'rm -f "$TODO_FILE"' EXIT

skipped=0
backfilled=0
for line in "${LINES[@]}"; do
  url="${line%%$'\t'*}"
  rel="${line#*$'\t'}"
  abs="$REFS_DIR/$rel"
  if [[ -f "$abs" ]] && [[ "$FORCE" -eq 0 ]]; then
    # Backfill: stamp `# {url}` header onto pre-existing files that
    # lack it. Cheap, idempotent, keeps provenance consistent across
    # the whole reference set without forcing a re-download.
    if ! head -n1 "$abs" | grep -q "^# $url\$"; then
      tmp="${abs}.tmp"
      { printf '# %s\n\n' "$url"; cat "$abs"; } > "$tmp"
      mv "$tmp" "$abs"
      backfilled=$((backfilled + 1))
    fi
    skipped=$((skipped + 1))
    continue
  fi
  printf '%s\t%s\n' "$url" "$abs" >> "$TODO_FILE"
done

todo_count=$(wc -l < "$TODO_FILE" | tr -d ' ')
log "Skipping $skipped already-downloaded file(s); will fetch $todo_count."
[[ "$todo_count" -eq 0 ]] && { log; log "✓ Nothing to do."; exit 0; }

# Pre-create all target directories so curl writes succeed.
awk -F'\t' '{print $2}' "$TODO_FILE" | xargs -n1 dirname | sort -u | while read -r d; do
  mkdir -p "$d"
done

# Parallel fetch via xargs. Each line in $TODO_FILE is "url\tabs-path".
# We pass them to a small bash function that runs curl. -0 mode would
# require null-separated input; tab-separated + read -r is simpler.
fetch_one() {
  local pair="$1"
  local url="${pair%%$'\t'*}"
  local abs="${pair#*$'\t'}"
  local tmp="${abs}.tmp"
  if curl -fsSL --max-time 30 --retry 2 --retry-delay 1 "$url" -o "$tmp"; then
    # Prepend a `# {url}` provenance header so each file remembers
    # where it came from. The header is re-stamped on every refresh,
    # so it never drifts even if upstream content changes.
    {
      printf '# %s\n\n' "$url"
      cat "$tmp"
    } > "$abs"
    rm -f "$tmp"
    echo "OK	$abs"
  else
    rm -f "$tmp"
    echo "FAIL	$url	$abs"
  fi
}
export -f fetch_one

# Use xargs to run in parallel.
RESULT_FILE="$(mktemp)"
trap 'rm -f "$TODO_FILE" "$RESULT_FILE"' EXIT

xargs -P "$PARALLEL" -I {} bash -c 'fetch_one "$@"' _ {} < "$TODO_FILE" > "$RESULT_FILE"

ok=$(grep -c '^OK	' "$RESULT_FILE" || true)
fail=$(grep -c '^FAIL	' "$RESULT_FILE" || true)

if [[ "$fail" -gt 0 ]]; then
  err
  err "Failed downloads:"
  grep '^FAIL	' "$RESULT_FILE" | awk -F'\t' '{print "  ✗ " $2}' >&2
fi

log
log "✓ Downloaded: $ok · Skipped (already present): $skipped · Backfilled headers: $backfilled · Failed: $fail"

[[ "$fail" -gt 0 ]] && exit 1
exit 0
