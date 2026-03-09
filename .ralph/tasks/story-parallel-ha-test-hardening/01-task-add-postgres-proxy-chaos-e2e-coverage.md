## Task: Add PostgreSQL Proxy Chaos E2E Coverage <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Add real behavioural coverage for PostgreSQL data-plane faults by exercising the existing Postgres proxy infrastructure in the HA end-to-end harness. The higher-order goal is to stop validating only control-plane failures such as etcd loss and API isolation, and also validate that HA and replication remain correct when the PostgreSQL network path itself is degraded.

The user explicitly wants pg-proxy chaos coverage and believed some form of it already existed. Current research suggests that the harness wires pg proxies but the existing scenarios do not actually use them for a meaningful fault matrix. This task should close that gap directly.

**Scope:**
- Audit the Postgres proxy support in `tests/ha/support/partition.rs`, `tests/ha/support/multi_node.rs`, and the `src/test_harness/net_proxy.rs` plus HA startup wiring.
- Add one or more new e2e scenarios that inject PostgreSQL-path faults through the pg proxy links, not through etcd or API isolation alone.
- Focus on data-plane behaviours that are operationally meaningful and parallel-safe:
- replica losing PostgreSQL upstream connectivity while the rest of the cluster remains healthy
- former primary or candidate replica experiencing PostgreSQL path isolation during failover or rejoin
- healing of the PostgreSQL path and convergence back to one healthy primary with consistent replicated data
- Keep the tests fully isolated in their own ports, namespaces, and artifact paths so they remain safe to run in parallel with the rest of the suite.
- Keep CLI changes out of scope.

**Context from research:**
- `PartitionFixture` already contains `pg_proxies`, and `heal_all_network_faults()` resets them, but the current named scenarios only exercise etcd and API-path faults.
- The current HA behavioural coverage is strong on no-split-brain guarantees during etcd/API partitions, planned switchover, unassisted failover, and quorum-loss fencing.
- There is no current e2e scenario explicitly validating behaviour under PostgreSQL streaming/data-path interruption.
- The user specifically called out pg-proxy chaos as desirable and aligned with current testing goals.

**Expected outcome:**
- The e2e suite gains at least one real data-plane chaos scenario instead of only control-plane chaos.
- HA decisions and eventual convergence are validated when Postgres replication links are degraded independently of etcd and API connectivity.
- The suite better reflects the failures that real clusters actually encounter.

</description>

<acceptance_criteria>
- [x] Review the current pg-proxy wiring and identify the exact existing hooks for blocking or degrading PostgreSQL-path traffic.
- [x] Add at least one new e2e scenario that uses pg-proxy fault injection as the primary fault mechanism rather than etcd-only or API-only isolation.
- [x] The new scenario proves there is no dual-primary window during the injected PostgreSQL-path fault.
- [x] The new scenario proves that writes committed before the fault remain intact after heal/recovery.
- [x] The new scenario proves that post-heal replication converges across all participating nodes.
- [x] If multiple pg-path fault shapes are practical, prefer separate tests for each distinct behaviour rather than one oversized scenario with ambiguous failure cause.
- [x] Reuse the current HA observer/sampling helpers so split-brain assertions remain as strict as the existing suite.
- [x] Ensure the new tests are parallel-safe with isolated namespaces, ports, proxy links, tables, and artifact names.
- [x] Add or update helper functions only where the current partition or multi-node support code is missing a clean pg-proxy injection surface.
- [x] Document in code comments and timeline artifacts exactly which network path is being blocked and why the scenario is meaningful.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If the new scenario belongs in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>

## Execution Plan

### Planning notes locked in before execution

- Current research shows a subtle but important gap: the HA E2E harness already creates one PostgreSQL proxy listener per node in [src/test_harness/ha_e2e/startup.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/startup.rs), and [tests/ha/support/partition.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/partition.rs) already stores those links in `pg_proxies`, but inter-node replication does not traverse those proxies yet.
- The reason is concrete:
  - the runtime publishes `cfg.postgres.listen_port` into DCS in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs),
  - the DCS worker writes that value as `MemberRecord.postgres_port` in [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs),
  - source connections for basebackup/rewind/follow-leader use `member.postgres_port` in [src/ha/source_conn.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/source_conn.rs).
