## Bug: HA e2e false-pass via best-effort polling and timestamp fallback <status>done</status> <passes>true</passes>

<description>
HA e2e assertions can pass without reliable cluster-wide observations during unstable windows.

Detected during review of plan section 2.4 against:
- src/ha/e2e_multi_node.rs:1395-1410 (`assert_no_dual_primary_window` ignores polling errors and returns success if every poll fails)
- src/ha/e2e_multi_node.rs:1412-1458 (`wait_for_all_failsafe` is explicitly best-effort but is used with all-node language in scenario logs)
- src/ha/e2e_multi_node.rs:2099-2100 (matrix scenario logs "fail-safe observed on all nodes" after calling `wait_for_all_failsafe`)
- src/ha/e2e_multi_node.rs:2484-2500 (post-failsafe verification path tolerates per-node poll errors and still can proceed)
- src/ha/e2e_multi_node.rs:1718-1721 (`unix_now()` failure mapped to `0` for commit timestamps)
- src/ha/e2e_multi_node.rs:2619-2629 (fencing cutoff assertion depends on commit timestamps; `0` fallback can undercount post-cutoff commits)

Ask the fixing agent to explore and research the codebase first, then implement a minimal, deterministic fix set.

Expected direction:
- Make dual-primary window assertions fail closed if sample reliability is insufficient (e.g. zero successful samples, excessive poll errors, or missing-node coverage).
- Align no-quorum helper semantics and scenario claim text: either require strict all-node observation or explicitly downgrade assertion wording + criteria.
- Remove/replace `unix_now()` fallback-to-0 in fencing-sensitive commit timestamp collection; propagate hard error or mark sample invalid and fail the assertion.
- Add/adjust tests to ensure these paths cannot silently pass under repeated API transport failures.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<plan>
## Plan

### 0) Scope and constraints
- Scope: fix false-pass behavior in HA e2e checks where API polling / timestamp sampling failures currently allow assertions to succeed without evidence.
- Constraint: keep changes minimal and deterministic; do not introduce panics/unwrap/expect; prefer fail-closed semantics when evidence is missing.
- Primary target file: `src/ha/e2e_multi_node.rs` (all nodes should be reachable in these scenarios).

### 1) Make split-brain window assertions fail closed (no “0 samples => pass”)
Problem:
- `ClusterFixture::assert_no_dual_primary_window` currently ignores polling errors and returns `Ok(())` at deadline even if all polls fail.
- It also uses `cluster_ha_states()` which (transitively) can block up to `E2E_HTTP_STEP_TIMEOUT` per poll, so a 3–6s “window” may contain zero usable samples.

Plan (skeptical adjustment: keep timeouts stable; fail closed on missing evidence):
- Update `assert_no_dual_primary_window(window)` to require at least one successful full-cluster observation:
  - Continue polling using `cluster_ha_states()` (keeps “full cluster” semantics: a missing node is treated as missing evidence, not “safe”).
  - Track `successful_samples` and `last_poll_error` (and optionally `poll_error_count` / `poll_attempts` for better error messages).
  - If any successful sample shows `>1` primary members, return `Err(...)` immediately.
  - At deadline: if `successful_samples == 0`, return `Err(...)` with `last_poll_error` detail (fail-closed instead of “0 samples => pass”); otherwise return `Ok(())`.
- Tighten `assert_no_dual_primary_in_samples(stats)`:
  - If `stats.sample_count == 0`, return `Err(...)` (“insufficient evidence; zero successful samples”).
  - Keep existing `max_concurrent_primaries > 1` check.

Notes (why this change vs shrinking timeouts / adding outer timeouts):
- Avoid wrapping `cluster_ha_states()` in an outer `tokio::time::timeout(...)`, because `cluster_ha_states()` uses `spawn_local` polling; canceling the outer future can drop join handles and leave detached local tasks running, which risks flake and runtime noise.
- If “window must be hard-bounded” becomes necessary later, prefer adding an explicit *total* timeout inside each spawned poll task (so tasks complete cleanly) rather than canceling from outside.

