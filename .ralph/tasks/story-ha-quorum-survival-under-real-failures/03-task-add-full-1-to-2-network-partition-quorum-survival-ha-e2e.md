## Task: Add Full 1 To 2 Network Partition Quorum Survival HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add realistic full-network-partition coverage where all control-plane and data-plane traffic is split 1:2, and the majority side elects or preserves exactly one primary before any heal as long as DCS quorum still exists. The higher-order goal is to validate the strongest quorum claim the user cares about: majority partition stays available, minority partition cannot remain or become primary.

**Execution contract for this task:** The HA partition E2E coverage added here must remain safe under parallel execution. Do not serialize the partition suite to avoid harness conflicts. Any binary-build or artifact-sharing problem must be solved through nextest-friendly prebuild/reuse of the `pgtuskmaster` binary plus per-test namespace isolation, not by forcing single-test execution.

**Scope:**
- Extend the partition-proxy harness and partition scenarios in:
- `tests/ha/support/partition.rs`
- `tests/ha_partition_isolation.rs`
- `src/test_harness/ha_e2e/startup.rs`
- relevant proxy wiring under `src/test_harness/net_proxy.rs` and related harness files if needed
- Add fixture support for a true full partition split, not a one-path fault:
- etcd links partitioned 1:2,
- API links partitioned 1:2,
- PostgreSQL replication/data-path links partitioned 1:2,
- with scenario control over whether the isolated minority node started as the primary or as a replica.
- Add at least two partition scenarios:
- minority side initially contains the old primary,
- minority side initially contains a replica while the majority contains the old primary.
- In both cases require that, before any heal:
- the minority side is not primary,
- the majority side has exactly one primary,
- if leadership must change, that change happens inside the majority side without waiting for the minority to return.

**Context from research:**
- Current partition tests cover etcd isolation, API-only isolation, PostgreSQL-path isolation, and mixed faults, but not a true full 1:2 cluster split across all paths.
- Current tests also explicitly accept `FailSafe` for an etcd-isolated primary, but they do not prove the majority side remains available with one primary under a full partition.
- The user wants quorum-preserving availability to be the rule whenever the majority side still has DCS quorum.
- The user also wants HA E2E to keep running in parallel; this task must not rely on one-at-a-time execution to stay stable.

**Expected outcome:**
- The partition suite proves the majority side of a 1:2 split keeps or regains one primary before heal.
- The minority side never remains an active primary.
- The suite closes the current gap between “partition forces fail-safe somewhere” and “majority partition still behaves as HA”.

</description>

<acceptance_criteria>
- [ ] Add partition-fixture helpers that can impose a true 1:2 split across etcd, API, and PostgreSQL-path traffic together, not only one path at a time.
- [ ] The fixture helper must let the scenario choose exactly which node is isolated as the 1-side minority, so tests can force “minority had the old primary” and “minority had a replica” deterministically.
- [ ] Add one scenario where the minority partition contains the old primary and assert that:
- [ ] the minority side is not primary before heal,
- [ ] the majority side elects exactly one primary before heal,
- [ ] no dual-primary window is observed.
- [ ] Add one scenario where the minority partition contains a replica and assert that:
- [ ] the majority side preserves or converges to exactly one primary before heal,
- [ ] the minority side does not self-promote,
- [ ] no dual-primary window is observed.
- [ ] Each scenario verifies the pre-heal majority-side outcome explicitly, not only post-heal convergence.
- [ ] Each scenario must include a bounded pre-heal success criterion: majority side has one stable primary and accepts a proof write before the partition is healed.
- [ ] Each scenario must include a bounded pre-heal safety criterion on the minority side: zero primary observations and no successful proof writes as primary.
- [ ] Each scenario verifies post-heal SQL/data convergence once the partition is removed.
- [ ] The scenario design remains readable: one failure story per scenario, with timeline artifacts naming which node was isolated and whether it was primary or replica at split time.
- [ ] The added partition scenarios remain compatible with parallel `nextest`-style execution; this task must not introduce suite-wide serialization or build-lock workarounds.
- [ ] Verification for this task includes a nextest-friendly path that reuses a single built `pgtuskmaster` binary across parallel HA E2E runs.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
