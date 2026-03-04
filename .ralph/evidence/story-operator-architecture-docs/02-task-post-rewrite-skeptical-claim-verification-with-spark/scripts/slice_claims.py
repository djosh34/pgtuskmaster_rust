#!/usr/bin/env python3
from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    verification_matrix_csv: Path
    out_subagents_dir: Path
    out_slice_index_csv: Path
    slice_count: int


def doc_path_from_anchor(anchor: str) -> str:
    # anchor is like docs/src/foo.md:123
    return anchor.split(":", 1)[0]


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        verification_matrix_csv=root / "verification-matrix.csv",
        out_subagents_dir=root / "subagents",
        out_slice_index_csv=root / "subagents" / "slice-index.csv",
        slice_count=18,
    )

    if not inputs.verification_matrix_csv.exists():
        raise SystemExit(f"missing input: {inputs.verification_matrix_csv}")

    inputs.out_subagents_dir.mkdir(parents=True, exist_ok=True)

    with inputs.verification_matrix_csv.open("r", encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        claims = list(reader)

    # Group claims by doc file, then assign whole files to slices (simplifies ownership).
    by_file: dict[str, list[dict[str, str]]] = {}
    for claim in claims:
        anchor = claim.get("anchor", "")
        file_path = doc_path_from_anchor(anchor)
        by_file.setdefault(file_path, []).append(claim)

    files_by_size = sorted(by_file.items(), key=lambda kv: len(kv[1]), reverse=True)

    slices: list[list[tuple[str, list[dict[str, str]]]]] = [[] for _ in range(inputs.slice_count)]
    slice_sizes = [0 for _ in range(inputs.slice_count)]

    # Greedy load-balance by number of claims.
    for file_path, file_claims in files_by_size:
        idx = min(range(inputs.slice_count), key=lambda i: slice_sizes[i])
        slices[idx].append((file_path, file_claims))
        slice_sizes[idx] += len(file_claims)

    # Write slices and index.
    index_rows: list[tuple[str, str]] = []
    for i, slice_files in enumerate(slices, start=1):
        slice_id = f"slice-{i:02d}"
        out_path = inputs.out_subagents_dir / f"{slice_id}.md"

        lines: list[str] = []
        lines.append(f"# {slice_id}")
        lines.append("")
        lines.append("## Ownership")
        lines.append("")
        if not slice_files:
            lines.append("- (empty slice; should not happen)")
        else:
            for file_path, file_claims in slice_files:
                lines.append(f"- `{file_path}` ({len(file_claims)} claims)")
        lines.append("")
        lines.append("## Instructions (skeptical, fail-closed)")
        lines.append("")
        lines.append("- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.")
        lines.append("- For each claim below, output a row with:")
        lines.append("  - `claim_id`")
        lines.append("  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`")
        lines.append("  - `evidence_anchor` (file path + symbol and/or test name + line where possible)")
        lines.append("  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)")
        lines.append("- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.")
        lines.append("")
        lines.append("## Claims")
        lines.append("")

        for file_path, file_claims in slice_files:
            # stable per-file ordering
            file_claims_sorted = sorted(
                file_claims,
                key=lambda c: (
                    int(c.get("anchor", "0").split(":")[-1]) if ":" in c.get("anchor", "") else 0,
                    c.get("claim_id", ""),
                ),
            )
            lines.append(f"### `{file_path}`")
            lines.append("")
            for claim in file_claims_sorted:
                claim_id = claim.get("claim_id", "")
                anchor = claim.get("anchor", "")
                claim_type = claim.get("claim_type", "")
                severity = claim.get("severity", "")
                expected = claim.get("expected_evidence_type", "")
                text = claim.get("claim_text", "")

                lines.append(
                    f"- `{claim_id}` | `{anchor}` | `{severity}` | `{claim_type}` | expected: `{expected}` | {text}"
                )
                index_rows.append((claim_id, slice_id))
            lines.append("")

        out_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")

    with inputs.out_slice_index_csv.open("w", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["claim_id", "slice_id"])
        for claim_id, slice_id in sorted(index_rows):
            writer.writerow([claim_id, slice_id])

    print(f"wrote slices: {inputs.out_subagents_dir} ({inputs.slice_count} slices)")
    print(f"wrote: {inputs.out_slice_index_csv} ({len(index_rows)} claim rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

