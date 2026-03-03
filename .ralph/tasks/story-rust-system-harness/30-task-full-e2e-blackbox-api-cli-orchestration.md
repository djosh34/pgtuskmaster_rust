---
## Task: Migrate full e2e suites to black-box API and CLI orchestration <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>23-task-ha-admin-cli-over-api</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Convert full-system e2e tests into black-box tests that interact through public API/CLI surfaces rather than internal worker channels or binary-specific control paths.

**Scope:**
- Refactor full e2e suites to drive administrative actions through API endpoints or `pgtuskmasterctl` only.
- Replace internal-state peeking where possible with API-visible status checks and SQL-level behavior assertions.
- Define allowed test-only failure injection hooks clearly (for example process kill) while keeping control/verification surfaces public.
- Add documentation for e2e black-box policy and update test conventions.

**Context from research:**
- Existing e2e implementation is tightly coupled to internal fixtures/subscribers and direct store operations.
- Requested direction is external operator parity: control via admin API, observe via admin/read API plus SQL.
- This task aligns e2e signal with production usage and reduces hidden coupling.

**Expected outcome:**
- Full e2e flows are operator-realistic black-box tests that exercise system behavior through public control/read interfaces.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: full e2e scenario files migrated to API/CLI control paths, direct internal control helpers removed from full e2e codepaths, API status assertions replacing internal-only checks where feasible, e2e policy doc/conventions updated under repository docs/task notes
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, Skeptically Verified)

1. Parallel verification tracks completed (16 tracks)
- Track 1: Re-scan `src/ha/e2e_multi_node.rs` for all raw HTTP and control callsites.
- Track 2: Re-scan `src/ha/e2e_partition_chaos.rs` for all `/ha/state` callsites and reqwest usage.
- Track 3: Validate CLI API client capabilities in `src/cli/client.rs` (`get_ha_state`, `post_switchover`, `delete_switchover`).
- Track 4: Validate CLI command dispatch surface in `src/cli/mod.rs` and command DTOs.
- Track 5: Validate CLI args schema in `src/cli/args.rs` for switchover request shape.
- Track 6: Re-check policy guard precision in `tests/policy_e2e_api_only.rs`.
- Track 7: Re-check startup readiness/bootstrap helper implementations in `e2e_multi_node`.
- Track 8: Re-check startup readiness/bootstrap helper implementations in `e2e_partition_chaos`.
- Track 9: Confirm stress scenario codepaths still rely on `/ha/state` observation and must preserve diagnostics.
- Track 10: Confirm API isolation scenario intentionally expects `/ha/state` failure while proxy is blocked.
- Track 11: Re-check no-unwrap/no-panic constraints for touched files.
- Track 12: Re-check required gate commands and flake mitigations from AGENTS/Makefile.
- Track 13: Confirm binary preconditions and real-binary contract for full e2e.
- Track 14: Verify task state-machine marker currently `TO BE VERIFIED`.
- Track 15: Validate repository is dirty from prior artifacts; avoid touching unrelated files.
- Track 16: Validate docs target for convention updates (`RUST_SYSTEM_HARNESS_PLAN.md`).

2. Key skeptical changes versus Draft 1 (mandatory alterations)
- Change A (required): Migrate **all** `send_http_request`-based `/ha/state` paths in `e2e_multi_node` including startup readiness/bootstrap helpers, then delete `ApiHttpResponse`, `send_http_request`, `parse_http_response`, and `expect_accepted_response` entirely. This avoids policy ambiguity and keeps one HTTP access style.
- Change B (required): Drive switchover requests through the CLI command path by constructing/parsing `pgtuskmasterctl` command args (via `Cli::try_parse_from` + `crate::cli::run`) instead of direct helper-only HTTP payload assembly. This ensures orchestration tests exercise CLI semantics, not just shared client methods.
- Change C (required): Keep `reqwest` only where needed for transport-layer isolation readiness behavior in `e2e_partition_chaos` startup probes, but move steady-state `/ha/state` observation path to `CliApiClient`; do not add broad `reqwest` policy bans.

