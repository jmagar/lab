#!/usr/bin/env python3
"""
manage-llms-txt — unified manager for docs/references/*-llms.txt files.

Each *-llms.txt file is self-describing — its YAML frontmatter at the
top records where the content came from, what was included, and when
it was generated. There is NO separate manifest file: the directory
of *.llms.txt files (each with its own header) is the source of truth.

Header shape:

    ---
    repo: tokio-rs/axum
    include: README.md,axum/**,examples/**
    description: axum router, extractor, and middleware reference
    generated: 2026-04-28
    ---

    [repomix-generated content starts here]

Usage:
  manage-llms-txt <owner/repo>             Create a new -llms.txt entry (shortcut for `create`).
  manage-llms-txt create <owner/repo>      Same; explicit form.
  manage-llms-txt refresh                  Refresh every entry (preserving each file's header).
  manage-llms-txt refresh <owner/repo>     Refresh only that entry.
  manage-llms-txt refresh <name>           Refresh by filename (axum-llms.txt or axum).
  manage-llms-txt list                     List entries from the directory's headers.
  manage-llms-txt remove <owner/repo>      Delete the file (asks first). Index rebuilt automatically.
  manage-llms-txt rebuild-index            Regenerate docs/references/index.md from headers.

Create-only flags (after the repo arg):
  --include "<glob>"       Files to fetch via repomix.
                           Default: README.md,docs/**/*.md,docs/**/*.mdx,src/**
  --description "<text>"   Override; default fetches the GitHub repo description.
  --name <basename>        Override the filename basename. Default: the part
                           of <owner/repo> after the slash.

Exit codes:
  0  success
  1  repomix failed; or `remove` was declined
  2  bad arguments / target not found
  3  npx / repomix not available
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from shutil import which

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent.parent
REFS_DIR = REPO_ROOT / "docs" / "references"
INDEX_FILE = REFS_DIR / "index.md"

DEFAULT_INCLUDE = "README.md,docs/**/*.md,docs/**/*.mdx,src/**"


# ---- frontmatter ----------------------------------------------------

_QUOTE_TRIGGERS = (":", '"', "\n", "#")


def _format_value(v: str) -> str:
    """Quote a YAML value when it contains characters that would confuse a tiny parser."""
    if any(c in v for c in _QUOTE_TRIGGERS) or v.startswith(("-", "[", "{", "&", "*", "!", "|", ">")):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    return v


def _unquote_value(v: str) -> str:
    if len(v) >= 2 and v[0] == '"' and v[-1] == '"':
        return v[1:-1].replace('\\"', '"').replace("\\\\", "\\")
    return v


def format_frontmatter(meta: dict[str, str]) -> str:
    body = "\n".join(f"{k}: {_format_value(v)}" for k, v in meta.items())
    return f"---\n{body}\n---\n\n"


def parse_frontmatter(text: str) -> tuple[dict[str, str] | None, str]:
    """Return (meta, content_after) — meta is None if no frontmatter is found."""
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return None, text
    for i in range(1, min(len(lines), 100)):
        if lines[i].strip() == "---":
            meta: dict[str, str] = {}
            for line in lines[1:i]:
                if ":" in line:
                    k, v = line.split(":", 1)
                    meta[k.strip()] = _unquote_value(v.strip())
            content = "\n".join(lines[i + 1:]).lstrip("\n")
            return meta, content
    return None, text


# ---- entry model ----------------------------------------------------

@dataclass
class Entry:
    filename: str
    repo: str
    include: str
    description: str
    generated: str = ""

    @property
    def path(self) -> Path:
        return REFS_DIR / self.filename

    def header_dict(self) -> dict[str, str]:
        d = {
            "repo": self.repo,
            "include": self.include,
            "description": self.description,
        }
        if self.generated:
            d["generated"] = self.generated
        return d


def read_entry(path: Path) -> Entry | None:
    """Read an Entry from a *-llms.txt file by parsing its frontmatter. Returns None if no header."""
    if not path.is_file():
        return None
    text = path.read_text()
    meta, _ = parse_frontmatter(text)
    if not meta or "repo" not in meta:
        return None
    return Entry(
        filename=path.name,
        repo=meta.get("repo", ""),
        include=meta.get("include", DEFAULT_INCLUDE),
        description=meta.get("description", ""),
        generated=meta.get("generated", ""),
    )


def all_entries() -> list[Entry]:
    """Discover every *-llms.txt file in docs/references/ that has a valid frontmatter."""
    if not REFS_DIR.is_dir():
        return []
    out: list[Entry] = []
    for p in sorted(REFS_DIR.glob("*-llms.txt")):
        e = read_entry(p)
        if e is not None:
            out.append(e)
    return out


def write_entry(entry: Entry, content: str) -> None:
    """Write `<header>\\n\\n<content>` to the entry's path, atomically."""
    REFS_DIR.mkdir(parents=True, exist_ok=True)
    final = format_frontmatter(entry.header_dict()) + content.lstrip("\n")
    tmp = entry.path.with_suffix(entry.path.suffix + ".tmp")
    tmp.write_text(final)
    tmp.replace(entry.path)


