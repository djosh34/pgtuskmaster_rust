#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass
from typing import Iterable, Optional


SUMMARY_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
MD_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+\.md)(?:#[^)]+)?\)")


@dataclass(frozen=True)
class ScopeRow:
    doc_path: str
    classification: str
    referenced_from: str


def run_git_ls_files(repo_root: pathlib.Path) -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "docs/src/*.md", "docs/src/**/*.md"],
        cwd=repo_root,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"git ls-files failed: {result.stderr.strip()}")
    paths = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    paths.sort()
    return paths


def normalize_summary_target(target: str) -> Optional[str]:
    target = target.strip()
    if not target:
        return None
    if "://" in target:
        return None
    if target.startswith("#"):
        return None
    if target.startswith("./"):
        target = target[2:]
    if target.startswith("/"):
        target = target[1:]
    if not target.endswith(".md"):
        return None
    return f"docs/src/{target}"


def extract_summary_links(summary_text: str) -> list[str]:
    out: list[str] = []
    for match in SUMMARY_LINK_RE.finditer(summary_text):
        target = normalize_summary_target(match.group(1))
        if target is None:
            continue
        out.append(target)
    return sorted(set(out))


def normalize_doc_link(source_doc: str, target: str) -> Optional[str]:
    target = target.strip()
    if not target or "://" in target or target.startswith("#"):
        return None
    # Strip anchor if present (we matched .md but keep safe).
    target = target.split("#", 1)[0]
    source = pathlib.PurePosixPath(source_doc)
    base_dir = source.parent
    if target.startswith("./"):
        target = target[2:]
    if target.startswith("/"):
        target = target[1:]
    normalized = (base_dir / target).as_posix()
    if not normalized.startswith("docs/src/") or not normalized.endswith(".md"):
        return None
    return normalized


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8")


def extract_md_links(doc_path: pathlib.Path) -> list[str]:
    text = read_text(doc_path)
    links: list[str] = []
    for match in MD_LINK_RE.finditer(text):
        links.append(match.group(1))
    return links


def write_lines(path: pathlib.Path, lines: Iterable[str]) -> None:
    content = "\n".join(lines) + ("\n" if lines else "")
    path.write_text(content, encoding="utf-8")


def write_csv(path: pathlib.Path, rows: list[ScopeRow]) -> None:
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["doc_path", "classification", "referenced_from"])
        for row in rows:
            writer.writerow([row.doc_path, row.classification, row.referenced_from])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root (default: .)",
    )
    parser.add_argument(
        "--summary",
        default="docs/src/SUMMARY.md",
        help="Path to mdbook SUMMARY.md (default: docs/src/SUMMARY.md)",
    )
    parser.add_argument(
        "--out-reachable",
        required=True,
        help="Output path for SUMMARY-reachable docs list (one path per line).",
    )
    parser.add_argument(
        "--out-scope-map",
        required=True,
        help="Output path for scope-map.csv classification output.",
    )

    args = parser.parse_args()
    repo_root = pathlib.Path(args.repo_root).resolve()
    summary_path = (repo_root / args.summary).resolve()
    if not summary_path.is_file():
        raise RuntimeError(f"SUMMARY.md missing: {summary_path}")

    tracked_docs = run_git_ls_files(repo_root)
    tracked_set = set(tracked_docs)
    summary_doc = "docs/src/SUMMARY.md"
    has_summary_doc = summary_doc in tracked_set
    if has_summary_doc:
        tracked_set.remove(summary_doc)

    summary_links = extract_summary_links(read_text(summary_path))
    summary_set = set(summary_links)

    missing_from_repo = sorted(summary_set - tracked_set)
    reachable = sorted(summary_set & tracked_set)
    not_in_summary = sorted(tracked_set - summary_set)

    linked_from_reachable: set[str] = set()
    for doc in reachable:
        doc_path = repo_root / doc
        if not doc_path.is_file():
            continue
        for raw_target in extract_md_links(doc_path):
            normalized = normalize_doc_link(doc, raw_target)
            if normalized is None:
                continue
            linked_from_reachable.add(normalized)

    rows: list[ScopeRow] = []
    if has_summary_doc:
        rows.append(ScopeRow(doc_path=summary_doc, classification="reachable", referenced_from="mdbook"))
    for doc in reachable:
        rows.append(ScopeRow(doc_path=doc, classification="reachable", referenced_from="SUMMARY.md"))
    for doc in not_in_summary:
        if doc in linked_from_reachable:
            rows.append(
                ScopeRow(doc_path=doc, classification="not-in-summary", referenced_from="reachable-doc-link")
            )
        else:
            rows.append(ScopeRow(doc_path=doc, classification="orphaned", referenced_from="none"))
    for doc in missing_from_repo:
        rows.append(ScopeRow(doc_path=doc, classification="unreachable", referenced_from="SUMMARY.md"))

    # Deterministic ordering for review.
    rows.sort(key=lambda row: (row.classification, row.doc_path))

    out_reachable = pathlib.Path(args.out_reachable)
    out_scope = pathlib.Path(args.out_scope_map)
    out_reachable.parent.mkdir(parents=True, exist_ok=True)
    out_scope.parent.mkdir(parents=True, exist_ok=True)

    write_lines(out_reachable, reachable)
    write_csv(out_scope, rows)

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
