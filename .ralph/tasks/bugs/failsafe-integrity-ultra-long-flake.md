## Bug: Ultra-long failsafe integrity scenario flakes during final table verification <status>completed</status> <passes>true</passes>

<description>
`make test-long` failed during `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity` while this task was being verified on March 7, 2026.

Observed failure:
- the first full ultra-long run failed in `tests/ha/support/multi_node.rs` final integrity verification
- `assert_table_key_integrity_strict(...)` could not verify the workload table on any node
- node-1 and node-3 returned `FATAL: could not open file "global/pg_filenode.map": No such file or directory`
- node-2 refused TCP connections
- rerunning the exact test in isolation passed, and rerunning the full `make test-long` gate also passed, so this currently looks like a flaky scenario or verification strategy rather than a deterministic regression

Evidence from the failing run:
- junit: `target/nextest/ultra-long/junit.xml`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_multi_node_failsafe__e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity.log`
- stress artifacts:
  - `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843365646-0-1772843403435.timeline.log`
  - `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843365646-0-1772843403435.summary.json`

Explore and research the codebase first, then fix. Focus on whether the scenario is leaving nodes in a legitimately unreadable state, whether the final verification should wait for a narrower recovery condition, or whether the HA/process path is corrupting the data directory under this no-quorum fencing sequence.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed Implementation Plan (Draft for skeptical review, 2026-03-07)

### Evidence-backed code facts

- `tests/ha/support/multi_node.rs` currently stops etcd majority, samples HA state for 2 seconds, sleeps 8 seconds, stops the workload, and then immediately calls `assert_table_key_integrity_strict(...)` while quorum is still lost.
- `assert_table_key_integrity_strict(...)` succeeds on the first node that can answer two SQL queries without duplicate rows and with `row_count >= min_rows`; in this scenario `min_rows` is only `1`.
- The passing artifact `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843721390-0-1772843748919.summary.json` shows `committed_writes=16`, while the same run’s timeline records `no-quorum fencing key integrity verified on node-2 with row_count=14`. That means the current integrity assertion can pass on a lagging replica and does not prove that all committed keys survived.
- The failing artifact `.ralph/evidence/27-e2e-ha-stress/ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-1772843365646-0-1772843403435.summary.json` shows the final verification racing node state transitions instead of detecting duplicates or missing keys: node-1 and node-3 returned `FATAL: could not open file "global/pg_filenode.map": No such file or directory`, and node-2 still refused TCP connections.
- `tests/ha/support/multi_node.rs` already has resilient helpers for recovery-oriented checks (`wait_for_stable_primary_resilient`, `assert_former_primary_demoted_or_unreachable_after_transition`, `run_sql_on_node_with_retry`), but this scenario is not using them for its final integrity proof.
- There is no in-place etcd restart support today. `ClusterFixture::stop_etcd_majority()` only shuts members down through `EtcdClusterHandle::shutdown_member()`, and `EtcdClusterHandle` does not retain the full member specs or cluster spawn inputs required to respawn stopped members.

### Working diagnosis

The most likely bug is not silent data corruption during the no-quorum window. The stronger evidence points to a flaky and too-weak verification strategy:

1. the scenario checks table integrity before coordination is restored
2. it accepts any reachable node, including a lagging replica
3. it requires only `row_count >= 1`, not the full committed key set
4. when replicas are mid-fence, mid-restart, or reading an incomplete data directory, the helper times out with transient PostgreSQL errors such as `pg_filenode.map` missing or TCP refusal

That explains both observed outcomes:

- the test can falsely pass on a partially up replica with fewer rows than were committed
- the test can also fail transiently when every node is temporarily unreadable during the same no-quorum window

The correct long-term fix is to split the scenario into two phases:

- outage-phase assertions: verify fail-safe/fencing timing and post-cutoff write blocking while quorum is absent
- recovery-phase assertions: restore quorum, wait for a stable recovered primary, and then verify the exact committed key set on a node that should now be fully readable

### Skeptical review changes applied on 2026-03-07

- The outage phase should not keep its own bespoke `stop etcd -> sample -> sleep` control flow. `tests/ha/support/multi_node.rs` already has `stop_etcd_majority_and_wait_failsafe_strict_all_nodes(...)`, and the scenario plan should reuse that helper so the fail-safe transition is the explicit gate before workload cutoff accounting begins.
- Restart metadata should live on `EtcdClusterHandle`, not duplicated in a higher-level fixture wrapper. `src/test_harness/ha_e2e/startup.rs` already assembles the full `EtcdClusterSpec` and hands ownership to the etcd harness; keeping restart state there avoids a second source of truth for member ports, data directories, and cluster token.
- The exact-key verification work should be split into a pure comparison helper plus a thin SQL-fetching wrapper. That gives deterministic unit coverage for key-set mismatch cases without forcing every assertion branch through async node polling.

### Planned execution phases for the `NOW EXECUTE` pass

#### 1. Add minimal etcd restart support to the real-binary harness

- [x] Extend `EtcdClusterHandle` so a previously stopped member can be respawned without rebuilding the whole cluster from scratch.
- [x] Store the necessary spawn metadata directly on `EtcdClusterHandle`:
  - [x] cluster-wide fields equivalent to `EtcdClusterSpec` (`etcd_bin`, `namespace_id`, `startup_timeout`)
  - [x] the full per-member `EtcdClusterMemberSpec` set keyed by member name
  - [x] a stopped-member map so `shutdown_member()` does not discard restartable members after taking them out of the live vector
  - [x] enough information to rebuild the `initial_cluster` string consistently for restarted members
- [x] Add a restart API on `EtcdClusterHandle` that can restart one named member or a list of stopped members after `shutdown_member()`, and expose only the thin forwarding hook needed by the HA fixture.
- [x] Keep error handling explicit: if a requested member was never part of the cluster, was not stopped, or fails readiness checks after restart, return a structured error rather than falling back silently.

Primary files:

- `src/test_harness/etcd3.rs`
- `src/test_harness/ha_e2e/handle.rs`
- `src/test_harness/ha_e2e/startup.rs`

#### 2. Strengthen the table-integrity helper so it can prove exact workload preservation

- [x] Add a pure comparison helper plus a recovery-phase SQL wrapper instead of stretching `assert_table_key_integrity_strict(...)` beyond its intended shape.
- [x] The final helper verifies all of the following on a chosen recovered-primary node:
  - [x] the table is readable
  - [x] there are no duplicate `(worker_id, seq)` keys
  - [x] every key committed at or before the fencing cutoff is present after recovery
  - [x] every recovered key belongs to the client-observed committed key set (no phantom keys)
- [x] Keep retry behavior, but only for transient reachability/readiness failures. Once the node is queryable, a missing required key or unexpected key fails hard immediately.
- [x] Query the concrete key set from SQL and compare it against `BTreeSet` bounds derived from per-worker committed key/timestamp pairs.
- [x] Keep the comparison logic pure so unit tests can cover key-set mismatch cases without needing a live fixture.
- [x] Leave the existing weaker helper in place if other scenarios still need “some readable node” behavior, but stop using it for this bug’s scenario.

Primary files:

- `tests/ha/support/multi_node.rs`
- `src/test_harness/ha_e2e/util.rs` only if a shared row parser/helper materially reduces duplication

#### 3. Rewrite the no-quorum fencing scenario around outage assertions first, recovery assertions second

- [x] In `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`, replace the ad-hoc `sleep(8s)` plus immediate SQL verification with an explicit state machine:
  - [x] wait for stable bootstrap primary
  - [x] prepare table and start workload
  - [x] stop etcd majority through `stop_etcd_majority_and_wait_failsafe_strict_all_nodes(...)` so the outage phase is gated on an explicit all-node fail-safe observation
  - [x] keep any HA sampling as secondary telemetry for the summary, not as the control-flow mechanism for entering verification
  - [x] keep the write-cutoff accounting logic and rejection checks
  - [x] stop the workload
  - [x] restore the stopped etcd members
  - [x] wait for the cluster to regain trusted coordination and a stable primary
  - [x] verify recovery-time committed-key bounds on that recovered primary
- [x] Record additional timeline notes for the recovery phase:
  - [x] when fail-safe was observed and when recovery verification ran
  - [x] which node became the recovered primary
  - [x] how many pre-cutoff required keys and total client-observed committed keys were expected
- [x] Re-evaluate whether the recovered primary must equal the original bootstrap primary. The final behavior accepts either a recovered original primary or a promoted successor, and validates the recovered-primary key set rather than assuming leader identity stability.
- [x] A separate proof write was not needed once the recovered-primary key-set bounds and stable-primary wait were in place.

Primary files:

- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failsafe.rs`

