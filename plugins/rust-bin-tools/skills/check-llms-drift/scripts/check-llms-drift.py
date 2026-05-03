#!/usr/bin/env python3
"""
check-llms-drift.py — audit docs/references llms.txt manifest for drift.

Three categories of finding (all mechanical — no model judgement needed):

  STALE       File at docs/references/<name>-llms.txt was last refreshed
              more than --max-age-days days ago. "Last refreshed" is read
              from the file's `generated:` frontmatter field, falling back
              to git mtime if absent.

  ORPHAN      Reference entry whose name doesn't appear in any of:
                Cargo.toml workspace.dependencies
                Cargo.lock packages
                apps/web/package.json {dependencies, devDependencies}
                apps/web/pnpm-lock.yaml (best-effort)

  VERSION     (only with --check-versions) Pinned version in Cargo.lock /
              package.json is older than registry's newest_version by at
              least a minor bump. Patch-only diffs are not flagged.

Output matches the contract documented in SKILL.md:

    STALE       axum-llms.txt        last refreshed 2024-11-03 (175d ago)
    ORPHAN      wiremock-llms.txt    not in Cargo.lock or package.json
    VERSION     tokio-llms.txt       pinned 1.36, latest 1.40 (minor)
    OK          rmcp-llms.txt
    ...

    Summary: 2 stale, 1 orphan, 1 version-drift, 30 ok

Exit codes:
  0  no drift detected
  1  drift detected (any category)
  2  docs/references directory not found
  3  bad arguments

Usage:
  check-llms-drift.py [--max-age-days N] [--check-versions] [--quiet]
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

try:
    import tomllib  # Python 3.11+
except ImportError:
    tomllib = None  # graceful degradation: skip Cargo.toml/Cargo.lock parsing


# ---- repo discovery -------------------------------------------------

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent.parent.parent  # .claude/skills/check-llms-drift/scripts → repo


# ---- manifest parsing -----------------------------------------------
#
# The manifest lives in YAML frontmatter at the top of each
# docs/references/*-llms.txt file. Each file is self-describing:
#
#   ---
#   repo: tokio-rs/axum
#   include: "README.md,axum/**,examples/**"
#   description: "axum router, extractor, and middleware reference"
#   generated: 2026-04-28
#   ---

REFS_DIR = REPO_ROOT / "docs" / "references"


@dataclass(frozen=True)
class ManifestEntry:
    filename: str          # "axum-llms.txt"
    name: str              # "axum"  (filename without -llms.txt)
    repo: str              # "tokio-rs/axum"
    description: str
    generated: str = ""    # ISO date from frontmatter, "" if absent

    @property
    def file_path(self) -> Path:
        return REFS_DIR / self.filename


def _unquote(v: str) -> str:
    v = v.strip()
    if len(v) >= 2 and v[0] == '"' and v[-1] == '"':
        return v[1:-1].replace('\\"', '"').replace("\\\\", "\\")
    return v


def parse_frontmatter(text: str) -> dict[str, str] | None:
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return None
    for i in range(1, min(len(lines), 100)):
        if lines[i].strip() == "---":
            meta: dict[str, str] = {}
            for line in lines[1:i]:
                if ":" in line:
                    k, v = line.split(":", 1)
                    meta[k.strip()] = _unquote(v.strip())
            return meta
    return None


def parse_manifest() -> list[ManifestEntry]:
    if not REFS_DIR.is_dir():
        print(f"Error: {REFS_DIR} not found", file=sys.stderr)
        sys.exit(2)

    entries: list[ManifestEntry] = []
    for path in sorted(REFS_DIR.glob("*-llms.txt")):
        # Read just the first 4KB — frontmatter is always near the top.
        with path.open() as f:
            head = f.read(4096)
        meta = parse_frontmatter(head)
        if not meta or "repo" not in meta:
            print(f"Warning: {path.name} has no frontmatter; skipping", file=sys.stderr)
            continue
        name = path.name.removesuffix("-llms.txt")
        entries.append(ManifestEntry(
            filename=path.name,
            name=name,
            repo=meta["repo"],
            description=meta.get("description", ""),
            generated=meta.get("generated", ""),
        ))
    return entries


# ---- staleness check ------------------------------------------------

def file_last_refresh_date(entry: ManifestEntry) -> datetime | None:
    """Return the last refresh date.

    Prefers `generated:` from frontmatter (the manage-llms-txt skill
    stamps this on every refresh). Falls back to git log when absent.
    """
    if entry.generated:
        try:
            return datetime.fromisoformat(entry.generated).replace(tzinfo=timezone.utc)
        except ValueError:
            pass

    path = entry.file_path
    if not path.is_file():
        return None
    try:
        result = subprocess.run(
            ["git", "log", "--follow", "-1", "--format=%aI", "--", str(path)],
            cwd=REPO_ROOT, capture_output=True, text=True, check=True,
        )
        iso = result.stdout.strip()
        if not iso:
            return None
        return datetime.fromisoformat(iso)
    except (subprocess.CalledProcessError, ValueError):
        return None


def days_since(dt: datetime) -> int:
    return (datetime.now(timezone.utc) - dt.astimezone(timezone.utc)).days


# ---- lockfile parsing -----------------------------------------------

def collect_known_deps() -> tuple[set[str], dict[str, str]]:
    """Return (set of all known dep names, dict name→pinned-version where available).

    Searches Cargo.toml / Cargo.lock / apps/web/package.json. Names are
    normalized to lowercase with - and _ collapsed (since the manifest
    naming and crate naming sometimes diverge on dashes vs underscores).
    """
    names: set[str] = set()
    pinned: dict[str, str] = {}

    def add(name: str, version: str | None = None) -> None:
        for variant in {name, name.replace("_", "-"), name.replace("-", "_")}:
            names.add(variant.lower())
        if version:
            pinned[name.lower()] = version

    # Cargo.toml workspace deps
    cargo_toml = REPO_ROOT / "Cargo.toml"
    if cargo_toml.is_file() and tomllib:
        try:
            data = tomllib.loads(cargo_toml.read_text())
            workspace_deps = (data.get("workspace") or {}).get("dependencies", {})
            for name, spec in workspace_deps.items():
                v = spec if isinstance(spec, str) else (spec or {}).get("version")
                add(name, v)
            # Also top-level [dependencies] / [dev-dependencies] when not a workspace.
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, spec in (data.get(section) or {}).items():
                    v = spec if isinstance(spec, str) else (spec or {}).get("version")
                    add(name, v)
        except Exception:
            pass

    # Per-crate Cargo.toml files (e.g. crates/core, crates/app)
    for crate_toml in REPO_ROOT.glob("crates/*/Cargo.toml"):
        if tomllib is None:
            break
        try:
            data = tomllib.loads(crate_toml.read_text())
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                for name, spec in (data.get(section) or {}).items():
                    v = spec if isinstance(spec, str) else (spec or {}).get("version")
                    add(name, v)
        except Exception:
            pass

    # Cargo.lock — authoritative resolved versions.
    cargo_lock = REPO_ROOT / "Cargo.lock"
    if cargo_lock.is_file() and tomllib:
        try:
            data = tomllib.loads(cargo_lock.read_text())
            for pkg in data.get("package", []):
                add(pkg.get("name", ""), pkg.get("version"))
        except Exception:
            pass

    # apps/web/package.json
    pkg_json = REPO_ROOT / "apps" / "web" / "package.json"
    if pkg_json.is_file():
        try:
            data = json.loads(pkg_json.read_text())
            for section in ("dependencies", "devDependencies", "peerDependencies"):
                for name, ver in (data.get(section) or {}).items():
                    add(name, ver)
        except Exception:
            pass

    # apps/web/pnpm-lock.yaml — best-effort line-grep (full YAML parse not in stdlib).
    pnpm_lock = REPO_ROOT / "apps" / "web" / "pnpm-lock.yaml"
    if pnpm_lock.is_file():
        try:
            for line in pnpm_lock.read_text().splitlines():
                # Lines like `'@scope/pkg@1.2.3':` or `pkg@1.2.3:` in v9 format.
                m = re.match(r"^\s+(['\"]?)((?:@[\w./-]+/)?[\w.-]+)@[\d.]+", line)
                if m:
                    add(m.group(2))
        except Exception:
            pass

    return names, pinned


# ---- version-drift check (registry lookup) --------------------------

CRATES_API = "https://crates.io/api/v1/crates/{name}"
NPM_API = "https://registry.npmjs.org/{name}/latest"


def fetch_json(url: str, timeout: float = 10.0) -> dict | None:
    req = urllib.request.Request(url, headers={"User-Agent": "check-llms-drift/1.0"})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read())
    except (urllib.error.URLError, urllib.error.HTTPError, json.JSONDecodeError, TimeoutError):
        return None


def latest_version(name: str, prefer: str = "auto") -> tuple[str | None, str]:
    """Return (latest_version_string, registry_used). registry: 'crates' or 'npm' or 'unknown'."""
    if prefer in ("auto", "crates"):
        d = fetch_json(CRATES_API.format(name=name))
        if d:
            return (((d.get("crate") or {}).get("newest_version")), "crates")
    if prefer in ("auto", "npm"):
        d = fetch_json(NPM_API.format(name=name))
        if d:
            return (d.get("version"), "npm")
    return (None, "unknown")


def parse_semver(s: str) -> tuple[int, int, int] | None:
    m = re.match(r"^[~^]?\s*(\d+)\.(\d+)\.(\d+)", s.strip())
    if not m:
        return None
    return tuple(int(x) for x in m.groups())  # type: ignore


def version_drift(pinned: str, latest: str) -> str | None:
    """Return None if no significant drift, else 'major' / 'minor'."""
    p = parse_semver(pinned)
    l = parse_semver(latest)
    if not p or not l:
        return None
    if l[0] > p[0]:
        return "major"
    if l[0] == p[0] and l[1] > p[1]:
        return "minor"
    return None  # patch-only or no drift


# ---- main audit -----------------------------------------------------

@dataclass
class Finding:
    label: str       # "STALE" | "ORPHAN" | "VERSION" | "OK"
    filename: str
    detail: str


def audit(max_age_days: int, check_versions: bool) -> list[Finding]:
    entries = parse_manifest()
    known_names, pinned = collect_known_deps()
    findings: list[Finding] = []

    # Helper: try matching the manifest entry name to any known dep name.
    def find_pinned(name: str) -> str | None:
        for cand in (name, name.replace("-", "_"), name.replace("_", "-")):
            if cand.lower() in pinned:
                return pinned[cand.lower()]
        return None

    def is_known(name: str) -> bool:
        return any(c.lower() in known_names for c in
                   (name, name.replace("-", "_"), name.replace("_", "-")))

    # First pass: classify into STALE / ORPHAN / OK.
    for entry in entries:
        if not is_known(entry.name):
            findings.append(Finding(
                label="ORPHAN", filename=entry.filename,
                detail="not in Cargo.lock, Cargo.toml, or package.json",
            ))
            continue

        last = file_last_refresh_date(entry)
        if last is None:
            findings.append(Finding(
                label="STALE", filename=entry.filename,
                detail="file missing or not in git",
            ))
            continue

        age = days_since(last)
        if age > max_age_days:
            findings.append(Finding(
                label="STALE", filename=entry.filename,
                detail=f"last refreshed {last.date()} ({age}d ago)",
            ))
        else:
            findings.append(Finding(
                label="OK", filename=entry.filename,
                detail=f"last refreshed {last.date()} ({age}d ago)",
            ))

    # Second pass: optional version-drift check on entries currently OK or STALE.
    if check_versions:
        def check_one(entry: ManifestEntry) -> Finding | None:
            pin = find_pinned(entry.name)
            if not pin:
                return None
            latest, reg = latest_version(entry.name)
            if not latest:
                return None
            drift = version_drift(pin, latest)
            if drift:
                return Finding(
                    label="VERSION", filename=entry.filename,
                    detail=f"pinned {pin}, latest {latest} ({drift}, via {reg})",
                )
            return None

        # parallelize the registry calls
        with ThreadPoolExecutor(max_workers=8) as pool:
            for vf in pool.map(check_one, entries):
                if vf is None:
                    continue
                # promote the finding: replace existing OK/STALE with VERSION,
                # or append if entry was already ORPHAN (rare).
                replaced = False
                for i, f in enumerate(findings):
                    if f.filename == vf.filename and f.label in ("OK", "STALE"):
                        findings[i] = vf
                        replaced = True
                        break
                if not replaced:
                    findings.append(vf)

    return findings


# ---- output ---------------------------------------------------------

LABEL_ORDER = ("VERSION", "STALE", "ORPHAN", "OK")


def render(findings: list[Finding]) -> str:
    lines: list[str] = []
    counts = {l: 0 for l in LABEL_ORDER}
    for label in LABEL_ORDER:
        block = [f for f in findings if f.label == label]
        for f in block:
            counts[label] += 1
            lines.append(f"{label:<10}  {f.filename:<32}  {f.detail}")
        if block and label != "OK":
            lines.append("")  # blank line between non-OK groups
    lines.append("")
    lines.append(
        f"Summary: {counts['STALE']} stale, "
        f"{counts['ORPHAN']} orphan, "
        f"{counts['VERSION']} version-drift, "
        f"{counts['OK']} ok"
    )
    return "\n".join(lines)


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--max-age-days", type=int, default=90,
                   help="Mark a file as STALE if last git update is older than this.")
    p.add_argument("--check-versions", action="store_true",
                   help="Also query crates.io / npm for newer versions (slower).")
    p.add_argument("--quiet", "-q", action="store_true",
                   help="Suppress OK rows; only show drift findings.")
    args = p.parse_args()

    if args.max_age_days < 0:
        print("--max-age-days must be non-negative", file=sys.stderr)
        return 3

    findings = audit(args.max_age_days, args.check_versions)
    output = render(findings)
    if args.quiet:
        # drop OK rows from rendered output
        output = "\n".join(
            line for line in output.splitlines()
            if not line.startswith("OK ")
        )
    print(output)

    drift_count = sum(1 for f in findings if f.label != "OK")
    return 1 if drift_count else 0


if __name__ == "__main__":
    sys.exit(main())
