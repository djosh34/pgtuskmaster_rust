---
## Task: Replace Stringly DCS Writes With Typed Writer API <status>done</status> <passes>true</passes> <passing>true</passing>

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
- [x] Files/modules checklist:
- [x] `src/dcs/store.rs`: add typed writer API (`write_leader_lease`, `delete_leader`, `clear_switchover`) that handles path + JSON; keep raw `DcsStore` for modules that still require low-level writes.
- [x] `src/dcs/state.rs`: adjust `DcsWorkerCtx` or related structs to use the new writer wrapper (if required by the visibility change). (No change required after compile verification.)
- [x] `src/ha/state.rs`: replace `dcs_store` field with typed writer interface type and update constructor/contract stub inputs.
- [x] `src/ha/worker.rs`: remove direct JSON serialization for DCS writes; use typed writer methods instead; update error mapping accordingly.
- [x] `src/ha/worker.rs` tests: update `RecordingStore`/stubs to match new writer interface.
- [x] `src/worker_contract_tests.rs`: update `ContractStore` to implement the typed writer interface.
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test` — all BDD features pass
</acceptance_criteria>

<plan>
### Research snapshot (parallel)
- Completed parallel scan of the affected modules (`src/dcs/store.rs`, `src/dcs/state.rs`, `src/ha/state.rs`, `src/ha/worker.rs`, `src/worker_contract_tests.rs`, plus `src/dcs/mod.rs` and `src/dcs/worker.rs`) to map every direct `write_path`/`delete_path` usage and all HA contract stubs.
- Verified HA is the only worker currently building leader/switchover key strings directly in runtime dispatch; DCS worker member writes are already routed via a typed helper (`write_local_member`), which is the right pattern to extend.

### Execution plan
1. Introduce a typed HA writer interface in `src/dcs/store.rs`.
- Add a new trait dedicated to HA write operations (for example `DcsHaWriter`) with typed methods:
  - `write_leader_lease(&mut self, scope: &str, member_id: &MemberId) -> Result<(), DcsStoreError>`
  - `delete_leader(&mut self, scope: &str) -> Result<(), DcsStoreError>`
  - `clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError>`
- Implement these methods once in `dcs::store` so path building and serialization are centralized in DCS.
- Keep robust error mapping (no panic/unwrap/expect) by converting serde failures into `DcsStoreError::Decode`.

2. Bridge existing stores into the typed writer API without broad refactors.
- Add a blanket impl of the new typed trait for all types implementing `DcsStore` (and `?Sized`) so existing concrete stores (`EtcdDcsStore`, test doubles, contract stubs) automatically support typed HA calls.
- Keep current `DcsStore` trait intact and publicly reachable for modules that still need raw writes (notably API controller and e2e fixtures), while removing raw calls from HA entirely.

3. Update HA state context to depend on typed interface instead of raw store.
- In `src/ha/state.rs`, change `HaWorkerCtx.dcs_store` and `HaWorkerContractStubInputs.dcs_store` from `Box<dyn DcsStore>` to `Box<dyn DcsHaWriter>`.
- Update imports accordingly and ensure constructor wiring remains unchanged for callers that pass `Box::new(<type implementing DcsStore>)`.

4. Refactor HA dispatch logic to use typed DCS operations only.
- In `src/ha/worker.rs`, remove direct leader JSON serialization and direct path writes/deletes in `dispatch_actions`.
- Replace:
  - `write_path(/scope/leader, json(LeaderRecord))` with `write_leader_lease(...)`
  - `delete_path(/scope/leader)` with `delete_leader(...)`
  - `delete_path(/scope/switchover)` with `clear_switchover(...)`
- Keep HA-local path helpers only for `ActionDispatchError.path` population; they must no longer be used for DCS write/delete calls.
- Preserve best-effort dispatch semantics and typed `ActionDispatchError` collection.

5. Adjust HA worker tests to align with typed behavior.
- In `src/ha/worker.rs` tests, keep `RecordingStore` implementing `DcsStore`; rely on blanket impl for the new typed trait.
- Update assertions to validate effect (writes/deletes to leader/switchover) while no longer depending on HA-side manual path building internals.
- Ensure failure-path tests still validate `ActionDispatchError::{DcsWrite,DcsDelete,ProcessSend}` behavior.

6. Update worker contract test scaffolding.
- In `src/worker_contract_tests.rs`, change HA contract stub input field type to `Box<dyn DcsHaWriter>`.
- Keep `ContractStore` implementing `DcsStore`; typed writer support should be inherited via blanket impl.

7. Confirm whether `src/dcs/state.rs` needs changes.
- Expected minimal/no structural changes in `DcsWorkerCtx` (it still needs raw watch-drain behavior), but re-run compile checks to confirm there is no trait import fallout from the new typed abstraction.

8. Apply checklist updates in task file during execution.
- Tick file/module checklist items as each code/test change lands.
- Keep status tags aligned at completion (`<status>done</status> <passes>true</passes> <passing>true</passing>`), but only after all required gates pass.

9. Verification and required gates (sequential, full evidence).
- Run:
  - `make check`
  - `make test`
  - `make test`
  - `make lint`
- Run sequentially (not parallel cargo invocations) for deterministic results and avoid cargo artifact races noted in prior learnings.
- Capture and inspect command outcomes; do not skip any test category.

10. Finalization once gates pass.
- Set `<passing>true</passing>` in this task file.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changed files (including `.ralph` artifacts) with:
  - `task finished task-typed-dcs-writes-and-encapsulation: <summary with gates/evidence/challenges>`
- Append any new durable learning to `AGENTS.md`.
</plan>

NOW EXECUTE