#### 4. Add focused deterministic regressions around the new harness and assertion logic

- [x] Add harness-level tests for restarting stopped etcd members so the recovery path is covered without relying only on the ultra-long scenario.
- [x] Add unit coverage for the pure recovery key-bounds comparison helper:
  - [x] duplicate keys fail
  - [x] missing required keys fail
  - [x] extra unexpected keys fail
  - [x] allowed post-cutoff extra keys can still pass when they stay within the client-observed committed set
- [x] Keep these tests deterministic and local to the helper logic where possible; the long scenario should remain the integration proof, not the only proof.

Primary files:

- `src/test_harness/etcd3.rs`
- `tests/ha/support/multi_node.rs`

#### 5. Update docs where the scenario semantics or HA stress coverage are described

- [x] Update contributor/testing docs so the no-quorum fencing scenario is described as:
  - [x] proving writes are fenced after the cutoff during quorum loss
  - [x] then proving the recovered primary preserves all pre-cutoff committed keys and rejects phantom keys after the cluster recovers
- [x] Remove or adjust any wording that implies the final integrity proof is taken directly from an outage-state node while quorum is still absent.
- [x] Document the stronger post-recovery verification semantics where the real-binary testing system is described.

Likely docs:

- `docs/src/contributors/testing-system.md`
- `docs/src/lifecycle/failsafe-fencing.md` only if the scenario narrative there becomes stale

