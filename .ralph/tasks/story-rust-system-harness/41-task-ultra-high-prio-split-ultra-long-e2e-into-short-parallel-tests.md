---
## Task: Ultra-high-priority split ultra-long e2e tests into shorter parallel real-binary tests <status>completed</status> <passes>true</passes> <passing>true</passing> <priority>ultra-high</priority>

<description>
**Goal:** Replace the current ultra-long HA e2e stress scenario(s) with multiple shorter real-binary e2e tests that preserve full coverage and must run in parallel.

**Scope:**
- Decompose each current ultra-long scenario (runtime >= 3 minutes from evidence) into smaller independent real-binary e2e tests with narrow objectives.
- Preserve all existing behavioral coverage and assertions from the original long scenarios.
- Ensure resulting short tests are parallel-safe and designed to run concurrently (no serial-only exemptions).
- Normal `make test` must hard-enforce a total timeout of 2 minutes.
- Normal `make test` must run in full parallel mode; if parallel execution fails, that outcome is a bug to fix.
- Any requirement to run normal `make test` serially is a bug (serial-only operation is not allowed).
- `make test-long` must have no timeout.
- Keep the ultra-long-only target small over time by moving shortened scenarios back into `make test`.
- Document mapping from each original long scenario to its replacement short tests.

**Context from research:**
- A small number of very long HA e2e tests dominate runtime and block development flow.
- The project requires real binaries in these checks, but long duration should not force serial developer loops.
- New short tests must still catch the same failures, not reduce assurance.
- Current evidence identifies only one 3min+ stress scenario:
- `ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql` (`126357..297266` ms on passed runs).
- The other two stress scenarios are ~21-25 seconds and should remain in `make test`.

**Expected outcome:**
- Ultra-long scenarios are functionally replaced by a set of shorter real-binary e2e tests.
- Short replacements are parallelized by default and become part of regular `make test` flow when stable.
- `make test-long` shrinks to only truly unavoidable long-duration tests.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] For every current ultra-long scenario, a traceable set of shorter real-binary e2e tests exists that covers all prior assertions.
- [x] New shorter tests are parallel-safe and executed in parallel with no serial-only exception path.
- [x] Normal `make test` has a hard-enforced total timeout of 2 minutes.
- [x] Normal `make test` runs fully in parallel; any parallel execution failure is tracked as a bug.
- [x] Normal `make test` never requires serial execution; any serial requirement is tracked as a bug.
- [x] `make test-long` has no timeout.
- [x] Coverage mapping artifact is added (old long scenario -> new short test set).
- [x] Any failure discovered only in `make test-long` gains a new short real-binary e2e regression test in `make test`.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

---

## Plan

### 0) Baseline + constraints (do first, no code changes)
- [x] Read/understand current ultra-long test and its assertions:
  - `src/ha/e2e_multi_node.rs` test `ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql`.
- [x] Confirm current `make test` / `make test-long` behavior and where timeouts/parallelism are (Makefile only).
- [x] Confirm current HA harness parallel hazards:
  - Port reservation TOCTOU (ports are “reserved” then released long before bind).
  - Fixed `scope`/`cluster_name` in fixtures increases blast radius if misrouting happens during parallel runs.

### 1) Make real-binary HA harness genuinely parallel-safe (port reservation fixes)
Goal: multiple clusters starting at the same time under default `cargo test` parallelism should not intermittently collide on ports.

- [x] Upgrade port reservation API so reservations can be held and released “just-in-time”.
  - Files: `src/test_harness/ports.rs`, `src/test_harness/ha_e2e/util.rs`, `src/test_harness/ha_e2e/startup.rs`.
  - New/changed APIs (sketch):
    - In `src/test_harness/ports.rs`:
      - Keep `PortReservation::as_slice(&self) -> &[u16]` and add a way to *release a specific port* while keeping the rest reserved:
        - `fn release_port(&mut self, port: u16) -> Result<(), HarnessError>`
        - Semantics: drop the reserved listener for `port` only; keep other ports reserved.
      - Rationale: this is smaller surface area than index-based `take(...)` and avoids brittle ordering coupling.
    - In `src/test_harness/ports.rs`:
      - Extend `HaTopologyPortReservation` with a delegate:
        - `fn release_port(&mut self, port: u16) -> Result<(), HarnessError>`
      - Avoid adding a `layout_clone()` API; `HaTopologyPorts` already derives `Clone` so startup can do:
        - `let topology = reservation.layout().clone();` while keeping the reservation alive.
    - In `src/test_harness/ha_e2e/util.rs`:
      - Replace `allocate_non_overlapping_ports(...) -> Vec<u16>` with a reservation-backed variant:
        - `reserve_non_overlapping_ports(...) -> PortReservation`
      - Preserve the non-overlap retry loop, but keep listeners until startup releases ports just-in-time.

- [x] Refactor `src/test_harness/ha_e2e/startup.rs` startup sequence to release ports only immediately before spawn/bind.
  - Keep the topology reservation alive inside `start_cluster_inner` (do **not** call `into_layout()` early).
    - Use `let topology = reservation.layout().clone();` and keep `reservation` mutable and in-scope.
  - Release reserved ports just-in-time at the actual bind sites:
    - Right before `spawn_etcd3_cluster(...)`, call `reservation.release_port(port)` for every etcd client/peer port.
    - Right before spawning each node runtime (the call that results in Postgres+API binding), release that node’s postgres port and API port.
    - Right before spawning each `TcpProxyLink::spawn(...)`, release the proxy listen port.
  - For API/proxy port allocation, keep their reservations alive until their matching bind/spawn.

