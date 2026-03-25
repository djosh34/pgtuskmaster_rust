## Plan: Promote HA Polling Driver Into Shared Test Support

### Why this is the next reduction target

The HA test support code still repeats the same polling boundary across several owners even though one usable implementation already exists:

- `tests/ha/support/steps/mod.rs` already has `poll_until(...)`, which owns deadline calculation, poll interval sleeping, last-error tracking, and timeout formatting.
- `tests/ha/support/world/mod.rs` still open-codes the same retry loop in `wait_for_service_health(...)` and `wait_for_seed_primary(...)`.
- `tests/ha/support/invariants/write_convergence.rs` still open-codes the same retry/deadline loop in `wait_for_convergence(...)`, plus the integration-test helpers `wait_for_row_count_at_least(...)` and `wait_for_postgres_ready(...)`.

That is the exact kind of wrong-place duplication this reduce loop is supposed to remove: one polling algorithm is living in three owners with slightly different formatting instead of one canonical support helper plus small caller-specific wrappers.

This slice is a stronger reduction target than another file-local cleanup because it can delete repeated retry scaffolding from multiple large HA support files at once without inventing a new domain type.

### Current overlap already verified

- `tests/ha/support/steps/mod.rs:676` owns a local `poll_until(...)` helper that already proves the minimum viable abstraction is enough.
- `tests/ha/support/world/mod.rs:677` and `tests/ha/support/world/mod.rs:707` each recreate:
  - `let deadline = Instant::now() + ...`
  - `let mut last_error = None`
  - a retry loop with `tokio::time::sleep(...)`
  - a timeout error that uses the last observed failure text
- `tests/ha/support/invariants/write_convergence.rs:636`, `tests/ha/support/invariants/write_convergence.rs:1610`, and `tests/ha/support/invariants/write_convergence.rs:1631` each recreate the same deadline/sleep loop shape with only caller-specific attempt logic and timeout messages changed.
- The repeated boundary is mechanical retry orchestration, not cluster-domain logic.

### Execution plan

1. Move the canonical polling driver out of `steps` and into shared HA support.
   - Promote the existing `poll_until(...)` shape into `tests/ha/support/mod.rs` or a new sibling support module owned by `tests/ha/support`.
   - Keep the helper generic but narrow:
     - one deadline window
     - one poll interval
     - one retry body
     - one timeout formatter
   - Reuse the existing semantics instead of inventing a new polling enum or result wrapper.

2. Keep `steps` as a thin caller-specific wrapper.
   - Rebuild `tests/ha/support/steps/mod.rs::poll_until(...)` as a small adapter around the shared helper so it can preserve the existing terminal-container-failure check.
   - Keep `terminal_container_failure(...)` and step-specific timeout text in `steps`; only the generic retry mechanics should move.

3. Collapse the hand-rolled world bootstrap loops onto the shared helper.
   - Rewrite `wait_for_service_health(...)` to supply only the service-health attempt and timeout message.
   - Rewrite `wait_for_seed_primary(...)` to supply only the observer/status attempt and timeout message.
   - Preserve the existing status snapshot recording and error text.

4. Collapse the write-convergence retry loops onto the shared helper where it is a net deletion.
   - Rebuild `wait_for_convergence(...)` on top of the shared helper.
   - Rebuild test-only helpers `wait_for_row_count_at_least(...)` and `wait_for_postgres_ready(...)` on top of the same helper.
   - Only migrate additional loops if they delete code without forcing a more elaborate abstraction.

5. Only extend to `primary_count` if it remains obviously smaller afterward.
   - If `PrimaryCountInvariantRunner::ensure_healthy(...)` can reuse the same helper with fewer lines and no async adapter clutter, fold it in.
   - If not, leave `primary_count.rs` alone for this slice.

6. Rebuild focused tests around the shared owner.
   - Keep the existing `steps` coverage intact.
   - Update `world` tests if needed so timeout/error behavior remains stable.
   - Keep the real-postgres write-convergence tests and helper-specific timeout behavior intact.

7. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to improve beyond the current validated baseline.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse the existing polling shape; do not introduce a new enum taxonomy for poll modes.
- Do not move cluster-domain interpretation out of callers just to feed the helper.
- Keep the terminal-container-failure handling in `steps`; that is caller policy, not generic polling.
- If sharing the helper starts requiring boxed async trait objects, extra state couriers, or placeholder closures that make the callers bigger, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete one local polling implementation from `steps` as the canonical owner.
- Delete two hand-rolled bootstrap polling loops from `world`.
- Delete up to three retry loops from `write_convergence`, including test-only fixture wait helpers.
- Flatten HA test support around one existing retry boundary instead of keeping parallel loop implementations.

### Revalidation note

The implementation based on this plan did pass `make check`, `make lint`, `make test`, and `make test-long`, but it did not produce a net line reduction in the touched code. The next turn should either:

- shrink the shared helper and its callers enough to make this slice net-negative, likely by folding in `primary_count` and/or simplifying the steps adapter; or
- abandon this slice and write a different plan with a larger guaranteed deletion target.

TO BE VERIFIED
