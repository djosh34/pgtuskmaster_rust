---
## Task: Implement real etcd3-backed DCS store adapter and integration tests <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] Choose and add an etcd3 Rust client dependency (example: `etcd-client`) and keep it behind a focused module boundary.
- [ ] Implement `EtcdDcsStore` (or similarly named) that:
- [ ] connects to configured endpoints with timeouts,
- [ ] can write the local member record to `/{scope}/member/{id}` exactly,
- [ ] can subscribe to watch prefixes for the scope and emit `DcsWatchEvent` values compatible with `refresh_from_etcd_watch`.
- [ ] Extend or adapt `DcsStore` trait only as needed; document any breaking changes and update existing unit tests accordingly.
- [ ] Add integration tests that:
- [ ] spawn real etcd3 via `src/test_harness/etcd3.rs`,
- [ ] perform at least one put + delete cycle and assert observed watch updates,
- [ ] cover error mapping for unreachable endpoints and JSON decode failures.
- [ ] Ensure tests are self-skipping when etcd binary is missing, but support the enforcement mode introduced by task `10a`.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>

