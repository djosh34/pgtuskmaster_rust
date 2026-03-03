---
## Bug: DCS watch refresh errors are tracked but ignored <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`refresh_from_etcd_watch` in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) records `had_errors` (for unknown keys or decode failures) but no caller uses it. In [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), `step_once` only checks for `Err`, so unknown/malformed watch events can be silently ignored while the worker still reports healthy state. Decide on the correct behavior (e.g., mark store unhealthy, emit faulted state, or log/telemetry), and wire `had_errors` into worker health so errors do not pass silently.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Plan (drafted 2026-03-03, deep-verified 2026-03-03)

### Findings from initial parallel research
- The reported bug condition is not present in current code as written:
  - `src/dcs/store.rs` still tracks `RefreshResult.had_errors` for unknown watch keys.
  - `src/dcs/worker.rs::step_once` explicitly sets `store_healthy = false` when `result.had_errors` is true.
- `git blame` shows the `had_errors` health wiring was added recently (`73662f93`), after original DCS worker scaffolding.
- Existing worker test coverage verifies malformed JSON decode marks worker unhealthy, but there is no explicit test for the unknown-key path that drives `had_errors = true`.

### Execution plan
1. Add explicit unknown-key regression coverage at both layers before touching behavior.
- In `src/dcs/store.rs` tests, add a new case that feeds an out-of-contract key path under the correct scope and asserts:
  - `refresh_from_etcd_watch(...)` returns `Ok`.
  - `RefreshResult.had_errors == true`.
  - `RefreshResult.applied` only counts known-key updates.
- In `src/dcs/worker.rs` tests, add a new `step_once` case that injects an unknown-key watch event and asserts:
  - `step_once(...)` still returns `Ok(())`.
  - published worker state is `WorkerStatus::Faulted(...)`.
  - published trust is `DcsTrust::NotTrusted`.

2. Re-run focused unit slices to validate the stale-vs-open bug decision on current code.
- `cargo test dcs::store::tests -- --nocapture`
- `cargo test dcs::worker::tests -- --nocapture`
- If both new tests pass without code changes, classify the original report as stale/fixed-by-prior-commit with concrete evidence.
- If either fails, implement the smallest fix in `src/dcs/worker.rs` or `src/dcs/store.rs` and rerun the same focused slices until green.

3. Keep touched tests panic-free and lint-compatible.
- Do not introduce `unwrap/expect/panic`.
- Where existing touched assertions rely on panic-style flows, convert to `Result`/`matches!` assertions while preserving intent.

4. Run required gates sequentially after test/fix updates.
- `make check`
- `make test`
- `make test-long`
- `make lint`
- If any gate fails, fix root cause and rerun from failing gate, then rerun full sequence for confidence.

5. Task bookkeeping after gates pass.
- Update this task file:
  - tick acceptance checkboxes.
  - set `<status>done</status>`, `<passes>true</passes>`, and `<passing>true</passing>`.
  - add concise execution evidence including names/results of new unknown-key regression tests.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes (including `.ralph/*`) with:
  - `task finished dcs-watch-refresh-errors-ignored: ...`
- Append any cross-task learning to `AGENTS.md` if new.

EXECUTED
</implementation_plan>

<execution_evidence>
- Added `src/dcs/store.rs` regression test `refresh_sets_had_errors_for_unknown_keys_and_applies_known_updates`; confirms unknown-key watch paths are ignored but surfaced via `RefreshResult { had_errors: true, applied: 1 }`.
- Added `src/dcs/worker.rs` regression test `step_once_marks_store_unhealthy_when_watch_key_is_unknown`; confirms unknown-key watch events publish `WorkerStatus::Faulted(...)` and `DcsTrust::NotTrusted`.
- Focused test slices:
  - `cargo test dcs::store::tests -- --nocapture` (pass; includes new unknown-key store regression).
  - `cargo test dcs::worker::tests -- --nocapture` (pass; includes new unknown-key worker regression).
- Required gates (all pass):
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Conclusion: the behavior fix was already present in current code path, and this task closes by adding explicit unknown-key regression coverage so the prior fix cannot silently regress.
</execution_evidence>