#### 6. Verification order for the `NOW EXECUTE` pass

- [x] Run the new focused unit/harness regressions first.
- [x] Run the targeted long scenario directly while iterating:
  - [x] `cargo test --test ha_multi_node_failsafe e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity -- --nocapture`
- [x] Once the targeted scenario is stable, run all required gates with no skips:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Update this task file with execution notes and completed acceptance checkboxes only after those gates pass.
- [x] Set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph` state, and `git push` only after the task is actually complete.

### Execution notes

- The first recovery implementation used exact equality against all client-observed committed keys. That passed in isolation but failed in the full ultra-long suite when a promoted successor primary legitimately recovered with fewer rows than the client had observed, which means exact equality was too strong for this scenario.
- The final invariant is stronger than the original flaky check but valid under the actual recovery behavior: after quorum returns, the recovered writable primary must contain every key committed at or before the fencing cutoff, and it must not contain any key outside the client-observed committed set.
- Final gate results in the completed tree:
  - `make check` passed
  - `make test` passed
  - `make test-long` passed
  - `make lint` passed

### Risks the required `TO BE VERIFIED` pass must challenge

- The plan currently assumes the right fix is “recover then verify exact keys.” The skeptical pass must explicitly test whether a narrower non-restart approach could be both simpler and equally strong.
- The plan currently favors adding etcd restart capability to the harness. The skeptical pass must decide whether that belongs in `EtcdClusterHandle` itself or in a higher-level HA e2e fixture wrapper.
- The plan currently proposes exact committed-key matching on the recovered primary. The skeptical pass must confirm whether an exact-key-set query is necessary, or whether exact row-count plus explicit proof-key reads would be sufficient and simpler.
- The skeptical pass must alter at least one concrete part of this plan before switching the marker to `NOW EXECUTE`.

NOW EXECUTE