Notes:
- Keep caller churn minimal by either:
  - keeping the same signature but using a strict default policy internally, or
  - adding a `*_with_policy` variant and making the old one a wrapper.

### 2) Align “fail-safe observed” helper semantics and scenario log claims
Problem:
- `wait_for_all_failsafe` returns success on “no primary + at least one FailSafe ever observed”, but its timeout message and scenario log claim “all nodes”.

Plan:
- Keep behavior (best-effort) but fix claims/messages so the test output is not lying:
  - Update `wait_for_all_failsafe` timeout message to remove “all nodes observed” wording.
  - Update the matrix scenario log string that currently says “fail-safe observed on all nodes” to explicitly say it is best-effort.
- Refactor for clarity (low ripple: only 2 call sites in this file):
  - Rename `wait_for_all_failsafe` -> `wait_for_no_primary_and_any_failsafe_best_effort`.
  - Optionally rename `wait_for_all_nodes_failsafe` -> `wait_for_all_nodes_non_primary_with_failsafe_observed` (no semantic change; avoids implying “every node is FailSafe”).
- Keep strict behavior in the strict no-quorum tests (do not broaden best-effort semantics into places that claim strictness).

### 3) Remove fencing-sensitive timestamp fallback-to-0 and fail closed if sampling is incomplete
Problem:
- SQL workload worker records `committed_at_unix_ms` with `unix_now()` and currently falls back to `0` on error.
- The fencing cutoff assertion counts commits `timestamp > cutoff_ms`, so `0` silently undercounts post-cutoff commits and can let the test pass without evidence.

Plan:
- Change timestamp capture on successful commit:
  - If `unix_now()` succeeds: push timestamp into `committed_at_unix_ms`.
  - If `unix_now()` fails: increment a new counter (e.g. `commit_timestamp_capture_failures`) and **do not** push a sentinel value.
- Aggregate the counter from worker stats into the overall workload stats.
- Before evaluating “commits after cutoff”, enforce strict preconditions:
  - If `commit_timestamp_capture_failures > 0`: return `Err(...)` (cannot evaluate fencing cutoff safely).
  - If `committed_at_unix_ms.len() != committed_writes` (with safe conversions): return `Err(...)` (incomplete sampling).
- Extract the cutoff evaluation into a small pure helper function (inside `e2e_multi_node.rs` or a test-only module) so it can be unit-tested without real binaries.

Additional (skeptical adjustment: remove other misleading `0` time sentinels in touched code):
- Update `ClusterFixture::record` so that `unix_now()` failure does not render as `[0] ...`:
  - Use an explicit marker string like `[time_error:<...>]` rather than a numeric sentinel.
  - Keep `record(...)` non-fallible (do not widen signature and ripple call sites) unless it becomes necessary for correctness.

Related hygiene (in the same touched area):
- Replace any `Result::unwrap_or(...)` conversions in the workload worker / aggregation paths with explicit `match` + saturating fallback to comply with “no unwrap/expect/panic” policy.

### 4) Add fast, deterministic tests to prevent regression
Plan:
- Add unit tests (no real binaries) for:
  - `assert_no_dual_primary_in_samples`: fails when `sample_count == 0`.
  - `assert_no_dual_primary_window` finalization: fails when `successful_samples == 0` (extract a tiny pure helper for the decision so this stays unit-level).
  - Cutoff evaluation helper:
    - fails when `commit_timestamp_capture_failures > 0`
    - fails when timestamp count mismatches committed writes
    - counts `> cutoff_ms` correctly (including boundary `== cutoff_ms`)
- Optional (if feasible without mocking async polling):
  - unit-test a small “dual-primary evidence evaluator” helper that consumes synthetic poll outcomes and enforces the “min complete polls” rule.

### 5) Validation gates (must all pass)
- Run: `make check`
- Run: `make test`
- Run: `make test-long`
- Run: `make lint`
</plan>

NOW EXECUTE
