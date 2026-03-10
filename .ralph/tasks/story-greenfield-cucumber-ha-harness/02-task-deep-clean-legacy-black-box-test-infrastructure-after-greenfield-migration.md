## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
Task 02 is no longer a narrow overlap cleanup. The old HA/E2E surface is still considered actively harmful and must be removed decisively before the greenfield cucumber story continues. The repo already has an independent greenfield HA harness under `cucumber_tests/ha/`, and the goal of this task is to remove the bad old HA/E2E path without throwing away useful non-HA test coverage.

This task therefore fully replaces the earlier “keep the old harness for now” direction, but it also rejects the overreaction of “delete all tests.” The required cleanup boundary is:
- all E2E coverage for the old HA path must be removed
- all HA integration tests for the old harness must be removed
- all unit tests must stay
- most non-HA integration tests must stay
- helpers must be kept or removed based on whether specific surviving tests still need them

Do not keep harmful legacy coverage around just because tasks 03 and 04 have not yet recreated every scenario. Those later tasks should build the desired greenfield features from their scenario contracts, not from a half-dead legacy harness that keeps interfering with the repo.

Aggressive cleanup still applies, but only to the old HA/E2E path. Do not translate that into random deletion of useful unit tests or useful non-HA integration tests. The right cleanup is precise: delete the bad E2E and HA-integration path, keep the important tests for the rest of the product, and then delete any helper code that became unused because of that cleanup.

The decision boundary must be test-by-test, not file-by-file. For integration tests, inspect individual tests and keep only the ones that are still worth having. For helper modules such as `src/test_harness/pg16.rs`, `src/test_harness/etcd3.rs`, `src/test_harness/runtime_config.rs`, `src/test_harness/tls.rs`, and similar, do not preserve a whole helper file just because some part of it might still be useful in theory. Keep helper code only when a concrete surviving test still calls it. If a file mixes keeper helpers and dead helpers, shrink the file instead of treating it as all-or-nothing.

The required delete set for this task is at least:
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
- `tests/ha/support/observer.rs`
- `tests/policy_e2e_api_only.rs`
- `src/test_harness/ha_e2e/`
- `src/test_harness/net_proxy.rs`
- any direct module exports, helpers, comments, docs, nextest rules, or task text that still preserve or route through that deleted legacy HA/E2E surface

The executor must also review the rest of `tests/` and `src/test_harness/` with the correct boundary:
- keep all unit tests unless they only test deleted E2E code
- keep non-HA integration tests unless a specific test is clearly stale or only exists to support removed HA/E2E routing
- delete HA integration tests and old E2E tests
- if a `src/test_harness/*` helper existed mainly to support the deleted HA/E2E story, delete it
- only keep helper code when there is a direct surviving caller and a strong reason to keep it

The keep set is explicit and must be defended explicitly:
- keep all unit tests under `src/` unless they only test deleted legacy HA/E2E modules
- keep most non-HA integration tests under `tests/`
- delete HA integration tests and any old E2E tests
- helper modules such as `pg16`, `etcd3`, `runtime_config`, `tls`, `ports`, `namespace`, `binaries`, `auth`, `provenance`, and `signals` are not auto-delete and not auto-keep; retain only the helper functions, structs, and tests that specific surviving tests still need
- if any supposedly kept integration test still depends on deleted HA/E2E helpers or assumptions, either refactor it away from that dependency in this task or delete it too; do not keep accidental legacy coupling
- after each cleanup step, remove newly unused code instead of leaving it behind
- if the repo contains any `#[allow(dead_code)]`, `#[allow(unused...)]`, linter-ignore, or similar suppression that exists only because of now-deleted HA/E2E code, remove that suppression and then remove the unused code too