# ---- helpers --------------------------------------------------------

def _today_iso() -> str:
    return dt.date.today().isoformat()


def _require(tool: str) -> None:
    if not which(tool):
        sys.exit(f"Error: {tool} not found on PATH. Install it and re-run.")


def _fetch_github_description(repo: str) -> str | None:
    url = f"https://api.github.com/repos/{repo}"
    req = urllib.request.Request(url, headers={
        "Accept": "application/vnd.github+json",
        "User-Agent": "manage-llms-txt/2.0",
    })
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read())
            desc = data.get("description")
            return desc.strip() if desc else None
    except (urllib.error.URLError, urllib.error.HTTPError, json.JSONDecodeError, TimeoutError, ValueError):
        return None


def _run_repomix(repo: str, include: str, out_path: Path) -> int:
    return subprocess.run([
        "npx", "--yes", "repomix",
        "--remote", f"https://github.com/{repo}",
        "--include", include,
        "--style", "markdown",
        "--parsable-style",
        "--compress",
        "--no-security-check",
        "--output", str(out_path),
        "--quiet",
    ]).returncode


def _resolve_to_entry(target: str) -> Entry | None:
    """Resolve owner/repo OR a filename (with/without -llms.txt) to an Entry."""
    entries = all_entries()
    if "/" in target:
        return next((e for e in entries if e.repo == target), None)
    fname = target if target.endswith("-llms.txt") else f"{target}-llms.txt"
    return next((e for e in entries if e.filename == fname), None)


# ---- index rebuild --------------------------------------------------

INDEX_HEADER = """\
# External References

Vendored / mirrored docs from external libraries we depend on, in a
form agents can read in-context. Each `<library>-llms.txt` file is
self-describing — open one and the YAML frontmatter at the top tells
you where the content came from and what was included.

## Why this directory exists

> If the agent can't access it in-context while running, it doesn't
> exist.

When an agent needs to know "how do I use rmcp's elicitation API?" or
"how does utoipa-axum register routes?", a fresh WebFetch is slow,
network-dependent, and produces unverified results. A checked-in
`llms.txt` is fast, deterministic, and version-pinned to the library
version we're actually using.

## Conventions

- File name: `<library-name>-llms.txt`
- Each file's frontmatter records `repo`, `include`, `description`, and `generated` (date)
- Generated via `manage-llms-txt` (in `.claude/skills/manage-llms-txt/`) which wraps `repomix`
- **Never paraphrased** — verbatim from the source so we don't accidentally introduce hallucinations into the reference

## Contents

"""

INDEX_FOOTER = """

## See also

- **`docs/research/`** — external research artifacts (web stack
  audits, CVE checks, version verification reports). Distinct
  category — those are point-in-time investigations, this directory
  is ongoing reference material.
- **OpenAI harness engineering**: https://openai.com/index/harness-engineering/
"""


def cmd_rebuild_index() -> int:
    REFS_DIR.mkdir(parents=True, exist_ok=True)
    entries = all_entries()
    body = "\n".join(f"- `{e.filename}` — {e.description}" for e in entries) or "_(no entries)_"
    INDEX_FILE.write_text(INDEX_HEADER + body + INDEX_FOOTER)
    return 0


# ---- subcommand: create ---------------------------------------------

def cmd_create(repo: str, extra: list[str]) -> int:
    if "/" not in repo or repo.startswith("/") or repo.endswith("/"):
        sys.exit(f"Error: expected owner/repo format, got {repo!r}")

    p = argparse.ArgumentParser(prog="manage-llms-txt create", add_help=False)
    p.add_argument("--include")
    p.add_argument("--description")
    p.add_argument("--name")
    args = p.parse_args(extra)

    name = args.name or repo.split("/", 1)[1]
    filename = f"{name}-llms.txt"

    existing = all_entries()
    if any(e.filename == filename for e in existing):
        sys.exit(f"Error: {filename} already exists. Use `refresh {repo}` to update content, "
                 f"or `remove {repo}` first.")
    if any(e.repo == repo for e in existing):
        match = next(e for e in existing if e.repo == repo)
        sys.exit(f"Error: {repo} is already covered by {match.filename}.")

    description = args.description or _fetch_github_description(repo) or f"{name} reference"
    include = args.include or DEFAULT_INCLUDE

    _require("npx")
    REFS_DIR.mkdir(parents=True, exist_ok=True)
    out_path = REFS_DIR / filename

    print(f"→ Creating {filename}")
    print(f"  repo:        {repo}")
    print(f"  include:     {include}")
    print(f"  description: {description}")
    print()
    print("Fetching content via repomix...")

    rc = _run_repomix(repo, include, out_path)
    if rc != 0:
        sys.exit(f"repomix failed (exit {rc}). No file written.")
    if not out_path.is_file() or out_path.stat().st_size == 0:
        sys.exit("repomix exited 0 but produced no output. No file written.")

    # repomix wrote raw content; now rewrite with frontmatter prepended.
    content = out_path.read_text()
    entry = Entry(filename=filename, repo=repo, include=include,
                  description=description, generated=_today_iso())
    write_entry(entry, content)
    print(f"\n✓ {filename} created with header.")

    cmd_rebuild_index()
    print("✓ Index rebuilt.")
    return 0


