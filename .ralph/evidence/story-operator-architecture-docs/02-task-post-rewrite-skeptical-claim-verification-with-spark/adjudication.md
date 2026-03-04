# Adjudication Log (Task 02)

This document records how conflicting/uncertain claim findings were resolved, and provides a compact “final disposition” summary for the generated claim inventory and verification matrix.

## Coverage and conflict handling

- Claim set: `165` total claim IDs (including `removed` claims retained for audit trail).
- Slicing: 18 disjoint subagent slices; machine-checked to cover every claim ID exactly once.
- Conflicts: no cross-slice conflicts (each claim ID had a single owner). Any “uncertain” outcomes were resolved by either bounding doc wording or downgrading/removing the claim.

## Orphans / scope

- All orphan markdown under `docs/src/` (not linked from `docs/src/SUMMARY.md`) was treated as legacy in this greenfield repo and removed.
- Rationale and the removal list are recorded in `orphan-docs-triage.md`.

## Anchor refresh after doc rewrites

Because multiple operator-facing docs were rewritten during adjudication (to remove over-claims or to use bounded language), `path:line` anchors and `claim_text` were refreshed to match the final doc state:

- `scripts/refresh_claim_anchors.py` updates anchors where a unique match exists.
- It also refreshes `claim_text` from the anchored file line to ensure the inventory is self-consistent.
- A consistency check was run to ensure every non-removed claim’s `claim_text` exactly matches the referenced file line.

## Removed “claims” that were not claims

- Several inventory entries originally pointed at headings or contributor-process guidance rather than operator-facing behavioral claims.
- These were reclassified as `removed` (or corrected to the actual claim line when the surrounding section contained a real claim).

## Final disposition summary

- `verified`: 110
- `rewritten`: 44
- `removed`: 11

No claims remain in `uncertain-with-followup` state after bounding/rewording.

## Notes on “rewritten” status

Rows marked `rewritten` are treated as resolved by documentation bounding:

- They intentionally avoid hard guarantees that are not enforced by explicit guards/tests.
- They remain operator-useful as guidance (symptoms → likely causes → first checks) without asserting strong safety properties.