- Because the harness currently sets `listen_port` to the raw PostgreSQL port and only `NodeHandle.sql_port` points at the proxy, blocking a pg proxy today mainly breaks harness SQL clients, not replica-to-primary replication traffic. Execution must fix that first or the new test would be a false positive against the wrong network edge.
- The minimum valuable scenario is therefore not "add a test only"; it is "make the partition harness advertise the pg proxy as the remote PostgreSQL endpoint, then prove a PostgreSQL-path outage interrupts replication without creating a dual-primary window and heals cleanly."

### Phase 1: Put the pg proxy on the actual HA remote PostgreSQL path

- [x] Add an explicit advertised PostgreSQL endpoint override for runtime/DCS publication instead of overloading `postgres.listen_port`.
- [x] Make that override concrete and testable at the runtime-config seam, most likely as `postgres.advertise_port: Option<u16>`, because the DCS worker input is built from `RuntimeConfig` in normal runtime code and the change needs to be visible in parser/schema/builders/tests rather than hidden as a harness-only side channel.
- [x] Keep the local PostgreSQL process bound to the raw `listen_port`; the new override exists only so other nodes can discover and connect through the proxy in HA E2E partition mode.
- [x] Wire that override through the smallest coherent surface:
  - [x] identify the runtime config field or worker input that currently feeds `local_postgres_port` into the DCS worker,
  - [x] change that path so it uses `advertise_port.unwrap_or(listen_port)` in normal runtime usage,
  - [x] allow the HA E2E partition harness to supply the pg proxy port as the advertised PostgreSQL port.
- [x] Update [src/test_harness/ha_e2e/startup.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/startup.rs) so `Mode::PartitionProxy` publishes each node's pg proxy listener port to peer nodes while still starting PostgreSQL on the raw reserved `pg_port`.
- [x] Update the config parser/schema/sample builders that construct `PostgresConfig` so the new field is accepted everywhere the runtime config is instantiated in tests and helpers.
- [x] Add or update focused unit coverage around the publication/source-connection path so it is explicit that remote source connections use the advertised port and the default runtime still uses the raw listen port when no override is set.
- [x] Re-check any harness assertions that inspect generated config or node handles so they continue to distinguish:
  - [x] raw PostgreSQL bind/listen port,
  - [x] proxied SQL client port used by the harness,
  - [x] advertised PostgreSQL port published to peers for replication traffic in partition mode.

### Phase 2: Add dedicated PostgreSQL-proxy fault helpers to the partition fixture

- [x] Extend [tests/ha/support/partition.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/partition.rs) with explicit PostgreSQL-path helpers instead of reaching into `pg_proxies` inline from the scenario body.
- [x] Add a helper along the lines of `set_pg_mode_for_node(...)` / `isolate_postgres_path(...)` that:
  - [x] looks up the pg proxy by node id,
  - [x] sets `ProxyMode::Blocked`,
  - [x] emits a timeline message naming the blocked node and clearly stating that this is the advertised PostgreSQL data path, not etcd or API transport.
- [x] Keep `heal_all_network_faults()` as the single reset path, but ensure the new timeline entries make it obvious when only PostgreSQL traffic was blocked.
- [x] Add a direct/raw SQL helper that talks to `NodeHandle.pg_port` rather than `NodeHandle.sql_port`.
- [x] Use that direct helper only where the scenario needs to prove that the primary stayed locally writable while its advertised PostgreSQL path was severed. This avoids conflating "proxy is blocked" with "postgres is down."
- [x] If needed, add one negative-observation helper for replicas, for example a bounded poll that proves a row written on the isolated primary does not appear on replicas during the fault window. Prefer a helper with strong error messages over ad hoc sleep-plus-query logic.

### Phase 3: Add one focused replication-path chaos scenario before considering any extra variants

