#!/usr/bin/env python3
"""
nudge-session-adr.py — PostToolUse hook helper.

Fires a one-time nudge when a NEW session log is written to docs/sessions/
(matching the YYYY-MM-DD-<slug>.md naming pattern). Prompts the LLM to
review docs/design-docs/index.md and create any new ADRs that the
session's decisions warrant.

Reads the standard PostToolUse JSON payload from stdin:
  - tool_name: must be "Write" (creation/overwrite — not Edit)
  - tool_input.file_path: must match docs/sessions/YYYY-MM-DD-<slug>.md

Silent on any non-match. Never blocks (always exits 0).

Wired into .claude/settings.json as a third PostToolUse Edit|Write|MultiEdit
command, so it runs in parallel with the format/sync/validate commands.
"""

from __future__ import annotations

import json
import re
import sys

SESSION_LOG_RE = re.compile(r"docs/sessions/\d{4}-\d{2}-\d{2}-.+\.md$")


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0

    tool = data.get("tool_name", "")
    path = data.get("tool_input", {}).get("file_path", "")

    if tool != "Write":
        return 0
    if not SESSION_LOG_RE.search(path):
        return 0

    print(f"NOTE: New session log written to {path}.")
    print(
        "Before ending this session, review docs/design-docs/index.md and "
        "consider whether any architectural decisions made during this "
        "session warrant a new ADR. The ADR format is documented at "
        "docs/design-docs/2026-04-28-adr-format.md; new ADRs go in "
        "docs/design-docs/YYYY-MM-DD-<slug>.md and need a row in the "
        "Active table in index.md."
    )
    print(
        "Decisions worth an ADR: structural pivots, framework swaps, "
        "contract changes, reversed prior decisions, or anything you would "
        "want a future maintainer (or future you) to understand the rationale of."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