Concrete guidance from the current `tests/**` review:
- delete all tests in `tests/ha_multi_node_failover.rs`
- delete all tests in `tests/ha_partition_isolation.rs`
- delete all tests in `tests/policy_e2e_api_only.rs`
- in `tests/bdd_api_http.rs`, delete the HA-oriented tests around switchover, `/ha/state`, and removed HA leader routes; keep only the non-HA API/auth/debug/fallback contract tests that still matter after the HA integration cleanup
- in `tests/cli_binary.rs`, delete the HA/cluster operator tests around `state`, `status`, `primary`, `replicas`, and `switchover`; keep the generic CLI help/debug/node-config validation tests
- in `tests/bdd_state_watch.rs`, keep the state-channel contract test
- `tests/nextest_config_contract.rs` is not a protected keeper; if it still hardcodes the deleted `ha_*` layout after the cleanup, delete or rewrite it to the new reality rather than preserving stale routing assumptions

Concrete guidance from the current `src/test_harness/**` review:
- delete `src/test_harness/ha_e2e/`
- delete `src/test_harness/net_proxy.rs`
- `src/test_harness/auth.rs` currently has no meaningful non-HA callers outside its own tiny tests; delete it unless the executor finds a concrete surviving caller during execution
- keep the non-HA helper areas that still back important surviving tests, but trim them helper-by-helper:
- `runtime_config` because many surviving unit and non-HA integration tests still construct runtime configs through it
- `tls` because surviving TLS/API tests still use its adversarial fixture and material writers
- `namespace` and `ports` because many surviving real-binary non-HA tests still need isolation and port reservation
- `binaries` and `provenance` because surviving real-binary tests still need verified binary discovery/attestation
- `pg16` because surviving postgres-control and logging tests still need direct PostgreSQL spawn helpers
- `signals` because surviving kept helpers such as `pg16` still use it
- `etcd3` is not an automatic delete: it has non-HA etcd-control tests of its own, so the executor must decide whether those tests are still valuable under the new boundary; if yes, keep only the etcd-control parts, and if not, delete the module entirely

This task must make the deletion boundary fully unambiguous before any more greenfield feature growth. After this task, there should be no repo ambiguity about whether the legacy HA/E2E harness is still a supported path. It is not.
</description>

<acceptance_criteria>
- [ ] Task 02 explicitly supersedes the earlier “retain unreplaced legacy HA coverage” plan and instead treats old HA/E2E removal as mandatory before continuing the greenfield story.
- [ ] `tests/ha_multi_node_failover.rs` is deleted.
- [ ] `tests/ha_partition_isolation.rs` is deleted.
- [ ] `tests/ha/support/multi_node.rs` is deleted.
- [ ] `tests/ha/support/partition.rs` is deleted.
- [ ] `tests/ha/support/observer.rs` is deleted.
- [ ] `tests/policy_e2e_api_only.rs` is deleted.
- [ ] `src/test_harness/ha_e2e/` is deleted.
- [ ] `src/test_harness/net_proxy.rs` is deleted.
- [ ] Any newly dead `src/test_harness` exports or modules caused by those deletions are removed in the same task instead of being left as tombstones.
- [ ] Docs, task text, and gate/config references no longer claim that `tests/ha_*`, `tests/ha/support/*`, `src/test_harness/ha_e2e`, `src/test_harness/net_proxy.rs`, or any other removed legacy helpers are still part of the supported verification story.
- [ ] All unit tests remain unless they only test deleted HA/E2E code.
- [ ] Non-HA integration tests remain unless a specific test is stale or only exists to support deleted HA/E2E routing.
- [ ] HA integration tests and old E2E tests are removed.
- [ ] No kept test target or helper still imports, references, or textually depends on the deleted legacy HA/E2E harness.
- [ ] Surviving integration tests are chosen on a test-by-test basis rather than by preserving whole files out of convenience.
- [ ] Surviving helper code under `src/test_harness/` is chosen on a test-by-test caller basis rather than by preserving whole helper files out of convenience.
- [ ] Surviving helper code under `src/test_harness/` is chosen function-by-function, struct-by-struct, and test-by-test when needed rather than preserving dead code in mixed files.
- [ ] Any `allow(dead_code)`, `allow(unused)`, clippy suppression, or similar unused-code marker that existed to tolerate removed HA/E2E code is deleted, and the newly unused code is deleted too.
- [ ] Tasks 03 and 04 are left clearly positioned as greenfield feature-construction tasks, not as reasons to keep the old harness alive.
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 0: Rewrite the deletion boundary so it is explicit
- [ ] Update this task text and any directly adjacent story wording so the repo no longer says task 02 must preserve the legacy HA/E2E harness.
- [ ] Make it explicit that the old HA/E2E surface is being removed because it is actively harming the repo, not because it has been perfectly feature-for-feature replaced already.
- [ ] Make it explicit that all unit tests stay, most non-HA integration tests stay, and only the HA/E2E part is being aggressively removed.
- [ ] Make it explicit that keep/delete decisions are made test-by-test and helper-by-helper, not by preserving or deleting whole files out of convenience.