- [x] Implement one new partition scenario in [tests/ha/support/partition.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/partition.rs) and register it in [tests/ha_partition_isolation.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_partition_isolation.rs).
- [x] Use a narrow scenario name that says exactly what is happening, for example "primary postgres path blocked, replicas catch up after heal" rather than a broad generic "pg chaos" label.
- [x] Structure the scenario around the existing resilient primary-election and observer helpers rather than inventing a second orchestration style.
- [x] Planned scenario flow:
  - [x] start a 3-node partition fixture,
  - [x] wait for an initial stable primary using the existing resilient helper,
  - [x] create a dedicated proof table with a task-specific name and insert a pre-fault row on the primary,
  - [x] wait until every node has the pre-fault row so the scenario starts from a clean fully replicated baseline,
  - [x] block the primary node's pg proxy, with timeline text that explains this now severs replica-to-primary PostgreSQL traffic because peers discover the primary through the advertised proxy port,
  - [x] explicitly wait for replica-side evidence that the fault took effect on active replication sockets before writing new data, preferably by polling a replica-local observation such as `pg_stat_wal_receiver` disconnect/stall state rather than relying on a blind sleep,
  - [x] assert no dual-primary window during the fault using the existing HA observer sampling,
  - [x] write a second row directly to the primary's raw PostgreSQL port so the primary's local database can progress even though the advertised network path is blocked,
  - [x] prove that replicas do not receive that second row during the fault window,
  - [x] heal the network fault,
  - [x] wait for a stable post-heal primary,
  - [x] verify that both rows become visible on every participating node,
  - [x] verify digest convergence across all nodes,
  - [x] run one final no-dual-primary assertion after heal to prove the cluster converged back to a single primary.
- [x] Make the scenario's timeline artifact explicit enough that a future engineer can reconstruct:
  - [x] when the PostgreSQL path was blocked,
  - [x] which writes were made before and during the fault,
  - [x] what evidence showed replication was interrupted,
  - [x] when replication convergence was observed after heal.

### Phase 4: Keep the scenario parallel-safe and intentionally scoped

- [x] Reuse the fixture's isolated namespace, ports, and artifact directory patterns so the new scenario stays safe inside the `ha_partition_isolation` binary under parallel nextest scheduling.
- [x] Use a unique table name and scenario artifact prefix dedicated to the PostgreSQL-path case.
- [x] Avoid adding a second failover/rejoin scenario in the same execution pass unless the first scenario lands cleanly and still leaves clear time for full verification. Acceptance requires at least one meaningful pg-path test; forcing a second under time pressure is more likely to create an ambiguous or flaky matrix.
- [x] If execution discovers that the current single-proxy-per-node model is too coarse for a later failover/rejoin variant, record that explicitly in the task notes instead of stretching this task into pairwise directional proxying without tests to support it.

### Phase 5: Docs and verification

- [x] Update the user-facing or developer-facing docs that describe the HA partition suite and long test gates.
- [x] Prefer touching the concrete docs pages that already discuss partition handling and long HA tests:
  - [x] [docs/src/how-to/handle-network-partition.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/handle-network-partition.md) for the expanded test coverage summary,
  - [x] [docs/src/how-to/run-tests.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/run-tests.md) if the description of what `make test-long` exercises needs to mention PostgreSQL-path chaos explicitly.
- [x] There is no `update-docs` skill available in the current session, so execution should update the relevant docs directly in-repo and then run the existing docs/lint gates through `make lint`.
- [x] Treat the new scenario as a `make test-long` case by keeping it in the `tests/ha_partition_isolation.rs` binary; the nextest layout already routes `tests/ha_*.rs` binaries into the `ultra-long` profile, so no gate config change is needed unless the test is misplaced.
- [x] Run the full required gates in this exact sequence once implementation is complete:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Only after all gates pass:
  - [x] mark every completed checkbox in this task file,
  - [x] set `<passes>true</passes>`,
  - [x] run `/bin/bash .ralph/task_switch.sh`,
  - [ ] commit all changes including `.ralph` files with the required `task finished ...` commit message containing test evidence and implementation notes,
  - [ ] push with `git push`.
