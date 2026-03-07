# Harness Internals

The harness under `src/test_harness/` is the code that turns HA scenarios from "a pile of subprocess helpers" into repeatable evidence. Its job is not just to launch etcd and Postgres. Its real job is to create an isolated topology, wire the node runtimes, inject failures, and return enough handles that the scenario code can prove safety properties instead of merely hoping they held.

If you are new to this area, the best reading path is:

1. `src/test_harness/ha_e2e/startup.rs`
2. `src/test_harness/ha_e2e/handle.rs`
3. `src/test_harness/ha_e2e/ops.rs`
4. `tests/ha/support/multi_node.rs` or `tests/ha/support/partition.rs`

That sequence shows the real control path: cluster creation, returned handles, teardown, then scenario-specific assertions.

## The main harness call path

The contributor entrypoint for HA scenarios is `ha_e2e::start_cluster(config)` in `src/test_harness/ha_e2e/startup.rs`.

In the current implementation, `start_cluster(...)` does all of the following before a scenario begins:

- validates `TestConfig`
- creates a unique `NamespaceGuard`
- salts the DCS scope and cluster name with the namespace id to avoid cross-test collisions
- verifies required real binaries through `require_pg16_process_binaries_for_real_tests()` and `require_etcd_bin_for_real_tests()`
- allocates a full HA topology port reservation with `allocate_ha_topology_ports(...)`
- starts the etcd cluster
- prepares per-node runtime config and data directories
- spawns the node runtimes and any proxy links needed for the scenario mode
- returns a `TestClusterHandle` with node metadata, process handles, timeouts, and proxy controls

That is the quickest place to start if you need to understand why an HA scenario has the topology it does.

## What `TestClusterHandle` gives scenario code

`src/test_harness/ha_e2e/handle.rs` defines the objects that scenario modules actually use.

`NodeHandle` exposes the facts a scenario typically needs:

- node identity
- SQL and Postgres ports
- API addresses, including `api_observe_addr` for observation loops
- the node data directory

`TestClusterHandle` owns the wider cluster lifecycle:

- the namespace guard
- binary paths and timeout policy
- the etcd cluster handle
- all node runtime tasks
- proxy maps for etcd, API, and Postgres links

That split is intentional. Scenario code should drive behavior through the returned handles rather than reaching back into startup internals.

## Cleanup is a first-class contract

Both startup failure cleanup and normal teardown are explicit. `StartupGuard::cleanup_best_effort(...)` in `startup.rs` and `TestClusterHandle::shutdown(...)` in `ops.rs` abort runtime tasks, stop Postgres with `pg_ctl`, shut down proxy links, and then tear down etcd.

The design contract is that cleanup failures are surfaced as aggregated errors, not swallowed. That matters because leftover processes or ports can poison later tests and create false flakes.

If you add a new long-lived resource to the harness, add it to both startup-failure cleanup and steady-state shutdown. Do not rely on process exit or `Drop` side effects alone.

## Namespaces: isolation on disk and in identifiers

`src/test_harness/namespace.rs` is the root of test isolation. `NamespaceGuard::new(test_name)` creates a unique temp-root namespace with dedicated `logs/` and `run/` subdirectories.

Two details matter for contributors:

- the namespace id is used both for filesystem isolation and for salting cluster scope/name in HA startup
- the namespace path is the first place to inspect when an e2e scenario fails

If your new scenario needs artifacts, put them under the namespace instead of writing to ad hoc temp paths.

## Ports: reserve before you bind the world

`src/test_harness/ports.rs` does more than "pick a free port." It returns a `PortReservation` that keeps listeners open so other concurrent tests cannot steal the chosen ports, and on Unix it also records leases in `/tmp/pgtuskmaster_rust_port_leases.json`.

For HA scenarios, `allocate_ha_topology_ports(...)` reserves the full topology in one shot. That is the safe default because the topology needs etcd client ports, etcd peer ports, node API ports, observe ports, and SQL/Postgres ports that must not overlap.

Safe change rule:

- if a component can accept an already-bound listener, prefer that pattern
- do not drop a reservation early and then hope the real component binds quickly enough

The TCP proxy helper follows the safer "pass the bound listener in" model via `spawn_with_listener(...)`.

## Real-binary verification is part of harness startup

The harness treats binaries as part of the trusted test environment. `src/test_harness/binaries.rs` provides the public "require this binary" entrypoints, but the actual trust logic lives in `src/test_harness/provenance.rs`.

The important contributor mental model is:

- installers create an attestation manifest under `.tools/`
- the harness validates path constraints, metadata, hashes, sizes, and version expectations
- a missing or mismatched binary is a hard test failure

This is why harness-backed scenarios are allowed to make strong claims: they are not running against arbitrary executables found on the machine.

## Process helpers: etcd and Postgres boundaries

The harness has narrow helpers for the real binaries, and the separation between them matters.

### etcd: cluster spawning and readiness