### Phase 1: Delete the legacy HA/E2E integration tests in `tests/`
- [ ] Delete `tests/ha_multi_node_failover.rs`.
- [ ] Delete `tests/ha_partition_isolation.rs`.
- [ ] Delete `tests/ha/support/multi_node.rs`.
- [ ] Delete `tests/ha/support/partition.rs`.
- [ ] Delete `tests/ha/support/observer.rs`.
- [ ] Delete `tests/policy_e2e_api_only.rs`.
- [ ] Remove the now-empty `tests/ha/` directory if it becomes empty.
- [ ] Inventory every remaining integration test in `tests/` with the rule “remove HA/E2E, keep non-HA unless specifically unjustified”.
- [ ] Delete any remaining `tests/*.rs` file that has no keeper tests left after the per-test review.
- [ ] If a remaining file mixes keeper tests and dead tests, delete only the dead tests and keep the file only if the surviving tests still justify it.

### Phase 2: Delete the hidden legacy HA/E2E support surface in `src/test_harness/`
- [ ] Delete the entire `src/test_harness/ha_e2e/` directory.
- [ ] Delete `src/test_harness/net_proxy.rs`.
- [ ] Remove `pub mod ha_e2e;`, `pub mod net_proxy;`, and any other newly dead exports or references from `src/test_harness/mod.rs` or elsewhere.
- [ ] Review the remaining `src/test_harness/*` modules helper-by-helper with the rule “keep only what surviving tests still need”.
- [ ] If deletion exposes additional `src/test_harness` modules that were only there for the old HA/E2E path, remove them too in this task; do not preserve dead support code.
- [ ] Only keep helper code under `src/test_harness/*` when there is a direct surviving test caller and a clear reason to keep it.
- [ ] If a helper file mixes keeper helpers and dead helpers, delete the dead helpers instead of preserving the whole file by default.
- [ ] After each helper cleanup, remove newly unused functions, structs, constants, imports, and linter suppressions instead of leaving dead remnants.

