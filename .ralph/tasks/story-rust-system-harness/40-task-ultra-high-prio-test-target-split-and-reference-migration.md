## Task: Ultra-high-priority migrate repo gates to `make test` + `make test-long` only <status>done</status> <passes>true</passes> <priority>ultra-high</priority>

<description>
**Goal:** Complete and verify the global migration from legacy test targets to only two test groups: `make test` (regular) and `make test-long` (ultra-long only).

**Scope:**
- Enforce Makefile target surface to only `test` and `test-long` (remove all legacy extra test targets).
- Keep `make test` as the default frequently-run suite and ensure it excludes only tests with evidence-backed runtime >= 3 minutes.
- Keep `make test-long` scoped strictly to tests with evidence-backed runtime >= 3 minutes and print a clear warning that long tests must be moved back to `make test` when they become short.
- Replace legacy references in:
- all `.ralph/tasks/**`
- all `.agents/skills/**`
- the rest of the repository (excluding `.ralph/progress/**` and `.ralph/archive/**`)
- Normalize wording after replacement so no stale/duplicated gate text remains.

**Context from research:**
- Current developer workflow is blocked by very long test execution windows.
- Prior multi-target naming caused gate drift and duplicated wording across tasks/skills.
- A strict two-group model is required to keep fast loops fast while still preserving long-run coverage.
- Current evidence in `.ralph/evidence/27-e2e-ha-stress/*.summary.json` shows:
- `ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql` has passed-run duration range `126357..297266` ms (>= 3 minutes on multiple runs).
- `ha::e2e_multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql` stays around `~21s`.
- `ha::e2e_multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql` stays around `~24-25s`.
- Therefore only no-quorum fencing qualifies for `make test-long` under the 3-minute policy.

**Expected outcome:**
- Repository text and Makefile consistently reference only `make test` and `make test-long`.
- No legacy gate names remain in active tasks/skills/repo documentation, except historical log artifacts if unavoidable.
- Team guidance clearly distinguishes default vs ultra-long execution flow.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Makefile exposes only `test` and `test-long` as test targets (no legacy extra test target definitions).
- [x] `make test` excludes only tests with evidence-backed runtime >= 3 minutes and remains the default frequent-run gate.
- [x] `make test-long` runs only tests with evidence-backed runtime >= 3 minutes and prints a warning to move tests back to regular flow when shortened.
- [x] Under current evidence, `make test-long` includes exactly `ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql`.
- [x] Under current evidence, planned switchover and unassisted failover stress tests run under `make test` (not `make test-long`).
- [x] Legacy target mentions are migrated across tasks, skills, and repo content (excluding `.ralph/progress/**` and `.ralph/archive/**`).
- [x] No duplicate/contradictory gate wording remains after migration.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Execution Plan (Draft)

<execution_plan>

### 0) Lock the two-group contract (definitions + invariants)
- **Definition:** `make test` is the default frequent-run suite and must exclude *only* the explicitly identified ultra-long tests.
- **Definition:** `make test-long` is *ultra-long only* and must run *only* the tests that qualify under the ultra-long policy.
- **Ultra-long policy (evidence aggregation rule):** a test qualifies as ultra-long when the **max passed** duration across the current evidence window is `>= 180_000 ms` (3 minutes). Ignore failed-run `0 ms` summary artifacts.
- **Invariant:** there are no other user-facing “test groups” or “gate” make targets beyond `test` and `test-long`.
- **Invariant:** avoid drifting semantics like “BDD suite”, “full suite”, “real-only suite” that imply additional groups.

Baseline repo sweeps to repeat after edits (excluding progress/archive):
- Find any accidental legacy make-target spellings (precise):
  - Disallowed: `make test-<anything>` other than `make test-long`:
    - `rg -n "make\\s+test-" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'`
  - Disallowed: `make test<suffix>` (for example `make testfoo`):
    - `rg -n "make\\s+test[A-Za-z0-9_]" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'`
  - Disallowed: `make test-long<suffix>` (for example `make test-longer`):
    - `rg -n "make\\s+test-long[A-Za-z0-9_]" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'`
  - `rg -n "\\.PHONY:" Makefile`
  - `rg -n "^\\s*(test|test-long):" Makefile`
- Find duplicated/contradictory gate lists (common drift pattern):
  - `rg -n "CARGO_BUILD_JOBS=1 make test\\b" .ralph/tasks --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'`

