## Task: Add PostgreSQL Proxy Chaos E2E Coverage <status>not_started</status> <passes>false</passes>

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
- [ ] Review the current pg-proxy wiring and identify the exact existing hooks for blocking or degrading PostgreSQL-path traffic.
- [ ] Add at least one new e2e scenario that uses pg-proxy fault injection as the primary fault mechanism rather than etcd-only or API-only isolation.
- [ ] The new scenario proves there is no dual-primary window during the injected PostgreSQL-path fault.
- [ ] The new scenario proves that writes committed before the fault remain intact after heal/recovery.
- [ ] The new scenario proves that post-heal replication converges across all participating nodes.
- [ ] If multiple pg-path fault shapes are practical, prefer separate tests for each distinct behaviour rather than one oversized scenario with ambiguous failure cause.
- [ ] Reuse the current HA observer/sampling helpers so split-brain assertions remain as strict as the existing suite.
- [ ] Ensure the new tests are parallel-safe with isolated namespaces, ports, proxy links, tables, and artifact names.
- [ ] Add or update helper functions only where the current partition or multi-node support code is missing a clean pg-proxy injection surface.
- [ ] Document in code comments and timeline artifacts exactly which network path is being blocked and why the scenario is meaningful.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If the new scenario belongs in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>
