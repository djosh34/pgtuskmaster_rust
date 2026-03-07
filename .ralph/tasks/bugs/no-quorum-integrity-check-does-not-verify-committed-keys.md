## Bug: No-quorum integrity check does not verify committed keys <status>done</status> <passes>true</passes>

<description>
`tests/ha/support/multi_node.rs` records `committed_writes`, `committed_keys`, and commit timestamps during `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`, but the final verification only calls `assert_table_key_integrity_strict(...)`, which succeeds when any reachable node reports `COUNT(*) >= min_rows` and no duplicate `(worker_id, seq)` rows.

That means the scenario can pass without proving that the recovered table contains exactly the keys that were acknowledged before fencing. It does not detect lost committed rows, unexpected extra rows, or divergence between the in-memory workload ledger and the post-recovery table contents.

Explore and research the codebase first, then fix. Focus on restoring a real post-recovery path for this scenario and adding an assertion that compares the recovered table contents against the expected committed key set captured by the workload.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<execution_plan>
## Detailed Execution Plan (Draft 2, 2026-03-07)

1. Re-establish the actual baseline before touching code or task state
- The task description describes a real gap, but the current `HEAD` and `origin/master` already contain the intended fix in `tests/ha/support/multi_node.rs` via commit `369369a`.
- The execution pass must treat this as a stale-task-state problem until targeted verification proves otherwise. Do not begin by re-designing the feature or searching for alternate implementations.
- Concretely verify these existing baseline facts in the execution pass without broad re-exploration:
  - `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity(...)` computes `required_committed_keys` from the workload ledger through the fencing cutoff and `allowed_committed_keys` from all acknowledged commits.
  - recovery verification happens after restoring quorum and waiting for a recovered stable primary.
  - the scenario calls `assert_table_recovery_key_integrity_on_node(...)` instead of relying only on `assert_table_key_integrity_strict(...)`.
  - helper coverage exists for duplicates, missing required keys, unexpected keys, and cutoff alignment.
- Because `HEAD` already matches `origin/master`, the execution pass should spend its first energy on proving the landed behavior still passes all gates, then only change code if verification reveals a real regression or hidden gap.

2. Define the invariant that execution must validate and preserve
- The recovered table must contain every key acknowledged at or before the fencing cutoff.
- The recovered table must not contain any key outside the acknowledged workload ledger.
- Duplicate `(worker_id, seq)` rows remain a hard failure.
- Post-cutoff writes may still be tolerated only if they are already recorded in the workload ledger and therefore fall inside the allowed committed-key superset.
- The scenario must validate the recovered primary directly after recovery, not accept success from any random reachable node whose row count happens to meet a threshold.

3. Only if targeted verification disproves the landed implementation, patch the code in one coherent pass
- Do not make speculative edits just because the task description is stale.
- If verification shows the current code is insufficient, then update the no-quorum fencing scenario to restore a real post-recovery verification path:
  - stop the workload
  - compute the fencing cutoff
  - derive the required pre-cutoff committed-key set from the workload’s per-worker key/timestamp ledger
  - restore the stopped etcd members
  - wait for a stable recovered primary
  - query the recovered table contents from that node
- If verification shows the helper is insufficient, replace or supplement row-count-only verification with an exact key-set assertion that:
  - rejects duplicate observed rows
  - reports missing required keys
  - reports unexpected keys outside the committed ledger
  - returns the observed row count only after those invariants pass
- Keep all error handling explicit. Do not add `unwrap`, `expect`, panics, or swallowed errors.

4. Preserve and, only if needed, extend focused test coverage around the key-set logic
- Confirm the existing focused unit tests already cover the helper that validates recovered keys:
  - pass when the recovered table contains all required keys plus only allowed post-cutoff extras
  - fail on duplicate rows
  - fail when a required committed key is missing
  - fail when an unexpected key appears
- Confirm the existing focused test that proves cutoff calculation respects per-worker timestamp/key alignment rather than assuming global ordering.
- Add or change tests only if execution reveals a real missing assertion, broken expectation, or API mismatch.

5. Sweep docs and task-state references for stale semantics
- Search contributor docs and verification notes for references that still describe this scenario as row-count-only or otherwise stale.
- Current review found the relevant invariant references only in `tests/ha/support/multi_node.rs`, so execution should still run the doc search but should expect “no external doc changes needed” unless a hidden stale reference appears.
- The task markdown itself must be brought into sync with the actual work performed; leaving the task in `not_started` after the code is already fixed is not acceptable.

6. Execute verification in a strict order once code and docs are settled
- Run focused tests first so failures are interpreted near the changed area:
  - the relevant `tests/ha/support/multi_node.rs` unit tests around committed-key recovery bounds and cutoff derivation
  - the no-quorum HA scenario test entry point if execution changed scenario wiring or behavior
- Then run the required repo gates in this exact order:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Do not mark the task complete, set `<passes>true</passes>`, switch tasks, commit, or push until every required gate is green.

7. Completion protocol for the later execution pass
- Tick the acceptance boxes only after the required gates succeed.
- Set the task header to the final completed state and set `<passes>true</passes>` only after all four commands pass.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit every relevant change, including `.ralph` state files, with a message of the form:
  - `task finished no-quorum-integrity-check-does-not-verify-committed-keys: ...`
- The commit message must summarize:
  - whether the turn performed code changes or verified an already-landed implementation
  - the recovered-key invariant that is now enforced
  - the exact verification commands that passed
  - any non-obvious issue encountered, such as stale task state or documentation cleanup
- Push with `git push`.

</execution_plan>

## Execution Notes (2026-03-07)

- Targeted verification disproved the stale-task assumption: the recovered-key helper was present, but `make test-long` exposed a real regression in `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`.
- The recovered primary was losing acknowledged pre-fencing keys because the fail-safe recovery path kept routing a restored-quorum node back through release/fence behavior instead of re-entering the normal primary decision path.
- Fixed the HA decision flow so a node that is still PostgreSQL primary when quorum returns exits fail-safe into the normal primary logic and attempts to reacquire leadership instead of re-fencing itself.
- Updated HA unit tests and worker contract coverage to reflect the new blocking point and published state: leadership reacquisition rather than release-leader blocking.
- No external docs required changes; the stale artifact was the task state itself.

NOW EXECUTE
