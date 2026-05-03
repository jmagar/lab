#!/usr/bin/env python3
"""Stop hook: remind to run spec gates if Rust/TOML files are modified in the working tree."""
import json, sys, os, subprocess

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError:
    sys.exit(0)

project_dir = os.environ.get('CLAUDE_PROJECT_DIR', os.getcwd())

result = subprocess.run(
    ['git', 'status', '--porcelain'],
    capture_output=True, text=True, cwd=project_dir,
)
if result.returncode != 0:
    sys.exit(0)

changed = [line[3:].strip() for line in result.stdout.splitlines() if line.strip()]
rust_changed = [f for f in changed if f.endswith('.rs') or f.endswith('.toml')]

if rust_changed:
    print(json.dumps({
        "decision": "approve",
        "systemMessage": (
            f"{len(rust_changed)} Rust/TOML file(s) modified. "
            "Before declaring done, verify spec gates pass: invoke the `spec-check` skill "
            "or run `cargo xtask check-tracing-fields && cargo xtask check-env-allowlist && "
            "cargo xtask check-no-mod-rs && just lint && just test`."
        ),
    }))
