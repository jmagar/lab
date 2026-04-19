#!/usr/bin/env python3
"""
Human-readable summary of PR review threads, grouped by file and reviewer.

Usage:
  gh-fetch-comments --pr 2 | gh-pr-summary
  gh-pr-summary --input pr.json
  gh-pr-summary --input pr.json --by reviewer
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from typing import Any


def _first_comment(thread: dict[str, Any]) -> dict[str, Any]:
    nodes = thread.get("comments", {}).get("nodes", [])
    return nodes[0] if nodes else {}


def _thread_status(thread: dict[str, Any]) -> str:
    if thread.get("isResolved"):
        return "resolved"
    if thread.get("isOutdated"):
        return "outdated"
    return "open"


def _status_icon(status: str) -> str:
    return {"resolved": "✓", "outdated": "~", "open": "✗"}.get(status, "?")


def summarize_by_file(threads: list[dict[str, Any]]) -> None:
    by_file: dict[str, list[dict]] = defaultdict(list)
    for thread in threads:
        by_file[thread.get("path", "(unknown)")].append(thread)

    for path in sorted(by_file):
        file_threads = by_file[path]
        open_count = sum(1 for t in file_threads if _thread_status(t) == "open")
        resolved_count = sum(1 for t in file_threads if _thread_status(t) == "resolved")
        print(f"\n  {path}  [{open_count} open, {resolved_count} resolved]")
        for thread in sorted(file_threads, key=lambda t: t.get("line") or t.get("originalLine") or 0):
            comment = _first_comment(thread)
            author = comment.get("author", {}).get("login", "?")
            line = thread.get("line") or thread.get("originalLine", "?")
            body = comment.get("body", "")[:120].replace("\n", " ")
            status = _thread_status(thread)
            icon = _status_icon(status)
            tid = thread.get("id", "")
            print(f"    {icon} L{line} @{author}: {body}...")
            if status == "open":
                print(f"       ID: {tid}")


def summarize_by_reviewer(threads: list[dict[str, Any]]) -> None:
    by_reviewer: dict[str, list[dict]] = defaultdict(list)
    for thread in threads:
        comment = _first_comment(thread)
        author = comment.get("author", {}).get("login", "unknown")
        by_reviewer[author].append(thread)

    for reviewer in sorted(by_reviewer):
        rev_threads = by_reviewer[reviewer]
        open_count = sum(1 for t in rev_threads if _thread_status(t) == "open")
        resolved_count = sum(1 for t in rev_threads if _thread_status(t) == "resolved")
        print(f"\n  @{reviewer}  [{open_count} open, {resolved_count} resolved]")
        for thread in rev_threads:
            path = thread.get("path", "?")
            line = thread.get("line") or thread.get("originalLine", "?")
            comment = _first_comment(thread)
            body = comment.get("body", "")[:120].replace("\n", " ")
            status = _thread_status(thread)
            icon = _status_icon(status)
            tid = thread.get("id", "")
            print(f"    {icon} {path}:L{line}: {body}...")
            if status == "open":
                print(f"       ID: {tid}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Human-readable summary of PR review threads",
        epilog="Reads JSON produced by gh-fetch-comments.",
    )
    parser.add_argument("--input", "-i", metavar="FILE", help="Read from FILE instead of stdin")
    parser.add_argument(
        "--by", choices=["file", "reviewer"], default="file",
        help="Group threads by file (default) or reviewer",
    )
    parser.add_argument("--open-only", action="store_true", help="Show only unresolved threads")
    args = parser.parse_args()

    try:
        if args.input:
            with open(args.input) as f:
                data = json.load(f)
        else:
            data = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    pr = data.get("pull_request", {})
    threads: list[dict[str, Any]] = data.get("review_threads", [])
    conv_comments: list[dict[str, Any]] = data.get("conversation_comments", [])
    reviews: list[dict[str, Any]] = data.get("reviews", [])

    open_threads = [t for t in threads if _thread_status(t) == "open"]
    resolved_threads = [t for t in threads if _thread_status(t) == "resolved"]
    outdated_threads = [t for t in threads if _thread_status(t) == "outdated"]

    print(f"PR #{pr.get('number', '?')}: {pr.get('title', 'Unknown')}")
    print(f"URL: {pr.get('url', '')}")
    print("=" * 80)
    print(f"  Review threads:  {len(open_threads)} open  •  {len(resolved_threads)} resolved  •  {len(outdated_threads)} outdated")
    print(f"  Conversation:    {len(conv_comments)} comment(s)")
    print(f"  Reviews:         {len(reviews)} submission(s)")

    display_threads = open_threads if args.open_only else threads
    if not display_threads:
        print("\n✓ No threads to display.")
        return

    label = "open threads" if args.open_only else "all threads"
    print(f"\n{'─' * 80}")
    print(f"Grouped by {args.by} ({label}):")

    if args.by == "file":
        summarize_by_file(display_threads)
    else:
        summarize_by_reviewer(display_threads)

    if open_threads:
        print(f"\n{'─' * 80}")
        print(f"To resolve all open threads at once:")
        print(f"  gh-mark-resolved --all --input <your-pr.json>")


if __name__ == "__main__":
    main()
