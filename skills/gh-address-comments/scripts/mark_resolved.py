#!/usr/bin/env python3
"""
Mark specific review threads as resolved using the GitHub GraphQL API.

Usage:
  python mark_resolved.py <thread_id_1> [thread_id_2] [...]

Example:
  python mark_resolved.py PRRT_kwDOABCDEF1234567 PRRT_kwDOABCDEF7654321
"""

from __future__ import annotations

import json
import subprocess
import sys
from typing import Any


MUTATION = """\
mutation($threadId: ID!) {
  resolveReviewThread(input: {threadId: $threadId}) {
    thread {
      id
      isResolved
      resolvedBy {
        login
      }
    }
  }
}
"""


def _run(cmd: list[str], stdin: str | None = None) -> str:
    """Execute command and return stdout."""
    p = subprocess.run(cmd, input=stdin, capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"Command failed: {' '.join(cmd)}\n{p.stderr}")
    return p.stdout


def _run_json(cmd: list[str], stdin: str | None = None) -> dict[str, Any]:
    """Execute command and parse JSON output."""
    out = _run(cmd, stdin=stdin)
    try:
        return json.loads(out)
    except json.JSONDecodeError as e:
        raise RuntimeError(
            f"Failed to parse JSON from command output: {e}\nRaw:\n{out}"
        ) from e


def mark_thread_resolved(thread_id: str) -> dict[str, Any]:
    """
    Mark a single review thread as resolved using GraphQL mutation.

    Args:
        thread_id: GitHub review thread ID (e.g., "PRRT_kwDOABCDEF1234567")

    Returns:
        GraphQL response with updated thread state
    """
    cmd = [
        "gh",
        "api",
        "graphql",
        "-F",
        "query=@-",
        "-F",
        f"threadId={thread_id}",
    ]

    result = _run_json(cmd, stdin=MUTATION)

    if "errors" in result and result["errors"]:
        raise RuntimeError(
            f"Failed to resolve thread {thread_id}:\n"
            f"{json.dumps(result['errors'], indent=2)}"
        )

    return result


def main() -> None:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(1)

    thread_ids = sys.argv[1:]
    results = []

    for thread_id in thread_ids:
        try:
            result = mark_thread_resolved(thread_id)
            thread = result["data"]["resolveReviewThread"]["thread"]
            resolved_by = thread.get("resolvedBy", {}).get("login", "unknown")

            print(f"✓ Resolved thread {thread_id} (by {resolved_by})")
            results.append({"thread_id": thread_id, "success": True})

        except RuntimeError as e:
            print(f"✗ Failed to resolve {thread_id}: {e}", file=sys.stderr)
            results.append({"thread_id": thread_id, "success": False, "error": str(e)})

    # Print summary
    success_count = sum(1 for r in results if r["success"])
    total_count = len(results)

    print(f"\nResolved {success_count}/{total_count} threads")

    if success_count < total_count:
        sys.exit(1)


if __name__ == "__main__":
    main()
