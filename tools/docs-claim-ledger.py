#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import re
import sys
from dataclasses import dataclass


HEADING_RE = re.compile(r"^(#{1,6})\s+(.*)\s*$")
CLAIM_LINE_RE = re.compile(r"^(?P<path>[^:]+):(?P<line>\d+):(?P<text>.*)$")


@dataclass(frozen=True)
class Candidate:
    doc_path: str
    line: int
    text: str


def read_lines(path: pathlib.Path) -> list[str]:
    return path.read_text(encoding="utf-8").splitlines()


def parse_candidates(path: pathlib.Path) -> list[Candidate]:
    out: list[Candidate] = []
    for raw in read_lines(path):
        raw = raw.strip("\n")
        if not raw:
            continue
        match = CLAIM_LINE_RE.match(raw)
        if match is None:
            raise RuntimeError(f"unparseable candidate line: {raw!r}")
        out.append(
            Candidate(
                doc_path=match.group("path"),
                line=int(match.group("line")),
                text=match.group("text").strip(),
            )
        )
    return out


def heading_for_candidate(repo_root: pathlib.Path, candidate: Candidate) -> str:
    doc = repo_root / candidate.doc_path
    if not doc.is_file():
        return ""
    lines = read_lines(doc)
    # Candidate line numbers are 1-based.
    idx = min(max(candidate.line - 1, 0), max(len(lines) - 1, 0))
    heading: str = ""
    for i in range(idx, -1, -1):
        match = HEADING_RE.match(lines[i])
        if match is None:
            continue
        heading = match.group(2).strip()
        break
    return heading


def classify_claim_type(candidate: Candidate) -> str:
    path = candidate.doc_path
    text = candidate.text.lower()
    if path.startswith("docs/src/interfaces/"):
        return "api"
    if path == "docs/src/operator/configuration.md" or path.startswith("docs/src/operator/"):
        if "config" in text or "configuration" in text or "`" in candidate.text:
            return "config"
    if path.startswith("docs/src/assurance/") or "split-brain" in text or "split brain" in text:
        return "safety"
    if "must" in text or "never" in text or "cannot" in text or "impossible" in text:
        return "safety"
    return "behavior"


def expected_evidence_for_type(claim_type: str) -> str:
    if claim_type == "api":
        return "code+tests"
    if claim_type == "config":
        return "schema+parser+tests"
    if claim_type == "safety":
        return "code+tests+e2e/runtime"
    return "code+tests/runtime"


def tsv_row(fields: list[str]) -> str:
    return "\t".join(field.replace("\t", " ").strip() for field in fields)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--candidates", required=True)
    parser.add_argument("--out-ledger", required=True)
    args = parser.parse_args()

    repo_root = pathlib.Path(args.repo_root).resolve()
    candidates_path = pathlib.Path(args.candidates).resolve()
    out_path = pathlib.Path(args.out_ledger).resolve()

    candidates = parse_candidates(candidates_path)
    if not candidates:
        raise RuntimeError("no candidates provided; claim ledger would be empty")

    header = [
        "claim_id",
        "doc_path",
        "section_anchor_or_heading",
        "claim_text",
        "claim_type(behavior/safety/config/api)",
        "expected_evidence(code/test/runtime/doc_rationale)",
        "verification_status(pass/fail/uncertain)",
        "evidence_pointer",
    ]
    lines: list[str] = [tsv_row(header)]

    for idx, candidate in enumerate(candidates, start=1):
        claim_id = f"C{idx:04d}"
        heading = heading_for_candidate(repo_root, candidate)
        claim_type = classify_claim_type(candidate)
        expected = expected_evidence_for_type(claim_type)
        lines.append(
            tsv_row(
                [
                    claim_id,
                    candidate.doc_path,
                    heading,
                    candidate.text,
                    claim_type,
                    expected,
                    "uncertain",
                    "",
                ]
            )
        )

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
