---
## Task: Add network partition e2e chaos tests with proxy fault injection <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
**Goal:** Validate split-brain safety and recovery under true network partition conditions using a controllable proxy layer for etcd, postgres, and API traffic.

**Scope:**
- Introduce a network-fault harness (toxiproxy integration or in-repo proxy utility) to inject partitions, latency, and disconnects.
- Route etcd, postgres, and API connections through the proxy in dedicated e2e tests.
- Add partition matrix scenarios (minority isolation, primary isolation, API path isolation, heal/rejoin) with explicit split-brain assertions.
- Verify post-heal convergence and data integrity through API and SQL checks.

**Context from research:**
- Current e2e tests do not model real network partitions.
- Requirement explicitly asks for separate e2e partition tests and no split-brain guarantees.
- Existing test harness can host additional helper processes and deterministic lifecycle control.

**Expected outcome:**
- Dedicated partition chaos e2e suites prove no dual-primary behavior and safe recovery for partition/heal cycles.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: new proxy harness module(s) (for example `src/test_harness/net_proxy.rs`), e2e partition scenario files, fixture wiring for etcd/postgres/api via proxy endpoints, assertions covering split-brain prevention and post-heal convergence/data checks
- [x] `make check` — passes cleanly
- [x] `make test` — log captured at `.ralph/evidence/story-rust-system-harness/28-task-e2e-network-partition-chaos-no-split-brain/gates/make-test.log`; no `evaluation failed` marker and command exited 0
- [x] `make lint` — log captured at `.ralph/evidence/story-rust-system-harness/28-task-e2e-network-partition-chaos-no-split-brain/gates/make-lint.log`; no `evaluation failed` marker and command exited 0
- [x] `make test` — all BDD features pass
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Verified)

### Skeptical verification deltas (16-track pass)
- Changed plan item: removed nondeterministic `FlakyDrop` mode from the first implementation; keep deterministic `PassThrough`, `Blocked`, and bounded `Latency` only so e2e assertions remain stable and reproducible.
- Changed plan item: require force-closing active proxied flows on any mode transition to `Blocked` so long-lived etcd watch streams are truly severed.
- Changed plan item: include a dedicated scenario assertion that API-path-only isolation does not trigger promotion (primary identity should remain stable after transient API-only cut).
- Changed plan item: run targeted partition suite first under `RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0` before full gates to reduce mount/link/test flake risk captured in AGENTS learnings.
- Verification track 1: module wiring compatibility (`src/ha/mod.rs`, `src/test_harness/mod.rs`) for additive new files.
- Verification track 2: policy guard tokens for `src/ha/e2e_*.rs` to avoid forbidden direct-control patterns.
- Verification track 3: startup DCS probe path in `runtime/node.rs` to ensure proxied endpoints are applied at bootstrap.
- Verification track 4: etcd reconnect behavior in `dcs/etcd_store.rs` to ensure blocked links cause reconnect attempts.
- Verification track 5: port reservation strategy for etcd peers + node ports and additive proxy ports.
- Verification track 6: pgdata permissions expectation (`0700`) and fixture compatibility.
- Verification track 7: API probing/readiness helper reuse and timeout boundaries.
- Verification track 8: SQL helper reuse for integrity/digest convergence checks.
- Verification track 9: no-unwrap/no-expect/no-panic policy compliance for new harness code.
- Verification track 10: artifact path layout and naming for partition timelines/summaries.
- Verification track 11: cleanup semantics for async tasks and child-process lifecycle interactions.
- Verification track 12: fault helper API naming clarity and deterministic behavior.
- Verification track 13: scenario sequencing to avoid conflating failover/fencing with API-only path isolation.
- Verification track 14: acceptance-criteria mapping coverage from matrix scenarios to explicit assertions.
- Verification track 15: canonical gate order and evidence capture expectations.
- Verification track 16: bounded latency mode limits to avoid hiding true deadlinks during blocked partitions.

### Parallel exploration tracks completed (15)
- Track 1: audited current HA e2e topology in `src/ha/e2e_multi_node.rs` and confirmed 3-node + 3-etcd baseline exists but no partition injection.
- Track 2: validated test-harness modules in `src/test_harness/` and confirmed no generic TCP fault proxy module currently exists.
- Track 3: re-read `src/test_harness/etcd3.rs` to confirm cluster lifecycle APIs are additive and suitable for proxy endpoint overlays.
- Track 4: re-read `src/test_harness/ports.rs` to confirm structured topology port reservation helper can be reused for proxy listeners.
- Track 5: audited API-only post-start policy guard (`tests/policy_e2e_api_only.rs`) and confirmed external process/network fault injection is explicitly allowed.
- Track 6: audited e2e fixture control/observation patterns and confirmed split-brain checks (`assert_no_dual_primary_window`) and SQL integrity helpers already exist.
- Track 7: inspected stress scenarios and artifact conventions to reuse deterministic timeline/summary output patterns.
- Track 8: audited etcd store behavior (`src/dcs/etcd_store.rs`) and confirmed endpoint lists can be safely pointed at proxies.
- Track 9: audited runtime startup path (`src/runtime/node.rs`) and confirmed startup DCS probe uses configured endpoints, so proxy wiring must account for bootstrap/reconnect.
- Track 10: confirmed `Cargo.toml` has no toxiproxy client dependency and no existing toxiproxy binary orchestration, making in-repo proxy utility the lowest-risk first implementation.
- Track 11: confirmed current e2e module wiring (`src/ha/mod.rs`) supports adding a dedicated partition test module under `#[cfg(test)]`.
- Track 12: scanned for existing network tools (`proxy`, `copy_bidirectional`, tc/iptables wrappers) and found none; new harness code is required.
- Track 13: validated no-unwrap/expect/panic lint policy and captured where new async proxy control APIs must return rich `Result` errors.
- Track 14: confirmed required completion gates and order remain `make check`, `make test`, `make lint`.
- Track 15: validated task lifecycle protocol for this file (`TO BE VERIFIED` -> skeptical delta -> `NOW EXECUTE`) and prepared this draft accordingly.

