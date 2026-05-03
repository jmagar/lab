#!/usr/bin/env python3
"""PreToolUse hook: block git hook/signing bypass flags."""
import json, sys, re

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    sys.exit(0)

cmd = data.get('tool_input', {}).get('command', '')
patterns = [r'--no-verify', r'--no-gpg-sign', r'-c\s+commit\.gpgsign=false', r'GIT_SSL_NO_VERIFY']
for p in patterns:
    if re.search(p, cmd):
        print(json.dumps({
            "hookSpecificOutput": {"permissionDecision": "deny"},
            "systemMessage": f"BLOCK: hook/signing bypass detected ({p!r}). Fix the underlying check failure rather than bypassing it. See spec.md § Bypass mechanisms.",
        }))
        sys.exit(0)