`src/test_harness/etcd3.rs` owns member data directories, process spawn, readiness checks, and cluster shutdown. `spawn_etcd3_cluster(...)` validates member names and port uniqueness before launching anything, then waits for TCP reachability and a KV round-trip before considering the cluster ready.

That means a scenario that starts with an etcd cluster can assume more than "the process exists." It can assume the store actually accepts reads and writes.

### Postgres: plain helper versus managed-runtime path

`src/test_harness/pg16.rs` owns `prepare_pgdata_dir(...)`, `spawn_pg16(...)`, and `spawn_pg16_for_vanilla_postgres(...)`.

The sharp edge here is intentional:

- `spawn_pg16(...)` is the plain helper for launching a vanilla Postgres instance
- `spawn_pg16_for_vanilla_postgres(...)` is the explicit escape hatch for tests that truly need raw `postgresql.conf` lines
- pgtuskmaster-managed startup scenarios should still prove behavior through the runtime and process materialization path, not by sneaking config directly into the vanilla helper

If a proposed test change needs direct `postgresql.conf` edits, treat that as an exception that must be justified in the test.

## Fault injection: proxy links are the network boundary

`src/test_harness/net_proxy.rs` provides `TcpProxyLink`, which is the harness mechanism for blocking or delaying traffic without rewriting the runtime.

Each proxy link can switch between:

- `PassThrough`
- `Blocked`
- `Latency { upstream_delay_ms, downstream_delay_ms }`

The proxy listeners run on their own dedicated thread with a single-threaded tokio runtime. That design avoids starving the proxy accept loop when the main test runtime is busy and makes network-fault scenarios more deterministic.

If you are debugging a partition scenario, this file is usually more important than the raw etcd or API helper modules.

## Scenario support: where assertions really live

The harness does not prove HA safety by itself. The scenario support modules under `tests/ha/support/` turn the returned cluster handle into evidence.

The most important companion module is `tests/ha/support/observer.rs`, which collects repeated API or SQL observations and rejects "proof" that does not have enough successful samples. The multi-node and partition scenario modules build on that observer and on `NodeHandle.api_observe_addr` to prove properties such as:

- no dual primary during the sampled window
- fail-safe is observed on the expected nodes
- healing returns the cluster to the intended topology

That is why harness changes should be reviewed together with scenario support changes. A startup convenience change can accidentally weaken the quality of the evidence.

## Runtime config fixtures: keep shared defaults generic

`src/test_harness/runtime_config.rs` provides `RuntimeConfigBuilder` and sample config fragments. Its role is to give tests a valid managed-config baseline, not to hide topology.

Good use:

- start from `RuntimeConfigBuilder::new()`
- layer node-specific ports, paths, IDs, auth, or DCS scope in the harness/scenario code

Bad use:

- baking scenario topology into the shared builder
- using the shared builder to obscure parser-shape tests

Keeping that split makes it much easier to reason about what the scenario is actually proving.

## Safe ways to change the harness

This area has several sharp edges that are easy to break accidentally:

- Keep all new files, logs, and sockets inside the namespace.
- Preserve aggregated cleanup errors; do not convert them into best-effort silent behavior.
- Reserve ports before building topology, and keep reservations alive until the consumer is ready.
- Add new long-lived resources to both startup cleanup and `TestClusterHandle::shutdown(...)`.
- Keep topology-specific config explicit in startup/scenario code instead of burying it in generic fixtures.
- Use the proxy layer for network-fault tests instead of inventing ad hoc blocking mechanisms.

## Failure triage: where to start in code

Use this map when a harness-backed test fails:

- cluster never came up: start in `src/test_harness/ha_e2e/startup.rs`
- etcd readiness or membership issue: open `src/test_harness/etcd3.rs`
- Postgres spawn or stale-data-dir issue: open `src/test_harness/pg16.rs`
- port collision or address reuse issue: open `src/test_harness/ports.rs`
- partition or latency behavior looks wrong: open `src/test_harness/net_proxy.rs`
- assertion quality looks weak or "insufficient evidence" fails: open `tests/ha/support/observer.rs`

That path usually gets you to the real bug faster than starting in the top-level test file.

## Adjacent subsystem connections

The harness is only meaningful in combination with the runtime and HA pipeline it exercises:

- Read [Testing System Deep Dive](./testing-system.md) for the full test-layer map and gate ownership.
- Read [Worker Wiring and State Flow](./worker-wiring.md) before adding new observations to HA scenarios.
- Read [API and Debug Contracts](./api-debug-contracts.md) when deciding whether a scenario should observe API state, SQL role state, or both.

## Evidence pointers

To verify the claims in this chapter directly, start here:

- `src/test_harness/ha_e2e/startup.rs`
- `src/test_harness/ha_e2e/handle.rs`
- `src/test_harness/ha_e2e/ops.rs`
- `src/test_harness/namespace.rs`
- `src/test_harness/ports.rs`
- `src/test_harness/etcd3.rs`
- `src/test_harness/pg16.rs`
- `src/test_harness/net_proxy.rs`
- `src/test_harness/provenance.rs`
- `tests/ha/support/observer.rs`
