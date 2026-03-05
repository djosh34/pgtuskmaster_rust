---
## Bug: HA action dedupe suppresses legitimate retries <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the HA functional rewrite story is fully complete. The current investigation already suggests the original report may be partly or fully stale on current `main`, and the remaining useful work may change shape substantially once the facts/decision/effect-plan/worker refactors land.

Do not pull this bug ahead of the rewrite. Reassess it only after `story-ha-functional-rewrite` is complete through its final task, then decide whether to:
- close it as stale because the rewrite removed the old failure mode,
- reduce it to a narrower residual bug in the rewritten HA pipeline, or
- convert the remaining concern into targeted docs/test hardening only.

Current concern recorded here: `HaState.recent_action_ids` may suppress legitimate retries if it effectively persists beyond a single decision tick, causing repeated process-triggering actions such as `StartPostgres`, `StartRewind`, or `RunBootstrap` to be dropped when they should be re-issued.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### What I think is going on (based on current `main`)

The bug description claims `HaState.recent_action_ids` grows forever and suppresses retries across ticks. In the current code, that behavior appears to already be fixed:

- `src/ha/decide.rs` clears `next.recent_action_ids` each `decide(...)` call, so dedupe is tick-local (dedupe within the current candidate list), not cross-tick.
- There are decision-layer regression tests in `src/ha/decide.rs` that explicitly assert:
  - actions are re-issued across ticks while conditions persist
  - prior `recent_action_ids` do not suppress current actions
- Contributor docs (`docs/src/contributors/ha-pipeline.md`) and some design-plan docs still describe the old cross-tick suppression behavior and are now stale.

So the likely “real work” for this task is:

1) verify the runtime behavior + tests match the “already fixed” hypothesis, and then  
2) close the bug properly by updating docs (and optionally strengthening tests at the worker boundary), and  
3) run the full gate suite (`make check/test/test-long/lint`) to meet acceptance criteria.

### Deep skeptical verification notes (2026-03-05)

The preferred `explore_spark` subagents were unavailable due to a quota limit (“try again at 9:12 AM”). I used 20+ `explorer` subagents instead to cross-check the plan.

Key confirmations (skeptical checks):

- `recent_action_ids` is only mutated in one place: it is cleared and rebuilt inside `src/ha/decide.rs` every decide tick, so it cannot grow unbounded across time and cannot suppress retries on future ticks.
- `src/ha/worker.rs` calls `decide(...)` every tick and dispatches `output.actions` without any additional cross-tick dedupe.
- `ActionId` is kind-level (stable) by design, so *if* cross-tick dedupe were reintroduced, it would immediately re-create the “never retry” bug. This makes a worker-level regression test valuable as a guardrail.
- Docs are currently wrong: `docs/src/contributors/ha-pipeline.md` explicitly claims cross-tick suppression, which contradicts implementation + tests.

Plan changes after verification (must-change item):

- **Remove `recent_action_ids` from `HaState` entirely.** Since it is cleared every tick, persisting it in state is redundant and is the direct reason docs drifted into describing cross-tick behavior. Use a local `seen_action_ids` set inside `decide(...)` instead. This permanently eliminates the original class of bug (“unbounded memory / silent cross-tick suppression”) by construction.

### Execution checklist

#### 0) Pre-flight: confirm current behavior (prove/disprove the task statement)

- [ ] Read `src/ha/decide.rs` end-of-function action selection and confirm the presence of:
  - `next.recent_action_ids.clear()`
  - `insert(action.id())` dedupe when building `actions`
- [ ] Confirm `recent_action_ids` is not used for filtering anywhere else (quick grep):
  - [ ] `rg -n "recent_action_ids" -S src docs RUST_SYSTEM_HARNESS_PLAN.md`
- [ ] Run targeted unit tests that should fail if retries are suppressed:
  - [ ] `cargo test --all-targets ha::decide::tests::actions_are_reissued_while_conditions_persist -- --exact`
  - [ ] `cargo test --all-targets ha::decide::tests::previous_recent_action_ids_do_not_suppress_actions -- --exact`
- [ ] Quick lint baseline check (so gate failures aren’t a surprise at the end):
  - [ ] `make lint` (if this fails due to existing clippy warnings, fix them as part of this task; gates must be green)

Decision point:

- If these checks show dedupe is already tick-local and tests pass, treat this as a “stale bug report” and close it by removing the confusing state field + updating docs/tests accordingly.
- If checks reveal cross-tick suppression still exists somewhere (or tests fail), proceed to the “fix path” below.

#### 1) Close-as-fixed / harden path (expected)

##### 1.1) Remove `recent_action_ids` from `HaState` (root-cause cleanup)

Rationale: tick-local dedupe should be a purely local concern of `decide(...)`. Persisting it in `HaState` makes it look like it is intended to affect future ticks, and it invites docs + future refactors to accidentally reintroduce cross-tick suppression.