# ---- subcommand: refresh --------------------------------------------

def cmd_refresh(target: str | None) -> int:
    _require("npx")
    if target is None:
        return _refresh_many(all_entries())
    entry = _resolve_to_entry(target)
    if entry is None:
        print(f"Error: {target!r} not found in docs/references/.", file=sys.stderr)
        print("Run `manage-llms-txt list` to see available entries.", file=sys.stderr)
        return 2
    return _refresh_many([entry])


def _refresh_many(entries: list[Entry]) -> int:
    if not entries:
        print("Nothing to refresh.")
        return 0
    fail = 0
    for e in entries:
        print(f"→ Refreshing {e.filename} from {e.repo}")
        rc = _run_repomix(e.repo, e.include, e.path)
        if rc != 0 or not e.path.is_file() or e.path.stat().st_size == 0:
            print(f"  ✗ repomix failed for {e.filename}", file=sys.stderr)
            fail += 1
            continue
        content = e.path.read_text()
        # PRESERVE the existing header (repo/include/description) and update `generated`.
        e.generated = _today_iso()
        write_entry(e, content)
        print(f"  ✓ {e.filename}")
    cmd_rebuild_index()
    if fail:
        print(f"\n{fail} of {len(entries)} entries failed to refresh.", file=sys.stderr)
        return 1
    print(f"\n✓ Refreshed {len(entries)} entries; index rebuilt.")
    return 0


# ---- subcommand: list -----------------------------------------------

def cmd_list() -> int:
    entries = all_entries()
    if not entries:
        print("(no llms.txt files in docs/references/)")
        return 0
    fname_w = max(len(e.filename) for e in entries)
    repo_w = max(len(e.repo) for e in entries)
    for e in entries:
        gen = f"  ({e.generated})" if e.generated else ""
        print(f"  {e.filename:<{fname_w}}  {e.repo:<{repo_w}}  {e.description}{gen}")
    print(f"\n{len(entries)} entries")
    return 0


# ---- subcommand: remove ---------------------------------------------

def cmd_remove(repo: str, assume_yes: bool) -> int:
    if "/" not in repo:
        sys.exit(f"Error: expected owner/repo format, got {repo!r}")

    entry = next((e for e in all_entries() if e.repo == repo), None)
    if entry is None:
        print(f"Error: {repo!r} not found in docs/references/.", file=sys.stderr)
        return 2

    if not assume_yes:
        print("Will remove:")
        print(f"  {entry.filename}")
        print(f"  {entry.repo}  —  {entry.description}")
        try:
            ans = input("\nProceed? [y/N] ").strip().lower()
        except EOFError:
            ans = ""
        if ans not in ("y", "yes"):
            print("Aborted.")
            return 1

    if entry.path.exists():
        entry.path.unlink()
        print(f"Deleted {entry.path.relative_to(REPO_ROOT)}")

    cmd_rebuild_index()
    print(f"\n✓ Removed {entry.filename}; index rebuilt.")
    return 0


# ---- dispatch -------------------------------------------------------

def main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help", "help"):
        print(__doc__)
        return 0

    cmd, rest = argv[0], argv[1:]

    # Shortcut: bare `owner/repo` with no subcommand → create.
    if "/" in cmd and not cmd.startswith("-"):
        return cmd_create(cmd, rest)

    if cmd == "create":
        if not rest:
            sys.exit("Usage: manage-llms-txt create <owner/repo> [--include ...] [--description ...] [--name ...]")
        return cmd_create(rest[0], rest[1:])

    if cmd == "refresh":
        return cmd_refresh(rest[0] if rest else None)

    if cmd == "list":
        return cmd_list()

    if cmd == "remove":
        if not rest:
            sys.exit("Usage: manage-llms-txt remove <owner/repo> [--yes]")
        assume_yes = "--yes" in rest or "-y" in rest
        return cmd_remove(rest[0], assume_yes)

    if cmd == "rebuild-index":
        return cmd_rebuild_index()

    print(f"Unknown command: {cmd!r}. Use --help for usage.", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
