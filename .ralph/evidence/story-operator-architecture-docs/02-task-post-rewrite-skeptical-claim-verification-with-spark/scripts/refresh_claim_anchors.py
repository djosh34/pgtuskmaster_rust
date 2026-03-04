#!/usr/bin/env python3
from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    claim_inventory_csv: Path
    verification_matrix_csv: Path


REPLACEMENT_PREFIXES = [
    "replace with:",
    "replacement wording:",
    "rewritten replacement:",
    "rewritten exact replacement:",
    "rewrite to:",
    "rewritten:",
]


def extract_replacement(notes: str) -> str | None:
    lower = notes.lower()
    for prefix in REPLACEMENT_PREFIXES:
        idx = lower.find(prefix)
        if idx == -1:
            continue
        tail = notes[idx + len(prefix) :].strip()
        # drop leading punctuation/quotes
        tail = tail.lstrip(" \t-–—:").strip()
        # strip surrounding quotes/backticks
        tail = tail.strip().strip("`").strip()
        tail = tail.strip("“”\"'")
        # keep only first line (often replacement is a single sentence)
        tail = tail.splitlines()[0].strip()
        return tail or None
    return None


def find_unique_line(path: Path, needle: str) -> int | None:
    if not needle:
        return None
    lines = path.read_text(encoding="utf-8").splitlines()
    matches = []
    for idx, line in enumerate(lines, start=1):
        if needle in line:
            matches.append(idx)
    if len(matches) == 1:
        return matches[0]
    return None


def update_rows(rows: list[dict[str, str]]) -> tuple[int, int]:
    updated = 0
    unchanged = 0
    for row in rows:
        anchor = row.get("anchor", "")
        if ":" not in anchor:
            unchanged += 1
            continue
        path_s, _line_s = anchor.rsplit(":", 1)
        path = Path(path_s)
        if not path.exists():
            unchanged += 1
            continue

        if row.get("status") == "removed":
            unchanged += 1
            continue

        candidates: list[str] = []
        claim_text = row.get("claim_text", "").strip()
        if claim_text:
            candidates.append(claim_text)

        notes = row.get("notes", "")
        repl = extract_replacement(notes)
        if repl and repl not in candidates:
            candidates.append(repl)

        new_line = None
        new_needle = None
        for needle in candidates:
            line = find_unique_line(path, needle)
            if line is not None:
                new_line = line
                new_needle = needle
                break

        if new_line is None:
            unchanged += 1
            continue

        new_anchor = f"{path_s}:{new_line}"
        if new_anchor == anchor:
            unchanged += 1
            continue

        # preserve old anchor in original_anchor if not already preserved
        if not row.get("original_anchor"):
            row["original_anchor"] = anchor
        row["anchor"] = new_anchor

        # If we matched on replacement text, update claim_text to what is now in the file.
        if new_needle and new_needle != claim_text:
            file_lines = path.read_text(encoding="utf-8").splitlines()
            row["claim_text"] = file_lines[new_line - 1].strip()

        updated += 1

    return updated, unchanged


def refresh_claim_text_from_anchor(rows: list[dict[str, str]]) -> int:
    refreshed = 0
    for row in rows:
        if row.get("status") == "removed":
            continue
        anchor = row.get("anchor", "")
        if ":" not in anchor:
            continue
        path_s, line_s = anchor.rsplit(":", 1)
        path = Path(path_s)
        if not path.exists():
            continue
        try:
            line_no = int(line_s)
        except ValueError:
            continue
        if line_no <= 0:
            continue
        file_lines = path.read_text(encoding="utf-8").splitlines()
        if line_no > len(file_lines):
            continue
        new_text = file_lines[line_no - 1].strip()
        if not new_text:
            continue
        if row.get("claim_text", "").strip() == new_text:
            continue
        row["claim_text"] = new_text
        refreshed += 1
    return refreshed


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        claim_inventory_csv=root / "claim-inventory.csv",
        verification_matrix_csv=root / "verification-matrix.csv",
    )

    inv_rows: list[dict[str, str]]
    with inputs.claim_inventory_csv.open("r", encoding="utf-8", newline="") as f:
        inv_reader = csv.DictReader(f)
        inv_fields = inv_reader.fieldnames or []
        inv_rows = list(inv_reader)

    mat_rows: list[dict[str, str]]
    with inputs.verification_matrix_csv.open("r", encoding="utf-8", newline="") as f:
        mat_reader = csv.DictReader(f)
        mat_fields = mat_reader.fieldnames or []
        mat_rows = list(mat_reader)

    inv_by_id = {r.get("claim_id", ""): r for r in inv_rows}
    mat_by_id = {r.get("claim_id", ""): r for r in mat_rows}

    updated_inv, _ = update_rows(inv_rows)
    refreshed_inv = refresh_claim_text_from_anchor(inv_rows)

    # mirror anchor/claim_text updates into verification-matrix for consistency
    for claim_id, inv_row in inv_by_id.items():
        mat_row = mat_by_id.get(claim_id)
        if not mat_row:
            continue
        mat_row["anchor"] = inv_row.get("anchor", mat_row.get("anchor", ""))
        mat_row["claim_text"] = inv_row.get("claim_text", mat_row.get("claim_text", ""))
        mat_row["original_anchor"] = inv_row.get("original_anchor", mat_row.get("original_anchor", ""))

    with inputs.claim_inventory_csv.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=inv_fields)
        w.writeheader()
        w.writerows(inv_rows)

    with inputs.verification_matrix_csv.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=mat_fields)
        w.writeheader()
        w.writerows(mat_rows)

    print(f"updated anchors: {updated_inv}")
    print(f"refreshed claim_text: {refreshed_inv}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
