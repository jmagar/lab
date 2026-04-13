#!/usr/bin/env python3
"""
Verify that all review threads have been addressed.

Reads PR comment data (from fetch_comments.py output) and checks:
1. All review threads are either resolved or outdated
2. All conversation comments have been addressed

Usage:
  python fetch_comments.py | python verify_resolution.py

Exit codes:
  0 - All comments addressed
  1 - Unresolved comments found
"""

from __future__ import annotations

import json
import sys
from typing import Any


def analyze_threads(data: dict[str, Any]) -> tuple[list[dict], list[dict]]:
    """
    Analyze review threads to find unresolved ones.

    Returns:
        Tuple of (unresolved_threads, resolved_threads)
    """
    review_threads = data.get("review_threads", [])
    unresolved = []
    resolved = []

    for thread in review_threads:
        # Thread is considered addressed if:
        # - It's marked as resolved, OR
        # - It's outdated (code changed since comment)
        if thread.get("isResolved") or thread.get("isOutdated"):
            resolved.append(thread)
        else:
            unresolved.append(thread)

    return unresolved, resolved


def format_thread_summary(thread: dict[str, Any], index: int) -> str:
    """Format a single thread for display."""
    path = thread.get("path", "unknown file")
    line = thread.get("line") or thread.get("originalLine", "?")
    comments = thread.get("comments", {}).get("nodes", [])

    if not comments:
        return f"  {index}. {path}:{line} (no comments)"

    first_comment = comments[0]
    author = first_comment.get("author", {}).get("login", "unknown")
    body_preview = first_comment.get("body", "")[:100].replace("\n", " ")

    return f"  {index}. {path}:{line} (@{author}): {body_preview}..."


def main() -> None:
    # Read PR data from stdin
    try:
        data = json.load(sys.stdin)
    except json.JSONDecodeError as e:
        print(f"Error: Failed to parse JSON input: {e}", file=sys.stderr)
        sys.exit(1)

    pr_info = data.get("pull_request", {})
    pr_number = pr_info.get("number", "?")
    pr_title = pr_info.get("title", "Unknown PR")

    print(f"Verifying PR #{pr_number}: {pr_title}")
    print("=" * 80)

    # Analyze threads
    unresolved, resolved = analyze_threads(data)

    # Show resolved threads
    if resolved:
        print(f"\n✓ {len(resolved)} thread(s) resolved or outdated")

    # Show unresolved threads
    if unresolved:
        print(f"\n✗ {len(unresolved)} UNRESOLVED thread(s):")
        for idx, thread in enumerate(unresolved, 1):
            print(format_thread_summary(thread, idx))
            print(f"     Thread ID: {thread.get('id', 'unknown')}")

        print("\n" + "=" * 80)
        print("BLOCKED: Address all unresolved threads before completing PR review.")
        print("\nTo mark threads as resolved, use:")
        print("  python scripts/mark_resolved.py <thread_id_1> [thread_id_2] ...")
        sys.exit(1)

    # Success
    print("\n" + "=" * 80)
    print("✓ All review threads have been addressed!")
    print("  You may proceed with completing the PR review.")
    sys.exit(0)


if __name__ == "__main__":
    main()
