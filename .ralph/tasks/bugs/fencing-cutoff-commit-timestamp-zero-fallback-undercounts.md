---
## Bug: Fencing cutoff commit timestamp fallback undercounts post-cutoff commits <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
In `src/ha/e2e_multi_node.rs`, successful SQL commits record `committed_at_unix_ms` using `ha_e2e::util::unix_now()`, but on error the code falls back to `0` (`Err(_) => 0`).

The no-quorum fencing assertion computes post-cutoff commits using `timestamp > cutoff_ms`. Any commit with fallback timestamp `0` is silently excluded, which can undercount post-cutoff commits and weaken (or falsely pass) the safety assertion.

Please explore and research the codebase first, then implement a fail-closed fix that does not use unwrap/panic/expect:
- avoid sentinel `0` timestamps for committed writes,
- either propagate timestamp capture failures or explicitly fail sampling/assertion when timestamps are incomplete,
- consider a monotonic-time based alternative for cutoff comparisons,
- add/update focused tests for regression coverage.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan (Skeptically verified; ready to execute)

### 0) Scope reconciliation (ensure we fix the *real* current state)
- [x] Create `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/`.
- [x] Confirm the original bug is already fixed in `src/ha/e2e_multi_node.rs`:
  - [x] The SQL workload worker does **not** push sentinel `0` timestamps on `unix_now()` error; instead it increments `commit_timestamp_capture_failures` and treats this as a hard failure.
  - [x] The fencing cutoff assertion uses a fail-closed helper and counts strictly `timestamp > cutoff_ms`.
  - [x] Focused unit tests exist for:
    - [x] timestamp capture failure => error
    - [x] incomplete timestamps => error
    - [x] zero timestamp present => error
    - [x] strict `>` cutoff semantics
- [x] Repo-wide audit for *remaining* sentinel time fallbacks (prevent recurrence / adjacent false evidence):
  - [x] `rg -n "Err\\(_\\)\\s*=>\\s*0|unwrap_or\\(0\\)" src/ha src/test_harness`
  - [x] If any remaining `0` fallbacks are found, either:
    - [x] remove them (prefer `time_error:{err}` / explicit failure counters), or
    - [ ] file a follow-up bug task if they are unrelated to fencing correctness (do not silently ignore).

### 1) Decide: “closure-only” vs “cleanup included”
- [x] If the original bug is fully fixed (as expected), keep code changes minimal:
  - [x] Prefer closure-only (evidence + task bookkeeping) unless the repo-wide audit finds a *directly related* sentinel that could weaken fencing/cutoff assertions.
- [x] If the audit finds a related sentinel in HA e2e timing used for correctness gates:
  - [x] unify the behavior to fail-closed (no sentinel `0`) and add/extend tests accordingly.

### 2) Required gates + evidence (no skips; all are mandatory in this workspace)
- [x] `make check` (save full output under `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/`)
- [x] `make test` (save full output under evidence dir)
- [x] `make test-long` (save full output under evidence dir)
- [x] `make lint` (save full output under evidence dir)

### 3) Task closeout (only after all gates are green)
- [x] Update this task file:
  - [x] Mark acceptance criteria checkboxes.
  - [x] Set `<status>done</status>`, `<passes>true</passes>`, and add `<passing>true</passing>`.
  - [x] Summarize: why the bug no longer reproduces, the key safeguards (strict helper + timestamp failure counter), and the evidence log paths.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all changes (including `.ralph`) with message: `task finished fencing-cutoff-commit-timestamp-zero-fallback-undercounts: <summary + evidence>`.
- [ ] `git push`.

DONE

## Summary

- The original reported issue in `src/ha/e2e_multi_node.rs` was already fixed: committed writes never record sentinel `0` timestamps; timestamp capture failures are tracked and the cutoff assertion fails closed via `count_commits_after_cutoff_strict`.
- Cleaned up remaining sentinel-time fallbacks found by repo audit so HA e2e timelines and harness best-effort cleanup never silently substitute `0` for unknown time.

## Evidence

- `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/make-check.log`
- `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/make-test.log`
- `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/make-test-long.log`
- `.ralph/evidence/fencing-cutoff-commit-timestamp-zero-fallback-undercounts/make-lint.log`
