## Task: Add Primary Storage Stall And WAL Full Failover E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add realistic HA coverage for the case where the primary does not disappear cleanly, but becomes unusable because storage is full or WAL cannot advance. The higher-order goal is to prove the cluster can replace a wedged primary whose process may still exist but whose PostgreSQL instance can no longer make forward progress safely.

**Scope:**
- Extend the HA E2E harness and/or process-fault tooling in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failover.rs`
- `src/test_harness/ha_e2e/`
- related process/fault-injection helpers if needed
- Add a fault shape that models storage exhaustion or WAL/archive saturation closely enough to force PostgreSQL into an unavailable or non-progressing state without pretending the whole node vanished.
- Prefer a deterministic mechanism the harness can control, such as:
- a filesystem/full-disk simulation on the primary data or WAL path inside the test namespace,
- or a targeted wrapper/fault injection around the write path that produces the same operator-visible effect: primary can no longer continue normal writes and must be replaced.
- The scenario must show:
- the old primary stops being a usable primary,
- a surviving quorum elects exactly one new primary before heal,
- the old primary does not remain or return as active primary,
- post-failover writes succeed on the new primary.

**Context from research:**
- Current failover tests model “primary Postgres stopped” but not “primary still exists yet is wedged by storage/WAL pressure”.
- Real incidents often look like disk full, WAL cannot be extended, or primary becomes read-only/broken without a clean shutdown.
- This fault can expose different bugs than simple process death because the old leader may linger longer and may still interact badly with DCS/trust.

**Expected outcome:**
- The suite covers a realistic “primary stalled by storage/WAL failure” outage, not only process death.
- HA replacement is validated for a degraded primary that cannot continue service but is not yet a fully dead node.
- The new test strengthens confidence that operator-visible primary breakage still results in one healthy promoted successor.

</description>

<acceptance_criteria>
- [ ] Add one deterministic fault-injection mechanism that simulates storage/WAL exhaustion or equivalent write-path failure on the current primary.
- [ ] Add at least one new scenario in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs) covering this primary-stall failure shape.
- [ ] The scenario proves the old primary stops being a usable primary and the surviving quorum elects exactly one new primary before any heal of the broken node.
- [ ] The scenario includes a bounded pre-heal availability assertion: after the primary stalls, a surviving node becomes stable primary and accepts a proof write while the old primary remains broken.
- [ ] The scenario asserts no dual-primary window and verifies post-failover SQL writes on the new primary.
- [ ] The scenario verifies the broken old primary does not silently remain an eligible leader while stalled.
- [ ] The scenario records evidence of the intended fault, such as WAL/full-disk style write failure, startup/readiness degradation, or another operator-visible marker proving the primary was stalled rather than cleanly shut down.
- [ ] Timeline artifacts clearly identify when the storage/WAL fault took effect and when the replacement primary became stable.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