### Phase 3: Preserve the important non-HA tests and the exact helpers they need
- [ ] Keep unit tests in `src/**` unless they only test deleted legacy HA/E2E code.
- [ ] Keep most non-HA integration tests in `tests/**`.
- [ ] Treat `tests/bdd_state_watch.rs`, `tests/bdd_api_http.rs`, `tests/cli_binary.rs`, and `tests/nextest_config_contract.rs` as examples of integration-test containers that should be reviewed test-by-test rather than deleted reflexively.
- [ ] Apply the current concrete review of `tests/**`:
- [ ] keep the state-channel test in `tests/bdd_state_watch.rs`
- [ ] keep only non-HA API/auth/debug/fallback tests in `tests/bdd_api_http.rs`
- [ ] keep only generic CLI help/debug/node-config validation tests in `tests/cli_binary.rs`
- [ ] delete all tests in `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and `tests/policy_e2e_api_only.rs`
- [ ] delete or rewrite `tests/nextest_config_contract.rs` if it still encodes the removed `ha_*` split
- [ ] For helpers such as `pg16`, `etcd3`, `runtime_config`, `tls`, `ports`, `namespace`, `binaries`, `auth`, `provenance`, and `signals`, inspect which surviving tests still call them before deciding which parts stay.
- [ ] Apply the current concrete review of `src/test_harness/**`:
- [ ] delete `ha_e2e` and `net_proxy`
- [ ] delete `auth` unless a concrete surviving caller is found during execution
- [ ] keep and trim `runtime_config`, `tls`, `namespace`, `ports`, `binaries`, `provenance`, `pg16`, and `signals` according to the exact surviving tests that still use them
- [ ] make an explicit keep-or-delete decision for `etcd3` based on whether its non-HA etcd-control tests are still wanted under the new boundary
- [ ] Do not keep any test or helper merely because it once provided coverage, but also do not delete good non-HA coverage just because the PO reacted strongly to the bad HA/E2E path.

### Phase 4: Remove stale references and routing
- [ ] Update `.config/nextest.toml` so it no longer routes or documents deleted `tests/ha_*` binaries as the long-test boundary.
- [ ] Update `Makefile` targets or comments that still imply the old HA binaries are the `test-long` payload.
- [ ] Update `docs/src/how-to/run-tests.md` and any directly stale docs so they no longer describe the deleted legacy HA/E2E harness as present.
- [ ] Update any nearby Ralph task text that still depends on the old “keep the legacy harness for later migration” assumption if that wording would now mislead the next executor.

### Phase 5: Verification and closeout
- [ ] Run `rg -n "tests/ha_|tests/ha/support|policy_e2e_api_only|src/test_harness/ha_e2e|src/test_harness/net_proxy" src tests docs .config Makefile .ralph`.
- [ ] Confirm any remaining matches are intentional historical/task references only, or remove/update them if they still describe live code paths.
- [ ] Verify the remaining integration-test surface keeps the non-HA tests while removing the HA/E2E path.
- [ ] Verify unit tests remain intact except for tests that necessarily disappear with deleted legacy modules.
- [ ] For every helper left in `src/test_harness/`, verify there is still at least one surviving concrete test that needs it.
- [ ] Verify there are no stale unused-code suppressions or dead helper remnants left behind by the cleanup.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.



### Own analysis:

• After reading the current tests/** files directly, my recommendation under your current rule is:

Delete all of tests/ha_multi_node_failover.rs, all of tests/ha_partition_isolation.rs, and all of tests/policy_e2e_api_only.rs. Those are exactly the old HA/E2E path or policy guards for that path, so they conflict with your cleanup boundary.

The tests I would keep in tests/** are these.

In tests/bdd_state_watch.rs#L4, keep bdd_state_watch_channel_flow. It is a tiny non-HA contract test for the state channel. It does not depend on the old HA harness, Docker, etcd helpers, or legacy E2E plumbing.

In tests/bdd_api_http.rs#L569, keep bdd_api_get_fallback_cluster_returns_name. It is an API contract test for the fallback surface, not an HA integration test.

In tests/bdd_api_http.rs#L602, keep bdd_api_auth_token_denies_missing_header. It is a generic API auth contract test.

In tests/bdd_api_http.rs#L634, keep bdd_api_debug_routes_expose_ui_and_verbose_contracts. It is a debug API contract test, not old HA behavior coverage.

In tests/cli_binary.rs#L283, keep help_exits_success. It is just CLI entrypoint behavior.

In tests/cli_binary.rs#L314, keep missing_required_subcommand_arg_exits_usage_code. Same reason: generic CLI contract.

In tests/cli_binary.rs#L489, keep debug_verbose_command_renders_human_summary.

In tests/cli_binary.rs#L519, keep debug_verbose_since_emits_query_parameter.

In tests/cli_binary.rs#L550, keep debug_verbose_json_outputs_raw_payload_shape.

In tests/cli_binary.rs#L581, keep debug_verbose_auth_failure_maps_to_exit_4.

Reason for those four: they validate the debug CLI/API surface, which is still valuable non-HA integration coverage.

In tests/cli_binary.rs#L953, keep node_help_exits_success.

In tests/cli_binary.rs#L976, keep node_missing_incomplete_config_reports_parse_error.

In tests/cli_binary.rs#L1010, keep node_missing_secure_field_prints_stable_field_path.

In tests/cli_binary.rs#L1073, keep node_rejects_postgres_role_tls_auth_with_stable_field_path.

In tests/cli_binary.rs#L1130, keep node_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled.

Reason for those five: they are configuration/validation contracts for the node binary, not old HA integration behavior.

The tests I would delete from mixed non-HA files are these.

In tests/bdd_api_http.rs#L363, delete bdd_api_post_switchover_writes_dcs_key. That is directly HA-specific.

In tests/bdd_api_http.rs#L415, delete bdd_api_removed_ha_leader_routes_and_ha_state_contract. Also HA-specific.

In tests/bdd_api_http.rs#L496, delete bdd_api_removed_ha_leader_routes_require_legacy_auth_token. Also HA-specific.

In tests/bdd_api_http.rs#L344, I would probably delete bdd_http_parser_rejects_four_digit_status_code too. It is not HA-specific, but it is testing the local test-file HTTP response parser rather than a product contract. Under cleanup pressure, that is low-value integration coverage.

In [tests/cli_binary.rs], I would delete the cluster/HA operator tests:

- state_command_maps_connection_refused_to_exit_3:330
- bare_pgtm_defaults_to_status_and_renders_human_table:366
- status_json_output_contains_queried_via_identity:399
- status_verbose_fetches_debug_verbose_and_renders_detail_block:424
- status_verbose_marks_debug_disabled_without_failing_status:461
- primary_command_renders_single_dsn_line:600
- primary_command_tls_json_uses_path_backed_fields:641
- replicas_command_renders_one_dsn_per_line:722
- primary_command_rejects_watch_flag:801
- state_command_with_config_only_maps_connection_refused_to_exit_3:816
- status_command_with_unusable_derived_api_target_exits_6:855
- switchover_request_uses_admin_token_from_config:881
- status_command_reports_missing_env_backed_token:926

My current recommendation for src/test_harness/**, under your constraints, is:

Delete outright:

- src/test_harness/ha_e2e/
- src/test_harness/net_proxy.rs

Reason: both are directly part of the old HA/E2E path you want removed.

Likely delete:

- src/test_harness/auth.rs

Reason: I found no meaningful surviving non-HA callers outside its own tiny tests. Right now it looks like dead convenience code once HA/E2E cleanup happens.

Keep, but trim helper-by-helper:

- src/test_harness/runtime_config.rs
- src/test_harness/tls.rs
- src/test_harness/namespace.rs
- src/test_harness/ports.rs
- src/test_harness/binaries.rs
- src/test_harness/provenance.rs
- src/test_harness/pg16.rs
- src/test_harness/signals.rs

Why:

- runtime_config is still used all over surviving unit and non-HA integration tests in API, DCS, runtime, debug, logging, postgres-managed, HA decision/worker unit tests, and tests/bdd_api_http.rs.
- tls is still used by surviving TLS/API tests in src/tls.rs and src/api/worker.rs.
- namespace and ports are still widely used by surviving real-binary non-HA tests.
- binaries and provenance are still needed by surviving real-binary tests to locate and attest binaries.
- pg16 is still clearly needed by non-HA tests for postgres control and logging, especially in src/logging/postgres_ingest.rs and src/pginfo/worker.rs.
- signals is still needed as support for kept helpers like pg16.

The nuanced one:

- src/test_harness/etcd3.rs

This is the only one I do not want to classify mechanically.

Why it is not an automatic delete:

- It does not currently have obvious non-HA external callers after HA/E2E removal.
- But its own tests are not HA scenario tests. They are direct etcd-control tests:
  preparing etcd dirs, formatting cluster config, spawning etcd, restarting a stopped cluster member, and verifying readiness/write/read after restart.
- That matches your stated desire to keep valuable coverage for “etcd control”.

So my recommendation is:

- make an explicit keep-or-delete decision for etcd3
- if you still want direct etcd-control coverage, keep etcd3 and its non-HA control tests
- if you decide that direct etcd process-control testing is not worth the maintenance, delete it entirely

My current lean is to keep etcd3, because it looks like the kind of non-HA infrastructure control coverage you explicitly said you want to preserve.
Verdict: keep etcd3

NOW EXECUTE
