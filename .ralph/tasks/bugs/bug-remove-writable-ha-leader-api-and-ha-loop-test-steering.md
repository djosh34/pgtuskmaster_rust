---
## Bug: Remove writable HA leader API control path and enforce HA-loop-only leadership transitions <status>completed</status> <passes>true</passes> <passing>true</passing>

<description>
Investigation found that writable `/ha/leader` was introduced by task `22-task-ha-admin-api-read-write-surface` as part of a "full HA admin API read and write surface". In runtime code, `src/api/worker.rs` routes `POST /ha/leader` and `DELETE /ha/leader` to controller handlers in `src/api/controller.rs` that call `DcsHaWriter::write_leader_lease` / `delete_leader`, so external callers can directly mutate the leader key outside autonomous HA-loop decision flow.

This conflicts with lease/autonomous leadership expectations and enables direct DCS steering through API, including in e2e scenario code.

Research first, then fix end-to-end:
- Remove writable `POST /ha/leader` and `DELETE /ha/leader` runtime routes and associated controller handlers.
- Keep read surfaces (`/ha/state`, debug reads) and switchover request semantics unless separately deprecated by another decision.
- Remove CLI commands/client methods that call writable `/ha/leader` APIs.
- Migrate tests and e2e scenarios that currently depend on `/ha/leader` mutation to HA-loop-driven transitions (failure injection + switchover + convergence observation).
- Update policy guard tests so e2e cannot reintroduce writable `/ha/leader` steering.
- Update published API endpoint contracts/lists so writable `/ha/leader` is no longer advertised.

Directly impacted usages discovered during investigation:
- `tests/bdd_api_http.rs` route/auth mutation checks for `/ha/leader`
- `src/api/worker.rs` tests that exercise `/ha/leader` routing/auth behavior
- `src/api/controller.rs` tests for `post_set_leader` and `delete_leader`
- `src/cli/mod.rs` and `src/cli/client.rs` `ha leader set/clear` command paths/tests
- `src/ha/e2e_multi_node.rs` scenario matrix uses `POST /ha/leader` and `DELETE /ha/leader` to inject leader conflicts and force failover steering
</description>

<acceptance_criteria>
- [x] Writable `/ha/leader` API surface is removed from runtime routing, controller handlers, and published debug API endpoint listing.
- [x] CLI no longer exposes leader set/clear commands that mutate DCS leader key directly.
- [x] HA e2e scenario(s) prove switchover/failover/fencing behavior without `/ha/leader` manual writes; transitions are driven by HA loop and external failure/switchover stimuli only.
- [x] Tests previously validating `/ha/leader` writes/deletes are replaced with tests validating forbidden/absent route behavior and HA-loop outcomes.
- [x] Policy guard coverage fails if e2e code reintroduces `/ha/leader` write/delete steering.
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

## Execution Plan (Draft 2 - Skeptical Verification Applied)

### Parallel exploration tracks completed in this planning pass
- [x] Track 1: API runtime route mapping and auth-role classification in `src/api/worker.rs`.
- [x] Track 2: HA controller write/delete handlers and unit coverage in `src/api/controller.rs`.
- [x] Track 3: CLI command graph and dispatch wiring in `src/cli/args.rs` and `src/cli/mod.rs`.
- [x] Track 4: CLI HTTP client methods and request-shape tests in `src/cli/client.rs`.
- [x] Track 5: HA e2e API steering helpers and scenario matrix dependencies in `src/ha/e2e_multi_node.rs`.
- [x] Track 6: BDD HTTP route/auth contract tests in `tests/bdd_api_http.rs`.
- [x] Track 7: Debug verbose endpoint listing in `src/debug_api/view.rs`.
- [x] Track 8: Existing API-only policy guard behavior in `tests/policy_e2e_api_only.rs`.
- [x] Track 9: Lifecycle examples for plan marker format in `.ralph/tasks/**`.
- [x] Track 10: Cross-reference sweep for all `/ha/leader` usage across `src/`, `tests/`, and `.ralph/tasks/`.

### Planned execution phases (run exactly in this order when promoted to `NOW EXECUTE`)

### 0) Preflight and evidence scaffolding
- [x] Create `.ralph/evidence/bug-remove-writable-ha-leader-api-and-ha-loop-test-steering/` and capture all gate logs there.
- [x] Snapshot baseline with `git status --short` and `rg -n "/ha/leader|LeaderCommand|post_set_leader|delete_leader" src tests -S`.
- [x] Keep all changes ASCII-only and maintain no-`unwrap`/`expect`/`panic` policy.

### 1) Remove writable runtime API surface
- [x] In `src/api/worker.rs`, remove route arms for:
  - [x] `("POST", "/ha/leader")`
  - [x] `("DELETE", "/ha/leader")`
