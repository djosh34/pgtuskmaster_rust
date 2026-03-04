#!/usr/bin/env python3
from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


@dataclass(frozen=True)
class Inputs:
    docs_src_files: Path
    summary_reachable_files: Path
    out_scope_map_csv: Path
    out_orphan_files_txt: Path


def read_lines(path: Path) -> list[str]:
    raw = path.read_text(encoding="utf-8")
    lines = [line.strip() for line in raw.splitlines()]
    return [line for line in lines if line]


def write_lines(path: Path, lines: Iterable[str]) -> None:
    path.write_text("".join(f"{line}\n" for line in lines), encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        docs_src_files=root / "docs-src-files.txt",
        summary_reachable_files=root / "summary-reachable-files.txt",
        out_scope_map_csv=root / "scope-map.csv",
        out_orphan_files_txt=root / "orphan-files.txt",
    )

    if not inputs.docs_src_files.exists():
        raise SystemExit(f"missing input: {inputs.docs_src_files}")
    if not inputs.summary_reachable_files.exists():
        raise SystemExit(f"missing input: {inputs.summary_reachable_files}")

    docs_files = read_lines(inputs.docs_src_files)
    reachable = set(read_lines(inputs.summary_reachable_files))

    rows: list[tuple[str, str, str]] = []
    orphans: list[str] = []
    for path in sorted(docs_files):
        if path == "docs/src/SUMMARY.md":
            rows.append((path, "internal-only", "mdbook_summary"))
            continue

        if path in reachable:
            rows.append((path, "reachable", "in_summary"))
            continue

        if path.startswith("docs/src/verification/"):
            rows.append((path, "internal-only", "verification_ledger"))
            continue

        rows.append((path, "orphan", "not_in_summary"))
        orphans.append(path)

    with inputs.out_scope_map_csv.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["path", "scope_status", "reason"])
        for row in rows:
            writer.writerow(list(row))

    write_lines(inputs.out_orphan_files_txt, orphans)
    print(f"wrote: {inputs.out_scope_map_csv} ({len(rows)} rows)")
    print(f"wrote: {inputs.out_orphan_files_txt} ({len(orphans)} orphans)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
