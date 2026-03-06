## Task: Implement real etcd3-backed DCS store adapter and integration tests <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Add a production-grade `DcsStore` implementation backed by a real etcd3 instance, and prove it via integration tests using the existing test harness spawner.

**Scope:**
- Implement an etcd3-backed adapter that satisfies the existing `src/dcs/store.rs` `DcsStore` trait (or evolve the trait minimally if required).
- Add integration tests that spawn a real etcd3 process (via `src/test_harness/etcd3.rs`) and verify:
  - writes land on the correct keys,
  - watch streams produce put/delete updates,
  - JSON decode failures are surfaced as typed errors,
  - DCS worker `step_once` can refresh from a real etcd watch path without mocking.

**Context from research:**
- Current DCS worker/store logic is validated only via an in-memory `TestDcsStore`; no etcd client integration exists yet.
- The story contains real etcd3 process spawner support in `src/test_harness/etcd3.rs`, but there is not yet a “behavioral consumer” path exercising real etcd events through DCS.
- E2E tasks (`12`/`13`) require real etcd3 to be wired, so this adapter is a prerequisite for meaningful multi-node HA tests.

**Expected outcome:**
- There is a clear, tested path from “etcd3 watch + puts” → typed `DcsWatchEvent` updates → `DcsState` cache refresh.
- Integration tests provide confidence that the DCS code will behave correctly in real clusters (not just in-memory simulations).

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Choose and add an etcd3 Rust client dependency (example: `etcd-client`) and keep it behind a focused module boundary.
- [x] Implement `EtcdDcsStore` (or similarly named) that:
- [x] connects to configured endpoints with timeouts,
- [x] can write the local member record to `/{scope}/member/{id}` exactly,
- [x] can subscribe to watch prefixes for the scope and emit `DcsWatchEvent` values compatible with `refresh_from_etcd_watch`.
- [x] Extend or adapt `DcsStore` trait only as needed; document any breaking changes and update existing unit tests accordingly.
- [x] Add integration tests that:
- [x] spawn real etcd3 via `src/test_harness/etcd3.rs`,
- [x] perform at least one put + delete cycle and assert observed watch updates,
- [x] cover error mapping for unreachable endpoints and JSON decode failures.
- [x] Ensure tests are self-skipping when etcd binary is missing, but support the enforcement mode introduced by task `10a`.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 2, Skeptically Verified)

### Parallel exploration done (8 tracks)
- `src/dcs/store.rs`: verified trait shape + decode/error surface used by worker.
- `src/dcs/worker.rs`: verified health semantics are driven by `refresh_from_etcd_watch`.
- `src/dcs/etcd_store.rs`: reviewed existing etcd adapter behavior and tests.
- `src/test_harness/etcd3.rs`: reviewed real etcd spawn/teardown contract.
- `src/test_harness/binaries.rs`: reviewed enforcement-compatible binary lookup helpers from task `10a`.
- `Cargo.toml`/`Cargo.lock`: confirmed `etcd-client` dependency is already present.
- `src/ha/e2e_multi_node.rs`: verified adapter is already consumed by real HA fixtures.
- task files under `.ralph/tasks/story-rust-system-harness`: checked story continuity and acceptance wording.

### Skeptical verification pass done (16+ tracks)
- Re-audited etcd client API in local crate sources (`etcd-client` 0.14.1) to confirm streaming watch semantics and start-revision behavior.
- Re-audited watch-state publisher/subscriber lifetime rules to avoid false failures from dropped subscribers in real integration tests.
- Re-audited worker faulting semantics (`had_errors` vs hard decode errors) so real-etcd tests assert the exact expected unhealthy/trust behavior.

### Findings that must be addressed before marking task done
- The adapter exists, but current watch behavior is polling-based (`get + diff`) rather than an etcd watch stream subscription contract.
- Current adapter tests use `require_etcd_bin()` instead of `require_etcd_bin_for_real_tests()`, so they do not align with enforced real-binary policy from `10a`.
- There is no explicit real-etcd integration test proving `step_once` consumes real watch-path events end-to-end (without a mock store).
- JSON decode failure handling is tested in unit paths, but this task needs explicit integration coverage in the real etcd path.

### Implementation phases
1. Harden `EtcdDcsStore` runtime boundary and timeouts
- Keep `DcsStore` trait unchanged unless strictly necessary.
- Ensure connect startup, command round trips, and etcd operations have bounded timeouts and explicit typed `DcsStoreError::Io` mapping.
- Preserve no-`unwrap`/no-`expect` rule and propagate all lock/channel/runtime errors.

2. Implement real watch-prefix subscription path
- Replace polling `get + diff` logic with real etcd watch subscription on `/{scope}/` prefix (`WatchOptions::with_prefix()`).
- Bootstrap with one `get(prefix)` snapshot, emit synthetic `Put` events from that snapshot, record header revision, then create a watch stream from `revision + 1` (`WatchOptions::with_start_revision(...)`).
- Translate watch stream responses into existing `WatchEvent { Put/Delete, path, value, revision }` payloads compatible with `refresh_from_etcd_watch`.
- Treat canceled/compacted watch responses (`compact_revision > 0`) as resync triggers: mark unhealthy, recreate client/watch, and replay bootstrap snapshot before continuing.
- Maintain reconnect loop behavior on watch/connect failures and keep `healthy()` state synchronized with latest successful round trip.

3. Add/adjust integration tests for this task contract
- Real etcd put/delete test:
- Spawn etcd via harness (`spawn_etcd3`) and use `EtcdDcsStore` to write + delete a key.
- Assert expected watch events are observed with bounded waiting.
- Real etcd decode-failure test:
- Write malformed JSON to a known typed key under scope (for example `/{scope}/leader`).
- Drive one `step_once` with that event path and assert resulting DCS state is faulted/not-trusted (decode surfaces through worker unhealthy path).
- Real worker step test (no mock store):
- Build a minimal `DcsWorkerCtx` wired to `EtcdDcsStore`, keep at least one `StateSubscriber<DcsState>` alive for full test lifetime, publish a PG snapshot, run ordered `step_once` cycles, and assert cache/member refresh from real etcd watch events.
- Unreachable endpoint test:
- Keep explicit mapping assertion for unreachable endpoint write/read/watch setup to `DcsStoreError::Io`.

4. Align binary gating with task `10a`
- Update all real-etcd tests in this module to use `require_etcd_bin_for_real_tests()`.
- When binary is missing and enforcement env is off, return early (`Ok(())`) without failure.
- When enforcement env is on, allow helper to fail-fast with `HarnessError::InvalidInput`.

5. Run full required gates sequentially and capture evidence
- Run `make check`, `make test`, `make lint` sequentially (no parallel Cargo gate invocations).
- For `make test` and `make lint`, capture logs and grep for `congratulations` / `evaluation failed` as required by task text.
- If archive/object-file corruption appears, perform deterministic recovery (`cargo clean`) then rerun full gates and record both runs.

6. Finalize task file once execution is complete
- Tick acceptance checkboxes based on actual outcomes only.
- Set `<status>done</status>` and `<passes>true</passes>` only after all mandatory gates succeed.
- Follow Ralph flow (`.ralph/task_switch.sh`) and include `.ralph` artifacts in commit.
</implementation_plan>

NOW EXECUTE
