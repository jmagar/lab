#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/bin/link-claude-mds"

tmpdir="$(mktemp -d)"
cleanup() {
    rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$tmpdir/repo/bin" "$tmpdir/repo/nested/deeper"

cat > "$tmpdir/repo/CLAUDE.md" <<'EOF'
root claude
EOF

cat > "$tmpdir/repo/nested/deeper/CLAUDE.md" <<'EOF'
nested claude
EOF

cat > "$tmpdir/repo/nested/deeper/AGENTS.md" <<'EOF'
old agents
EOF

ln -s "somewhere-else" "$tmpdir/repo/GEMINI.md"

"$SCRIPT" "$tmpdir/repo"

assert_symlink_target() {
    local path="$1"
    local expected="$2"
    if [ ! -L "$path" ]; then
        echo "expected symlink: $path" >&2
        exit 1
    fi

    local target
    target="$(readlink "$path")"
    if [ "$target" != "$expected" ]; then
        echo "unexpected target for $path: got '$target', want '$expected'" >&2
        exit 1
    fi
}

assert_symlink_target "$tmpdir/repo/AGENTS.md" "CLAUDE.md"
assert_symlink_target "$tmpdir/repo/GEMINI.md" "CLAUDE.md"
assert_symlink_target "$tmpdir/repo/nested/deeper/AGENTS.md" "CLAUDE.md"
assert_symlink_target "$tmpdir/repo/nested/deeper/GEMINI.md" "CLAUDE.md"

echo "ok"
