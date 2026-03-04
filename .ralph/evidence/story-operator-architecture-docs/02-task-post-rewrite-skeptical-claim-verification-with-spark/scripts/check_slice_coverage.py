#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Inputs:
    verification_matrix_csv: Path
    slice_index_csv: Path
    subagents_dir: Path
    require_outputs: bool


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--require-outputs", action="store_true")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    inputs = Inputs(
        verification_matrix_csv=root / "verification-matrix.csv",
        slice_index_csv=root / "subagents" / "slice-index.csv",
        subagents_dir=root / "subagents",
        require_outputs=bool(args.require_outputs),
    )

    for path in [inputs.verification_matrix_csv, inputs.slice_index_csv]:
        if not path.exists():
            raise SystemExit(f"missing input: {path}")

    with inputs.verification_matrix_csv.open("r", encoding="utf-8", newline="") as f:
        claims = list(csv.DictReader(f))
    claim_ids = [row.get("claim_id", "").strip() for row in claims]
    claim_id_set = {cid for cid in claim_ids if cid}
    if len(claim_id_set) != len(claim_ids):
        raise SystemExit("verification-matrix.csv contains blank/duplicate claim_id values")

    with inputs.slice_index_csv.open("r", encoding="utf-8", newline="") as f:
        index_rows = list(csv.DictReader(f))

    index_claim_ids = [row.get("claim_id", "").strip() for row in index_rows]
    index_slice_ids = [row.get("slice_id", "").strip() for row in index_rows]

    missing_in_index = sorted(claim_id_set.difference(index_claim_ids))
    extra_in_index = sorted(set(index_claim_ids).difference(claim_id_set))
    dupes = [cid for cid, count in Counter(index_claim_ids).items() if count != 1]

    if missing_in_index:
        raise SystemExit(f"missing claim_id in slice-index.csv: {missing_in_index[:10]} (and {len(missing_in_index)-10} more)" if len(missing_in_index) > 10 else f"missing claim_id in slice-index.csv: {missing_in_index}")
    if extra_in_index:
        raise SystemExit(f"slice-index.csv contains unknown claim_id values: {extra_in_index[:10]} (and {len(extra_in_index)-10} more)" if len(extra_in_index) > 10 else f"slice-index.csv contains unknown claim_id values: {extra_in_index}")
    if dupes:
        raise SystemExit(f"slice-index.csv claim_id not exactly once: {dupes[:10]} (and {len(dupes)-10} more)" if len(dupes) > 10 else f"slice-index.csv claim_id not exactly once: {dupes}")

    # Basic slice sanity: ensure >= 15 non-empty slices.
    by_slice: dict[str, list[str]] = defaultdict(list)
    for cid, sid in zip(index_claim_ids, index_slice_ids, strict=True):
        by_slice[sid].append(cid)
    non_empty_slices = [sid for sid, cids in by_slice.items() if cids]
    if len(non_empty_slices) < 15:
        raise SystemExit(f"need >= 15 non-empty slices; got {len(non_empty_slices)}")

    # Ensure slice definition files exist.
    missing_slice_defs = []
    for sid in sorted(non_empty_slices):
        def_path = inputs.subagents_dir / f"{sid}.md"
        if not def_path.exists():
            missing_slice_defs.append(str(def_path))
    if missing_slice_defs:
        raise SystemExit(f"missing slice definition files: {missing_slice_defs}")

    if inputs.require_outputs:
        missing_outputs = []
        for sid in sorted(non_empty_slices):
            out_path = inputs.subagents_dir / f"output-{sid}.md"
            if not out_path.exists():
                missing_outputs.append(str(out_path))
        if missing_outputs:
            raise SystemExit(
                f"missing required subagent output files ({len(missing_outputs)}): {missing_outputs[:10]}"
            )

    print(f"ok: {len(claim_id_set)} claims covered exactly once across {len(non_empty_slices)} slices")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

