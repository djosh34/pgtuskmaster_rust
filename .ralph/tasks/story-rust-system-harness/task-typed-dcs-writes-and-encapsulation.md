---
## Task: Replace Stringly DCS Writes With Typed Writer API <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Eliminate raw path/string DCS writes from HA by introducing a typed DCS writer API and restricting access to low-level write/delete operations.

**Scope:**
- `src/dcs/store.rs`: introduce typed writer helpers (e.g., leader lease write/delete) and hide raw write/delete from non-DCS modules where possible.
- `src/dcs/state.rs`: update contexts to carry the new typed writer interface (or wrapper) as needed.
- `src/ha/state.rs`: replace `dcs_store: Box<dyn DcsStore>` with a typed writer interface.
- `src/ha/worker.rs`: replace manual path building + JSON encoding with typed DCS writer calls.
- Tests in `src/ha/worker.rs` and `src/worker_contract_tests.rs`: update stubs/mocks to implement the new typed writer interface.

**Context from research:**
- HA currently writes directly to DCS using `write_path`/`delete_path` with a string path and JSON (`src/ha/worker.rs`).
- DCS already has a typed helper for member writes (`write_local_member`) in `src/dcs/store.rs`; extend this pattern for leader lease and other HA writes.
- The raw `DcsStore` trait is used in `src/dcs/state.rs`, `src/ha/state.rs`, and multiple tests. Rework visibility so non-DCS modules cannot call raw write/delete directly.

**Expected outcome:**
- HA uses only typed DCS writer methods to write/delete leader lease.
- Low-level `write_path`/`delete_path` are not reachable from HA or other non-DCS modules.
- Tests compile and pass with updated stubs for the new interface.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Files/modules checklist:
- [ ] `src/dcs/store.rs`: add typed writer API (e.g., `write_leader`, `delete_leader`) that handles path + JSON; restrict visibility of raw `DcsStore` write/delete to the `dcs` module.
- [ ] `src/dcs/state.rs`: adjust `DcsWorkerCtx` or related structs to use the new writer wrapper (if required by the visibility change).
- [ ] `src/ha/state.rs`: replace `dcs_store` field with typed writer interface type and update constructor/contract stub inputs.
- [ ] `src/ha/worker.rs`: remove `leader_path` usage and direct JSON serialization for DCS writes; use typed writer methods instead; update error mapping accordingly.
- [ ] `src/ha/worker.rs` tests: update `RecordingStore`/stubs to match new writer interface.
- [ ] `src/worker_contract_tests.rs`: update `ContractStore` to implement the typed writer interface.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