### 1) Makefile: keep two targets, reduce drift, enforce the >= 3 minute policy
- Confirm current `Makefile:test` and `Makefile:test-long` behavior matches the ultra-long policy defined in step 0 (before changes it did not).
- Update Makefile so:
  - `make test` excludes **only** the qualifying ultra-long test(s).
  - `make test-long` includes **only** the qualifying ultra-long test(s).
- Reduce drift risk by centralizing ultra-long test IDs in one place in the Makefile:
  - Introduce a variable like `ULTRA_LONG_TESTS := ha::e2e_multi_node::...` and reuse it for both:
    - the `--skip` list in `make test`
    - the explicit `cargo test ... <testname>` invocations in `make test-long`
- Re-check the repo contains no `#[ignore]` tests and remove redundant runtime:
  - `rg -n "#\\[ignore\\]" src tests` should stay empty.
  - Because it is empty today, remove the redundant `cargo test ... -- --include-ignored` pass from `make test` to avoid doubling runtime without coverage gain.
  - If `#[ignore]` tests are introduced in the future, revisit the policy explicitly (do not silently re-add a second pass).

### 2) Repository workflow template: fix gate drift in `.ralph/ralph-do-task.md`
- Replace duplicated “`make test`” in the “really done” checklist with `make test-long`.
- Canonical gate list in the workflow template:
  - `make check`
  - `make test`
  - `make test-long` (intentionally ultra-long)
  - `make lint`

### 3) Skills: update task/bug templates to avoid duplicated or misleading “make test” lines
- Update these skill templates to remove duplicated `make test` bullets and stop implying extra test groups:
  - `.agents/skills/add-bug/SKILL.md`
  - `.agents/skills/add-task-as-user/SKILL.md`
  - `.agents/skills/add-task-as-agent/SKILL.md`
- Canonical acceptance-criteria guidance:
  - keep a single `make test` bullet that describes the suite as “regular/default” and explicitly notes it excludes only the ultra-long test(s) under the >= 3 minute policy.
  - only mention `make test-long` when the task explicitly requires running the ultra-long suite.
  - avoid “BDD full suite” phrasing; if mentioning BDD, clarify it is covered by `make test`.

### 4) `.ralph/tasks/**`: normalize wording across tasks without rewriting intent
- Scope: update wording in all tasks (done + not done), excluding `.ralph/progress/**` and `.ralph/archive/**`.
- Primary normalization targets:
  1. Replace checklist items that treat BDD as a separate suite with wording that does **not** imply a second test group, e.g.:
     - “BDD features pass (covered by `make test`).”
  2. Fix duplicated gate sequences such as two consecutive `make test` entries by converting the drifted “second group” to `make test-long` when the surrounding text/evidence indicates it was intended.
  3. Fix evidence/log lists that repeat `make-test.log` twice by replacing the drifted item with `make-test-long.log` when the task intended `test-long`.
  4. Fix “real-only flow” phrasing to “ultra-long-only” (while keeping “real-binary prerequisites” wording as-is).
- Mechanically verify after edits:
  - `rg -nU --glob '!.ralph/progress/**' --glob '!.ralph/archive/**' -- '- `make test`\\n- `make test`' .ralph/tasks` returns 0 hits.

### 5) Docs: align wording with ultra-long-only semantics
- Update docs that describe test flows:
  - `RUST_SYSTEM_HARNESS_PLAN.md`: replace “Focused real-only flow: `make test-long`” with “Ultra-long-only flow: `make test-long`”.
  - Sweep other docs for “real-only flow” wording and update to “ultra-long-only flow”.

### 6) Final verification (text + behavior)
- Text-level verification (must be clean, excluding progress/archive):
  - `rg -n "make\\s+test-" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'` only reports `make test-long`.
  - `rg -n "make\\s+test[A-Za-z0-9_]" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'` returns 0 hits.
  - `rg -n "make\\s+test-long[A-Za-z0-9_]" --glob '!.ralph/progress/**' --glob '!.ralph/archive/**'` returns 0 hits.
- Behavior-level verification (must be green):
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`

### 7) Closeout protocol
- Tick acceptance criteria items as they are satisfied.
- Set task header to `<status>done</status> <passes>true</passes>`.
- Record concise evidence (commands run + outcomes, and summary of repo sweeps).

</execution_plan>

NOW EXECUTE
