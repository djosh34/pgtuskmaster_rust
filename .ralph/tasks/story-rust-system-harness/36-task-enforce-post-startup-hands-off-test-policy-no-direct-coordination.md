## Task: Enforce post-startup hands-off test policy (no direct coordination) <status>completed</status> <passes>true</passes> <priority>high</priority>

<blocked_by>35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only</blocked_by>

<description>
**Goal:** After cluster/node startup, tests must not perform direct internal coordination or DCS steering; they may only observe/listen plus allowed external actions.

**Scope:**
- Define and enforce a strict post-start policy for tests:
- allowed: observation/listening assertions, SQL writes/reads when test intent requires data mutation, and allowed admin API requests (for example switchover flows),
- forbidden: direct coordination mutations, direct DCS key steering, or internal state forcing once startup completes.
- Refactor any tests that currently violate this policy to use external behavior-driven stimuli instead of internals.
- Add/strengthen policy guard tests/scripts to block regressions.
- Ensure exception rules are explicit, minimal, and documented in test policy comments/docs.

**Context from research:**
- Request requires production-parity behavior once the cluster is running: tests should not "drive" internals that production cannot rely on.
- Existing test suites historically had some internal coordination shortcuts; those must be removed or gated.
- This builds on unified startup so all scenarios share the same initial runtime semantics.

**Expected outcome:**
- Test behavior from startup onward mirrors production constraints: observe externally, stimulate through allowed interfaces, and never directly coordinate internals.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: e2e/integration test files and harness helpers audited, violations removed, and policy guard coverage added/updated
- [x] Policy guard fails if tests reintroduce forbidden post-start direct coordination patterns (direct DCS writes/deletes, internal coordination forcing)
- [x] Allowed exception paths are explicitly codified: SQL data writes where needed, and approved API actions (including switchover requests)
- [x] Existing HA scenario tests continue to validate role/fencing/safety behavior using external-observable flows
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Detailed Implementation Plan (Draft 2, verified)

### Parallel research/audit tracks completed (5-15+ equivalent)
1. Scope discovery for integration/e2e files: enumerated `tests/*.rs` and `src/ha/e2e_*.rs`.
2. Existing policy guard audit: reviewed `tests/policy_e2e_api_only.rs` forbidden-pattern coverage and scope.
3. Runtime startup entrypoint audit: traced `run_node_from_config` usage to identify all node-starting tests.
4. Direct coordination mutation scan: searched for `.write_path(`, `.delete_path(`, `api::controller::`, and direct DCS steering usage in `tests/` + `src/`.
5. HA e2e flow audit: inspected `src/ha/e2e_multi_node.rs` control and observation paths (`/switchover`, `/ha/state`, SQL probes, failure injection).
6. Harness helper audit: reviewed `src/test_harness/*` and etcd/pg helpers for startup vs post-start coordination behavior boundaries.
7. Startup-test boundary audit: reviewed `src/runtime/node.rs` tests and confirmed they are startup-mode selection unit tests, not post-start coordination e2e.
8. False-positive risk audit: compared potential policy scope against `tests/bdd_api_http.rs` and worker contract tests that intentionally use internal stubs.

### Current-state findings from audit
- Node-starting full-cluster tests currently live in `src/ha/e2e_multi_node.rs`; this is the primary enforcement target.
- Existing policy guard is e2e-focused and useful, but its name/scope is still "API-only" and not explicit about broader post-start "hands-off coordination" policy.
- Existing guard does not explicitly forbid some future bypass vectors (for example direct store/etcd wiring tokens) in node-starting e2e files.
- Existing guard currently forbids `post_switchover(`, which is over-broad and can ban legitimate helper naming for an allowed admin API action.
- Existing guard is missing explicit bans for `crate::ha::worker::step_once(` and `crate::dcs::worker::step_once(` in node-starting e2e sources.
- Integration tests such as `tests/bdd_api_http.rs` intentionally use contract stubs and `step_once`; these are controller/worker contract tests and must be excluded from post-start runtime parity policy to avoid false positives.
- HA e2e currently uses external-observable/API-driven control paths for switchover and state polling, with SQL and process/etcd failure injection as external stimuli.