### Proposed architecture
1. Add an in-repo controllable network proxy harness in `src/test_harness/net_proxy.rs`.
- Implement TCP forwarding listeners with runtime fault controls using `tokio`.
- Expose per-link control modes:
- `PassThrough` (normal forwarding),
- `Blocked` (deny new connections, terminate active flows),
- `Latency` (inject bounded per-direction delay),
- `Latency` is optional and bounded; no random drop mode in v1.
- Ensure active connection tasks are tracked and can be force-closed when partition mode flips, so long-lived etcd watch/API streams are actually severed.
- Use explicit control handles (no global mutable state) and full error propagation through `HarnessError`.

2. Export proxy harness via `src/test_harness/mod.rs`.
- Add `pub(crate) mod net_proxy;`.
- Extend `HarnessError` only if necessary with non-panicking, actionable variants for proxy orchestration failures.

3. Add dedicated partition e2e module `src/ha/e2e_partition_chaos.rs`.
- Keep existing `e2e_multi_node` stable; implement partition chaos as separate black-box suite for clarity and isolation.
- Wire module in `src/ha/mod.rs` under `#[cfg(test)]`.

4. Build a partition-capable fixture in new e2e module.
- Reuse existing 3-node/3-etcd spawn primitives.
- For each node, create node-specific etcd proxy endpoints (node->etcd-member links) and configure node runtime `dcs.endpoints` to those proxies.
- For API and SQL traffic used by tests, route test clients through proxy listeners before reaching each node’s real API/postgres endpoint.
- Preserve external-control policy: post-start actions remain API reads/allowed admin calls, SQL probes/writes, and network/process fault injection only.

5. Add deterministic fault control API in fixture.
- Helpers such as:
- `partition_node_from_etcd(node_id)`,
- `partition_primary_from_etcd(current_primary_id)`,
- `isolate_api_path(node_id)`,
- `heal_all_network_faults()`.
- Each helper updates proxy modes and records timeline events with absolute member/link identities.

6. Add partition chaos scenario matrix tests in `src/ha/e2e_partition_chaos.rs`.
- Scenario A: minority isolation.
- Isolate one replica node from etcd/API/SQL client paths as appropriate.
- Assert cluster stays single-primary; isolated node does not create dual-primary evidence.
- Heal and verify rejoin + table digest convergence.
- Scenario B: primary isolation from etcd majority.
- Identify stable primary, cut its etcd links via proxy controls.
- Assert promotion elsewhere, former primary demotes/fences, and no dual-primary window.
- Heal and verify convergence/data continuity.
- Scenario C: API path isolation only.
- Partition test-client API route to a node while etcd/postgres paths remain healthy.
- Assert observation failures are scoped to API path, cluster safety remains intact, and no split-brain evidence appears.
- Scenario D: mixed partition + heal/rejoin soak.
- Apply multi-link partition (for example primary etcd cut + one API cut), hold window, then heal.
- Assert eventual stable single primary and SQL digest consistency across all nodes.

7. Strengthen invariants and diagnostics.
- Reuse and extend existing helpers for:
- no dual-primary checks over sampled windows,
- phase-history transition assertions (former primary demoted, promoted node observed),
- SQL key uniqueness and table digest convergence post-heal.
- Add explicit failure context including proxy mode snapshot + recent API sample ring on assertion errors.

8. Evidence and gate execution plan.
- Add partition artifact directory (for example `.ralph/evidence/28-e2e-network-partition-chaos/`) for timelines and optional JSON summaries.
- Execute required gates in order:
- `make check`
- `make test`
- `make test`
- `make lint`
- Capture marker grep evidence from `make test` and `make lint` logs (`congratulations` / `evaluation failed`) per acceptance criteria.

### Implementation sequencing (for future `NOW EXECUTE`)
1. Harness phase.
- Implement `net_proxy.rs` with connection forwarding, mode switching, forced connection teardown, and unit tests.
- Expose module in harness `mod.rs`.

2. Fixture phase.
- Build partition-aware cluster fixture in `e2e_partition_chaos.rs` with proxy-wired etcd/API/SQL paths.
- Keep startup and shutdown deterministic; ensure all proxy tasks terminate cleanly.

3. Scenario phase.
- Implement the four partition matrix scenarios with explicit split-brain and convergence/data assertions.
- Write timeline and optional summary artifacts with scenario-scoped names.

4. Validation phase.
- Run targeted partition tests first, then full required gates in canonical order.
- Store logs and marker grep outputs under task evidence directory.

### Parallel execution tracks to run in `NOW EXECUTE` (15)
- Track 1: proxy mode enum + control surface.
- Track 2: TCP forwarder listener lifecycle.
- Track 3: active-flow registry and forced teardown behavior.
- Track 4: latency/drop fault injection mechanics.
- Track 5: harness error and cleanup aggregation.
- Track 6: etcd per-node proxied endpoint wiring.
- Track 7: API proxied endpoint wiring.
- Track 8: postgres/SQL proxied endpoint wiring.
- Track 9: partition/heal fixture helper APIs.
- Track 10: minority isolation scenario implementation.
- Track 11: primary isolation scenario implementation.
- Track 12: API path isolation scenario implementation.
- Track 13: mixed partition + heal convergence scenario.
- Track 14: artifact/log output integration.
- Track 15: full gate run and acceptance checklist update.
</execution_plan>

COMPLETED
