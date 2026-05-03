#!/usr/bin/env python3
"""PreToolUse hook: block direct edits to generated/locked files."""
import json, sys

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    sys.exit(0)

path = data.get('tool_input', {}).get('file_path', '')
blocked = ['docs/generated/', 'apps/web/src/lib/api-types.ts', 'Cargo.lock', 'pnpm-lock.yaml']
if any(b in path for b in blocked):
    print(json.dumps({
        "hookSpecificOutput": {"permissionDecision": "deny"},
        "systemMessage": f"BLOCK: {path} is generated/locked — run `just codegen` or `cargo xtask gen-*` instead of editing directly.",
    }))
    sys.exit(0)
