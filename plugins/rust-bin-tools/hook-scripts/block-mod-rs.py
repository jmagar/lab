#!/usr/bin/env python3
"""PreToolUse hook: block creation of mod.rs files (forbidden by convention)."""
import json, sys

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    sys.exit(0)

path = data.get('tool_input', {}).get('file_path', '')
if path.endswith('/mod.rs') or path == 'mod.rs':
    print(json.dumps({
        "hookSpecificOutput": {"permissionDecision": "deny"},
        "systemMessage": f"BLOCK: {path} — mod.rs is forbidden. Use file-per-module convention (e.g. foo.rs not foo/mod.rs). See CLAUDE.md § Critical invariants.",
    }))
    sys.exit(0)
