#!/usr/bin/env python3
from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    claim_inventory_csv: Path
    out_verification_matrix_csv: Path


def default_evidence_for(anchor: str) -> tuple[str, str]:
    # Returns (expected_evidence_type, verification_method)
    if "/interfaces/node-api.md:" in anchor:
        return ("BDD black-box,unit test", "tests/bdd_api_http.rs + src/api/* unit tests")
    if "/interfaces/cli.md:" in anchor:
        return ("real-binary e2e,unit test", "tests/cli_binary.rs + src/cli/* unit tests")
    if "/quick-start/" in anchor:
        return ("real-binary e2e,runtime log evidence", "run harness/CLI and capture logs")
    if "/operator/" in anchor:
        return ("code symbol,unit test", "src/config/* + harness config tests")
    if "/lifecycle/" in anchor:
        return ("real-binary e2e,code symbol", "src/ha/e2e_* + src/ha/* decision/worker")
    if "/assurance/" in anchor:
        return ("code symbol,unit test", "trace symbol ownership + unit tests")
    if "/contributors/" in anchor:
        return ("code symbol", "cross-check referenced modules/tests")
    if "/start-here/" in anchor or "/introduction.md:" in anchor:
        return ("code symbol,real-binary e2e", "confirm high-level claims via code entry points + e2e")
    if "/concepts/" in anchor:
        return ("code symbol", "validate definitions against code semantics")
    return ("code symbol", "trace symbols + tests")


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        claim_inventory_csv=root / "claim-inventory.csv",
        out_verification_matrix_csv=root / "verification-matrix.csv",
    )

    if not inputs.claim_inventory_csv.exists():
        raise SystemExit(f"missing input: {inputs.claim_inventory_csv}")

    with inputs.claim_inventory_csv.open("r", encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        rows = list(reader)

    out_fieldnames = [
        "claim_id",
        "anchor",
        "section",
        "claim_type",
        "severity",
        "claim_text",
        "expected_evidence_type",
        "verification_method",
        "pass_criteria",
        "status",
        "evidence_anchor",
        "notes",
        "original_anchor",
    ]

    out_rows: list[dict[str, str]] = []
    for row in rows:
        anchor = row.get("anchor", "")
        expected_evidence_type = row.get("expected_evidence_type", "").strip()
        verification_method = row.get("verification_method", "").strip()

        if not expected_evidence_type or not verification_method:
            default_evidence, default_method = default_evidence_for(anchor)
            expected_evidence_type = expected_evidence_type or default_evidence
            verification_method = verification_method or default_method

        out_row = dict(row)
        out_row["expected_evidence_type"] = expected_evidence_type
        out_row["verification_method"] = verification_method
        out_rows.append(out_row)

    with inputs.out_verification_matrix_csv.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=out_fieldnames)
        writer.writeheader()
        for row in out_rows:
            writer.writerow({k: row.get(k, "") for k in out_fieldnames})

    print(f"wrote: {inputs.out_verification_matrix_csv} ({len(out_rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