- [x] Reduce cross-test interference radius: make fixture identity values unique per namespace.
  - Implemented by suffixing `scope` and `cluster_name` with the harness `namespace_id` inside `ha_e2e::start_cluster` (keeps fixture names stable to avoid socket-path blowups, while preventing DCS cross-talk).
  - Rationale: if a port collision or miswire does happen, shared `scope` makes the failure cascade across clusters more likely.

- [x] Add focused tests for the new reservation behavior (avoid adding heavy real-binary “start N clusters” tests).
  - In `src/test_harness/ports.rs`, extend unit tests to cover:
    - `release_port` succeeds for a reserved port
    - `release_port` errors for an unknown port
    - releasing one port keeps remaining ports reserved (sanity check: other ports still conflict on bind while held)

### 2) Replace the ultra-long no-quorum fencing stress scenario with shorter independent tests
Goal: preserve all assertions/coverage, but split into narrower tests that can run concurrently and complete quickly.

Current ultra-long scenario assertions to preserve (from the existing test):
- stable primary bootstrap + table creation
- etcd majority loss triggers fail-safe
- concurrent SQL workload:
  - commits occur (>0)
  - rejected writes occur (>0) (fencing or transient)
  - after fail-safe is observed + grace window, commits are near-zero (<= tolerance)
  - no split-brain write evidence / no duplicate committed keys / no “hard” SQL failures
- key integrity check on primary table
- artifacts written + cluster shutdown even on failure

- [x] Implement a shared helper that runs the “no-quorum fencing” setup but is parameterized for short tests.
  - File: `src/ha/e2e_multi_node.rs`.
  - Extract “scenario core” into an async function returning the pieces each test asserts on, for example:
    - `bootstrap_primary`
    - `failsafe_observed_at_ms`
    - `workload` stats
    - `ha_stats` sample window
    - `table_name` used
  - Ensure:
    - per-test `scenario_name` is unique
    - per-test `table_name` is unique (avoid any accidental cross-test overlap if ports collide)
    - per-test timeouts are short and explicit (don’t reuse `E2E_SCENARIO_TIMEOUT=300s` for new short tests)
  - Skeptical coverage fix: today `wait_for_all_failsafe(...)` is not “all nodes” strict; add a stricter helper for the short tests:
    - `wait_for_all_nodes_failsafe(...)` (or similar) that requires every node to report fail-safe (or an explicit “not Primary” + fail-safe state), not just “at least one”.

- [x] Add multiple new short real-binary e2e tests (all should be eligible for regular `make test`).
  - Revised split (reduce total cluster startups to keep wall-clock + resource use low under parallel runs):
    1) `e2e_no_quorum_enters_failsafe_strict_all_nodes`
       - Focus: stimulus + *all nodes* observe fail-safe + HA sampling
       - Assertions:
         - `wait_for_all_nodes_failsafe(...)` succeeds within a short bound
         - sample window indicates fail-safe occurred (and is stable enough for cutoff selection)
    2) `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`
       - Focus: workload fencing effectiveness + split-brain checks + key integrity in one cluster
       - Assertions:
         - `committed_writes > 0`
         - `(fencing_failures + transient_failures) > 0`
         - `commits_after_cutoff <= allowed_post_cutoff_commits`
         - no hard SQL failures / no duplicate committed keys (via existing helper)
         - `assert_table_key_integrity_on_node(...)` passes (min rows >= 1)
  - Keep per-test wall clock small by:
    - replacing fixed sleeps with “wait until condition or timeout”
    - shrinking sample windows while preserving invariants
    - shrinking fail-safe wait timeouts to the minimum that remains reliable in CI

- [x] Remove (or demote) the old ultra-long test to avoid re-introducing long serial wall-clock.
  - Preferred: delete the test function entirely once coverage is mapped to the new suite.
  - Alternative (only if needed for diagnostics): keep a longer “soak” variant behind `#[ignore]` and do not include it in Make targets.

### 3) Enforce `make test` total timeout = 2 minutes (and keep `make test-long` unlimited)
- [x] Update `Makefile`:
  - Add `TEST_TIMEOUT_SECS ?= 120`.
  - Add `TEST_TIMEOUT_KILL_AFTER_SECS ?= 15` so timed-out runs are force-killed deterministically.
  - Resolve timeout binary as `timeout` (Linux) or `gtimeout` (macOS coreutils).
  - Add an `ensure-timeout` target and make `test` depend on it.
  - Wrap the *entire* `cargo test --all-targets` invocation in the timeout tool for `make test`.
  - Keep `make test-long` without any timeout wrapper.
  - Fail fast with a clear error if no timeout tool exists (do not silently skip; install requirements instead).
  - Enforce “parallel required” at make-level:
    - if `RUST_TEST_THREADS=1`, fail `make test` with a clear error (serial-only developer loops are not allowed here).
  - Note: This timeout is measured around the command; on cold builds, compilation time may exceed 120s and that failure is intentional under this policy. CI should run `make check` before `make test` so the timed run primarily measures test execution.

### 4) Add the required coverage mapping artifact (old -> new)
- [x] Add a small, explicit mapping document in-repo (not just in commit message), e.g.:
  - `docs/ha-e2e-stress-mapping.md`
  - Include:
    - old test name
    - the new short test names
    - which old assertions each new test covers
    - any intentionally-changed thresholds (and why)

### 5) Verification (must be fully green; no skipping)
- [x] Run `make check`
- [x] Run `make lint`
- [x] Run `make test` (must complete in <= 120s and run with default parallelism)
- [x] Run `make test-long` (no timeout)
- [x] If *any* flake appears only under parallel execution, treat it as a harness bug and fix (do not paper over with serial-only knobs).

---

NOW EXECUTE
