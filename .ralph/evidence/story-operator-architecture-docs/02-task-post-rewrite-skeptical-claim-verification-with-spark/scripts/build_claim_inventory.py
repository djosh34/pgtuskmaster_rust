#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    claim_candidates_txt: Path
    out_claim_inventory_csv: Path


CANDIDATE_RE = re.compile(r"^(?P<path>[^:]+):(?P<line>\d+):(?P<text>.*)$")


def severity_for(text: str) -> str:
    lower = text.lower()
    high_markers = [
        "guarantee",
        "ensures",
        "never",
        "cannot",
        "impossible",
        "prevents",
        "split-brain",
        "split brain",
        "fence",
        "fencing",
        "fail-safe",
        "failsafe",
        "safety",
        "invariant",
    ]
    medium_markers = ["must", "only", "will", "require", "forbid", "always"]
    if any(m in lower for m in high_markers):
        return "high"
    if any(m in lower for m in medium_markers):
        return "medium"
    return "low"


def claim_type_for(text: str) -> str:
    lower = text.lower()
    if any(m in lower for m in ["never", "cannot", "impossible", "prevents", "ensures", "invariant"]):
        return "invariant"
    if any(m in lower for m in ["must", "only", "will", "require", "forbid", "always"]):
        return "behavioral"
    # default: descriptive / definitional
    return "descriptive"


def section_for(path: str) -> str:
    # path is like docs/src/<section>/...
    parts = path.split("/")
    if len(parts) >= 3:
        return parts[2]
    return "unknown"


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        claim_candidates_txt=root / "claim-candidates.txt",
        out_claim_inventory_csv=root / "claim-inventory.csv",
    )

    if not inputs.claim_candidates_txt.exists():
        raise SystemExit(f"missing input: {inputs.claim_candidates_txt}")

    candidates = inputs.claim_candidates_txt.read_text(encoding="utf-8").splitlines()

    out_rows: list[dict[str, str]] = []
    next_id = 1
    for raw in candidates:
        if not raw.strip():
            continue
        m = CANDIDATE_RE.match(raw)
        if not m:
            raise SystemExit(f"bad candidate line (expected path:line:text): {raw!r}")

        path = m.group("path")
        line = m.group("line")
        text = m.group("text").strip()
        anchor = f"{path}:{line}"

        claim_id = f"claim-{next_id:04d}"
        next_id += 1

        out_rows.append(
            {
                "claim_id": claim_id,
                "anchor": anchor,
                "section": section_for(path),
                "claim_type": claim_type_for(text),
                "severity": severity_for(text),
                "claim_text": text,
                "expected_evidence_type": "",
                "verification_method": "",
                "pass_criteria": "",
                "status": "unverified",
                "evidence_anchor": "",
                "notes": "",
                "original_anchor": anchor,
            }
        )

    fieldnames = [
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

    with inputs.out_claim_inventory_csv.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for row in out_rows:
            writer.writerow(row)

    print(f"wrote: {inputs.out_claim_inventory_csv} ({len(out_rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