3. Scope and migration objective
- Preserve scenario behavior and assertions (HA/fencing correctness, SQL invariants, split-brain checks).
- Use public surfaces only:
- Control actions: CLI command path (`pgtuskmasterctl ha switchover request ...`) executed in-process.
- Observation actions: public read API surface through `CliApiClient::get_ha_state` and SQL behavior checks.
- Preserve allowed external failure injection (process stop, network proxy faults, etcd connectivity faults).

4. Planned code changes (file-by-file)
- `src/ha/e2e_multi_node.rs`
- Add helper to build CLI base URL for a node and helper to run switchover through parsed CLI args (`Cli::try_parse_from` -> `crate::cli::run`).
- Replace `post_switchover_via_api` implementation with CLI command invocation and JSON output validation (`accepted=true`).
- Replace `/ha/state` fetching in cluster polling with `CliApiClient::get_ha_state` and preserve node-id contextual errors.
- Migrate startup helpers `wait_for_node_api_ready_or_task_exit` and `wait_for_bootstrap_primary` to use `CliApiClient` (or reqwest-based typed decoding without raw socket builder) so `send_http_request` stack is removable.
- Delete dead raw HTTP helpers and associated manual parser/types once all callsites are migrated.

- `src/ha/e2e_partition_chaos.rs`
- Replace `fetch_node_ha_state` with `CliApiClient::get_ha_state` against proxy-facing base URL.
- Preserve error context with node id and failure details for expected isolation failures.
- Keep fault-injection and SQL assertions unchanged.
- Keep startup readiness/bootstrap helpers functionally equivalent; avoid regressions in early boot timing.

- `tests/policy_e2e_api_only.rs`
- Add precise forbidden tokens for removed ad-hoc multi-node HTTP orchestration helpers (e.g. `send_http_request(`, `parse_http_response(`, `post_switchover_via_api(`) once removed.
- Add explicit allowed tokens proving public admin/read surfaces remain allowed (`CliApiClient::get_ha_state(`, `crate::cli::run(`, `"/switchover"`, SQL helper tokens).
- Avoid over-broad bans (`reqwest`) because partition startup/isolation transport logic may legitimately use it.

- `RUST_SYSTEM_HARNESS_PLAN.md`
- Add convention note: full e2e orchestration should prefer CLI command path for admin actions and public API/SQL for observation/assertion; no internal worker/controller steering after startup.

5. Planned sequencing for execution run
- Phase A: Implement multi-node CLI switchover path (`Cli::try_parse_from` + `crate::cli::run`) and API-client state polling.
- Phase B: Migrate multi-node startup readiness/bootstrap helpers off raw socket HTTP and remove dead helper stack.
- Phase C: Migrate partition chaos `/ha/state` observation to `CliApiClient` and keep isolation semantics intact.
- Phase D: Update policy guard tokens and documentation.
- Phase E: Compile/test fixups, then targeted tests, then full required gates.

6. Verification protocol
- Targeted first:
- `cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix -- --nocapture`
- `cargo test --all-targets ha::e2e_partition_chaos::e2e_partition_api_path_isolation_preserves_primary -- --nocapture`
- `cargo test --all-targets policy_e2e_api_only -- --nocapture`

- Required gates (all must pass):
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make check`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 make lint`

- Supplemental marker evidence:
- For `make test` and `make lint` logs, grep for `congratulations` and `evaluation failed` and record results; command exit status remains source of truth.

7. Risks and mitigations
- Risk: CLI parse+run path introduces output parsing fragility.
- Mitigation: request JSON output and parse typed `AcceptedResponse`; include parsed error context.
- Risk: Client migration reduces low-level diagnostics.
- Mitigation: preserve detailed node/scenario context in all propagated errors and timeline records.
- Risk: Policy guard false positives.
- Mitigation: ban only precise helper tokens and assert explicit allowed tokens stay unblocked.
</execution_plan>

COMPLETED