- [x] In `src/api/worker.rs`, remove any role-mapping/auth classification entries that treat `/ha/leader` as an admin route.
- [x] Replace route-level tests that currently assert 202/write-delete behavior for `/ha/leader` with:
  - [x] explicit `404 Not Found` (or absent-route) assertions for POST/DELETE `/ha/leader`,
  - [x] explicit auth-behavior assertions under configured tokens so `/ha/leader` no longer inherits admin-only `403` handling via role mapping,
  - [x] retained coverage for `/ha/state`, `/switchover`, and `/ha/switchover` behavior and authz.

### 2) Remove controller writable leader handlers
- [x] In `src/api/controller.rs`, remove `SetLeaderRequestInput`, `post_set_leader`, and `delete_leader`.
- [x] Remove associated controller unit tests:
  - [x] unknown-field and empty-member-id validation tests for set-leader input,
  - [x] typed-leader write/delete key assertions.
- [x] Keep `post_switchover`, `delete_switchover`, and `get_ha_state` intact with current contracts.

### 3) Remove CLI leader mutation surface
- [x] In `src/cli/args.rs`, remove `ha leader` command tree (`HaCommand::Leader`, `LeaderArgs`, `LeaderCommand`, `SetLeaderArgs`) and parser tests depending on it.
- [x] In `src/cli/mod.rs`, remove dispatch branches that call `client.post_set_leader(...)` and `client.delete_leader()`.
- [x] In `src/cli/client.rs`, remove:
  - [x] `SetLeaderRequestInput`,
  - [x] `post_set_leader(...)`,
  - [x] `delete_leader(...)`,
  - [x] request tests for `/ha/leader` POST/DELETE.
- [x] Preserve CLI support for `ha state`, `ha switchover request`, and `ha switchover clear`.

### 4) Update advertised endpoint contracts
- [x] In `src/debug_api/view.rs`, remove `"/ha/leader"` from `ApiSection.endpoints`.
- [x] Update any affected endpoint-list tests/assertions accordingly.

### 5) Migrate HA e2e scenarios away from `/ha/leader` steering
- [x] In `src/ha/e2e_multi_node.rs`, remove helper methods that call writable `/ha/leader`:
  - [x] `post_set_leader_via_api(...)`,
  - [x] `delete_leader_via_api(...)`.
- [x] Refactor HA e2e scenarios to keep behavioral intent without manual leader writes:
  - [x] planned switchover in `e2e_multi_node_real_ha_scenario_matrix` remains via `POST /switchover`,
  - [x] failover/fencing evidence remains HA-loop-driven in `e2e_multi_node_unassisted_failover_sql_consistency` (no `/ha/leader` steering),
  - [x] split-brain checks rely on observed HA phases and no-dual-primary windows, not injected conflicting leader records.
- [x] Ensure timeline logs still explain each scenario phase with precise stimuli and observed outcomes.

### 6) Replace BDD `/ha/leader` tests with absent-route assertions
- [x] In `tests/bdd_api_http.rs`, replace current route mutation tests (`POST/DELETE /ha/leader`) with:
  - [x] assertions that these paths are unavailable/not routable,
  - [x] retained checks for existing supported routes (`/ha/state`, `/switchover`, `/ha/switchover`) and auth rules.
- [x] Keep BDD tests real and non-optional.

### 7) Strengthen policy guard against reintroduction
- [x] In `tests/policy_e2e_api_only.rs`, extend forbidden patterns to fail if e2e source reintroduces writable `/ha/leader` steering (for example path literals and helper names tied to set/clear leader injection).
- [x] Preserve current guard intent that e2e control should flow through allowed API pathways only.

### 8) Validation sequence (must all pass)
- [x] Run targeted tests first:
  - [x] `cargo test --all-targets api::controller`
  - [x] `cargo test --all-targets api::worker`
  - [x] `cargo test --all-targets debug_api::view`
  - [x] `cargo test --all-targets cli::args`
  - [x] `cargo test --all-targets cli::client`
  - [x] `cargo test --all-targets bdd_api_http`
  - [x] `cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix`
  - [x] `cargo test --all-targets policy_e2e_api_only`
- [x] Run required gates and archive logs:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-bdd`
  - [x] `make lint`
- [x] For `make test` and `make lint`, store grep evidence for `congratulations` and `evaluation failed`.

### 9) Closeout once execution is complete
- [x] Update this task file checklist/status/passing tags only after all gates are green.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all touched files including `.ralph` artifacts using:
  - [x] `task finished bug-remove-writable-ha-leader-api-and-ha-loop-test-steering: ...`
- [x] Append any new durable learning to `AGENTS.md`.

NOW EXECUTE