- [ ] Replace the `HaState` field with a local set:
  - [ ] In `src/ha/state.rs`, delete `recent_action_ids: BTreeSet<ActionId>` from `HaState`.
  - [ ] In `src/ha/decide.rs`, replace the current `next.recent_action_ids.clear()` + `insert(...)` block with:
    - a local `let mut seen_action_ids = BTreeSet::<ActionId>::new();`
    - `if seen_action_ids.insert(action.id()) { actions.push(action); }`
  - [ ] Remove/adjust all initializers that currently set `recent_action_ids: BTreeSet::new()` (compile will guide this).
  - [ ] Update debug views/logging that currently report `recent_action_ids.len()` (replace with something meaningful, e.g. `pending.len()` or omit).
- [ ] Update/replace `ha::decide` tests:
  - [ ] Delete or rewrite `previous_recent_action_ids_do_not_suppress_actions` (its premise disappears once the field is gone).
  - [ ] Add a new unit test that locks in *tick-local dedupe* behavior (duplicate candidates within one decide call collapse to first-wins order).

##### 1.2) Add a worker-level regression test (not optional)

Rationale: decide-level tests prove “reissue is allowed”, but a worker-level test catches regressions where `step_once` fails to dispatch repeated actions across ticks (or someone accidentally reintroduces cross-tick dedupe).

- [ ] Add a new `tokio::test` in `src/ha/worker.rs` (or adjacent HA worker test module) that:
  - [ ] builds a `HaWorkerCtx` with a process inbox receiver that can be asserted on
  - [ ] sets HA state to a phase that emits a process-driving action when conditions persist (e.g. `WaitingPostgresReachable` → `StartPostgres`)
  - [ ] keeps the world snapshot conditions unchanged across two `step_once(&mut ctx).await?` calls
  - [ ] asserts the process inbox receives two `StartPostgres` requests (one per tick)
  - [ ] asserts the produced job IDs differ (tick/index encoded), proving a real retry attempt
- [ ] Ensure the test returns `Result<(), WorkerError>` (or the local error type) and does not use `unwrap`/`expect`/`panic`.

##### 1.3) Update docs to match actual behavior (and remove stale semantics)

Goal: remove stale claims that `recent_action_ids` blocks re-emission on future ticks, and document the current invariant.

- [ ] Update `docs/src/contributors/ha-pipeline.md`:
  - [ ] Remove or rewrite the `recent_action_ids` section entirely (field is removed from code).
  - [ ] Replace it with: *decide performs per-tick duplicate-action filtering (same `ActionId` within one output batch); retries across ticks are allowed/expected when conditions persist*.
  - [ ] Keep guidance that changing this semantic requires updating `decide` tests.
- [ ] Remove legacy design-plan doc that is now actively misleading:
  - [ ] Verify `RUST_SYSTEM_HARNESS_PLAN.md` is not referenced anywhere (`rg -n "RUST_SYSTEM_HARNESS_PLAN" -S .` should be empty).
  - [ ] Delete `RUST_SYSTEM_HARNESS_PLAN.md` (greenfield; no stale/legacy docs).
- [ ] Run docs validation/build (docs changed, so run both):
  - [ ] `make docs-lint`
  - [ ] `make docs-build`

##### 1.4) Close this task file with an explicit resolution summary

- [ ] In this file:
  - [ ] add a `<resolution_summary>` explaining that cross-tick suppression was already not present, and that we removed the misleading field to prevent reintroduction
  - [ ] cite the regression tests that prove retries are not suppressed (decide-level + new worker-level)
  - [ ] set `<status>done</status>` and `<passes>true</passes>` after the full gates pass
  - [ ] set `<passing>true</passing>` only after `make check/test/test-long/lint` all pass

#### 2) Fix path (only if the pre-flight disproves “already fixed”)

If there is still cross-tick suppression / unbounded action-id memory anywhere in the actual execution path:

- [ ] Fix it at the root cause:
  - [ ] remove any cross-tick memory used to suppress actions
  - [ ] keep only tick-local duplicate filtering inside `decide(...)` (local set)
- [ ] Add/extend tests:
  - [ ] decide-level: unchanged conditions across ticks reissue the same action
  - [ ] worker-level: `step_once` dispatches the retry across ticks
- [ ] Update docs as in 1.3.

#### 3) Required verification gates (must be green before marking passing)

- [ ] `make check`
- [ ] `make test`
- [ ] `make test-long`
- [ ] `make lint`
- [ ] `make docs-build` (docs changed in this task)

#### 4) Finish/land (only after all gates pass)

- [ ] Update this task file:
  - [ ] set `<passing>true</passing>`
  - [ ] tick the acceptance criteria checkboxes
- [ ] Run task switcher: `/bin/bash .ralph/task_switch.sh`
- [ ] `git status` sanity check, then stage all (`git add -A`)
- [ ] Commit with message: `task finished ha-action-deduping-suppresses-retry: ...` (include evidence of gates)
- [ ] `git push`

NOW EXECUTE