### Policy definition to codify (explicit and minimal)
Post-start (after node/cluster runtime startup completes) tests may:
- Observe/listen only through external surfaces (`GET /ha/state`, API responses, logs/artifacts, SQL reads).
- Stimulate via approved external interfaces:
- SQL writes/reads when scenario intent requires data mutation.
- Admin API requests (for example `POST /switchover`, `DELETE /ha/switchover` if needed).
- External process/network fault injection used by HA scenarios (for example stopping postgres, stopping etcd members) as production-like failure stimuli.

Post-start tests may not:
- Perform direct DCS key writes/deletes (`write_path`, `delete_path`, direct etcd put/delete from scenario control logic).
- Force internal worker coordination via direct worker/controller internals after startup.
- Reintroduce direct internal steering helpers in node-starting e2e sources.

### Planned implementation changes (file-by-file)
1. `tests/policy_e2e_api_only.rs`
- Expand and rename semantics to represent post-start hands-off policy (keep filename or rename with follow-up import-safe migration).
- Tighten forbidden patterns for node-starting e2e scope to include additional direct coordination tokens:
- `.write_path(`
- `.delete_path(`
- `api::controller::`
- `EtcdDcsStore::connect(`
- `refresh_from_etcd_watch(`
- `crate::ha::worker::step_once(`
- `crate::dcs::worker::step_once(`
- `crate::api::worker::step_once(`
- `crate::debug_api::worker::step_once(`
- keep leader-forcing route helpers banned (`"/ha/leader"`, `post_set_leader_via_api`, `delete_leader_via_api`).
- remove over-broad `post_switchover(` forbidden token because switchover is an explicitly allowed admin API path.
- Keep scanner scope restricted to `src/ha/e2e_*.rs` (node-starting e2e surface) to avoid false positives in contract/unit tests.
- Improve failure diagnostics to print offending file + pattern with policy hint.
- add a small assertion in the policy test to document allowed actions (switchover path + SQL helpers) are intentionally not forbidden by this guard.

2. `src/ha/e2e_multi_node.rs`
- Add concise policy comments near control helpers (`send_node_request`, switchover helpers, failure injection helpers) explicitly documenting allowed post-start actions and banned direct coordination.
- Verify no remaining direct internal coordination calls; if any are found during execution, refactor to external API or external stimulus flows.
- Keep existing external-observable assertions intact (API polling + SQL continuity/fencing proofs).

3. Optional policy-note location (if needed for discoverability)
- Add a short "post-start hands-off" note in a stable internal planning/docs location used by engineers (for example task comments or harness policy section) to reduce regressions from new contributors.
- Keep the canonical enforcement in the executable policy test so drift is caught automatically.

### Execution phases
1. Policy guard hardening first.
- Update the guard test patterns and messages.
- Ensure scope is narrow to node-starting e2e files.

2. Hands-off policy annotation and violation cleanup.
- Add/adjust comments in HA e2e helpers.
- Remove/refactor any newly discovered violating pattern in e2e flow.

3. Full validation gates (required order).
- `cargo test --test policy_e2e_api_only -- --nocapture` (fast fail-first policy gate)
- `make check`
- `make test`
- `make test-long`
- `make lint`

4. Task completion bookkeeping (only after all gates pass).
- Tick acceptance checkboxes in this task file.
- Set `<passes>true</passes>`.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes including `.ralph` files with required message format.
- Push branch and append any new learnings to `AGENTS.md`.

### Skeptical risk checks to apply during execution
1. Guard false positives against non-e2e contract tests.
- Mitigation: strict `src/ha/e2e_*.rs` scope; do not scan all `tests/*.rs` blindly for internal tokens.

2. Policy bypass via new helper names not covered by token list.
- Mitigation: include both concrete tokens and category comments; revisit list after first failing run.

3. Over-restricting legitimate external fault-injection scenarios.
- Mitigation: preserve explicit allowed exception comments and keep enforcement targeted to direct internal coordination/DCS steering only.

4. Regression in long HA scenario stability while refactoring.
- Mitigation: no behavior changes beyond policy enforcement/comments unless violating code is found.

NOW EXECUTE
