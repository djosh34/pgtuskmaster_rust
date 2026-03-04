#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    claim_inventory_csv: Path
    verification_matrix_csv: Path
    subagents_dir: Path
    out_claim_inventory_csv: Path
    out_verification_matrix_csv: Path
    out_claim_coverage_check_txt: Path


STATUS_ALLOWED = {
    "verified",
    "rewritten",
    "removed",
    "uncertain-with-followup",
}


def strip_md(s: str) -> str:
    out = s.strip()
    # remove surrounding backticks
    out = out.strip("`").strip()
    return out


def normalize_claim_id(raw: str) -> str:
    s = strip_md(raw)
    s = s.strip()
    # allow "0091" style
    if re.fullmatch(r"\d{1,4}", s):
        return f"claim-{int(s):04d}"
    m = re.fullmatch(r"claim-(\d{1,4})", s)
    if m:
        return f"claim-{int(m.group(1)):04d}"
    return s


def normalize_status(raw: str) -> str:
    s = strip_md(raw).strip().lower()
    return s


def parse_markdown_table(path: Path) -> list[dict[str, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    rows: list[dict[str, str]] = []
    for line in lines:
        if not line.strip().startswith("|"):
            continue
        # skip separator row
        if re.fullmatch(r"\s*\|\s*-+\s*\|\s*-+\s*\|\s*-+\s*\|\s*-+\s*\|\s*", line):
            continue
        # split markdown table row
        parts = [p.strip() for p in line.strip().strip("|").split("|")]
        if len(parts) < 4:
            continue
        if parts[0].strip() == "claim_id":
            continue
        claim_id, status, evidence_anchor, notes = parts[0], parts[1], parts[2], "|".join(parts[3:]).strip()
        rows.append(
            {
                "claim_id": normalize_claim_id(claim_id),
                "status": normalize_status(status),
                "evidence_anchor": evidence_anchor.strip(),
                "notes": notes.strip(),
            }
        )
    return rows


def read_csv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open("r", encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        fieldnames = list(reader.fieldnames or [])
        rows = list(reader)
    return fieldnames, rows


def write_csv(path: Path, fieldnames: list[str], rows: list[dict[str, str]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow({k: row.get(k, "") for k in fieldnames})


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        claim_inventory_csv=root / "claim-inventory.csv",
        verification_matrix_csv=root / "verification-matrix.csv",
        subagents_dir=root / "subagents",
        out_claim_inventory_csv=root / "claim-inventory.csv",
        out_verification_matrix_csv=root / "verification-matrix.csv",
        out_claim_coverage_check_txt=root / "claim-coverage-check.txt",
    )

    for required in [inputs.claim_inventory_csv, inputs.verification_matrix_csv, inputs.subagents_dir]:
        if not required.exists():
            raise SystemExit(f"missing input: {required}")

    inv_fields, inv_rows = read_csv(inputs.claim_inventory_csv)
    mat_fields, mat_rows = read_csv(inputs.verification_matrix_csv)

    inv_by_id = {row.get("claim_id", ""): row for row in inv_rows}
    mat_by_id = {row.get("claim_id", ""): row for row in mat_rows}

    output_files = sorted(inputs.subagents_dir.glob("output-slice-*.md"))
    if not output_files:
        raise SystemExit(f"no output files found under {inputs.subagents_dir}")

    merged: dict[str, dict[str, str]] = {}
    for out_path in output_files:
        for row in parse_markdown_table(out_path):
            claim_id = row["claim_id"]
            if not claim_id.startswith("claim-"):
                raise SystemExit(f"{out_path}: unexpected claim_id format: {claim_id!r}")
            if row["status"] not in STATUS_ALLOWED:
                raise SystemExit(f"{out_path}: invalid status {row['status']!r} for {claim_id}")
            if claim_id in merged:
                raise SystemExit(f"duplicate claim_id across outputs: {claim_id}")
            merged[claim_id] = row

    missing = sorted(set(inv_by_id.keys()).difference(merged.keys()))
    extra = sorted(set(merged.keys()).difference(inv_by_id.keys()))
    if missing:
        raise SystemExit(f"missing outputs for claim_ids (first 10): {missing[:10]}")
    if extra:
        raise SystemExit(f"outputs contain unknown claim_ids (first 10): {extra[:10]}")

    status_counts: dict[str, int] = {s: 0 for s in sorted(STATUS_ALLOWED)}
    for claim_id, result in merged.items():
        status = result["status"]
        status_counts[status] = status_counts.get(status, 0) + 1

        for table in [inv_by_id, mat_by_id]:
            row = table.get(claim_id)
            if row is None:
                continue
            row["status"] = status
            # Preserve any pre-filled evidence metadata, but overwrite if the agent provided it.
            if result["evidence_anchor"]:
                row["evidence_anchor"] = result["evidence_anchor"]
            if result["notes"]:
                row["notes"] = result["notes"]

    write_csv(inputs.out_claim_inventory_csv, inv_fields, inv_rows)
    write_csv(inputs.out_verification_matrix_csv, mat_fields, mat_rows)

    unverified = [row for row in inv_rows if row.get("status", "").strip() == "unverified"]
    if unverified:
        raise SystemExit(f"claim-inventory.csv still contains unverified rows: {len(unverified)}")

    lines = []
    lines.append("claim coverage check")
    lines.append("")
    lines.append(f"output files: {len(output_files)}")
    lines.append(f"claims: {len(merged)}")
    lines.append("")
    for status in sorted(status_counts.keys()):
        lines.append(f"- {status}: {status_counts[status]}")
    lines.append("")
    inputs.out_claim_coverage_check_txt.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"wrote: {inputs.out_claim_coverage_check_txt}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

