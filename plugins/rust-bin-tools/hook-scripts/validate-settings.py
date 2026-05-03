#!/usr/bin/env python3
"""PostToolUse hook: validate Claude/Codex/Gemini settings files against schema."""
import json, sys, os, subprocess

data = json.load(sys.stdin)
path = data.get('tool_input', {}).get('file_path', '')
base = os.path.basename(path)

project_dir = os.environ.get('CLAUDE_PROJECT_DIR', os.getcwd())

is_settings = (
    (base == 'config.toml' and '.codex/' in path) or
    (base == 'settings.json' and ('.claude/' in path or '.gemini/' in path)) or
    (base == 'settings.local.json' and '.claude/' in path)
)
if is_settings:
    validate_script = os.path.join(project_dir, '.claude/skills/agent-config/scripts/validate-settings.sh')
    r = subprocess.run([validate_script, path], capture_output=True, text=True)
    if r.returncode != 0:
        print(f'⚠ Settings/config schema validation FAILED (exit {r.returncode}) — fix before relying on the file.')
        if r.stdout:
            print(r.stdout)
        if r.stderr:
            print(r.stderr)
