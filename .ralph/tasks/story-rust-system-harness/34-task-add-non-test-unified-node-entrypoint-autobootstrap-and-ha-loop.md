---
## Task: Add non-test unified node entrypoint from start through autonomous HA loop <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Provide one production (non-test) entry path that starts a `pgtuskmaster` node from config only and runs it through bootstrap and HA loop without manual orchestration.

**Scope:**
- Add/extend runtime entry code in non-test modules so node startup is performed through a single canonical entrypoint.
- Ensure startup path decides bootstrap mode from existing state:
- no local PGDATA -> initialize as needed,
- existing upstream primary state -> base backup/follow path,
- existing local state -> resume/reconcile path.
- Wire all required channels/subscribers/workers internally (HA, DCS, process, pginfo, API/debug as required by runtime contracts) so callers only pass config.
- Keep decision logic integrated with existing state/coordination components, not test-only helpers.
- Ensure strict `Result`-based error propagation with no unwrap/expect/panic.

**Context from research:**
- Current request reports missing production-grade "main/entry" flow and ad-hoc startup behavior in tests.
- Existing HA/state/coordination logic already exists but must be invoked through one stable runtime entry interface.
- This must be implemented in non-test code and become the canonical startup surface.

**Expected outcome:**
- A node can be started in production using one entrypoint and config, and it autonomously reaches steady HA operation with correct bootstrap behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: runtime entry module(s), startup/bootstrapping decision layer, worker/channel wiring layer, binary/main invocation path, and any required config schema updates
- [x] Startup behavior validated for key states: empty PGDATA init path, replica/basebackup path, and restart/resume path from existing PGDATA
- [x] Entry API accepts config-only invocation (no test-only coordination hooks required to bring node online)
- [x] No unwrap/expect/panic introduced; all new error paths return typed errors
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Detailed Implementation Plan (Draft 1)

Research baseline for this draft:
- Parallel track mapping completed across `Cargo.toml`, `src/lib.rs`, all worker/state modules, HA e2e fixture, config parser/defaults/schema, process job model, DCS store adapter, and integration/bdd/examples call-sites.
- Key gap confirmed: there is no production node binary and no non-test runtime orchestrator that boots a node from config only.
- Key gap confirmed: startup currently depends on external/manual pre-initialization in tests (`initialize_pgdata`), not autonomous runtime startup decisions.
- Key gap confirmed: there is no process primitive for replica bootstrap (`pg_basebackup`), so fresh replica bootstrap path is not representable in non-test runtime.

### Implementation objectives for this task
1. Add one canonical, non-test node entry API and binary (`pgtuskmaster`) that starts and runs the full node runtime from config only.
2. Add an explicit startup decision layer that selects and executes bootstrap mode before steady HA loop:
   - empty PGDATA + no usable upstream leader => initialize local PGDATA and start
   - empty PGDATA + usable upstream primary => basebackup then follow path
   - existing local PGDATA => resume/reconcile path
3. Keep steady-state behavior in existing workers (pginfo/dcs/process/ha/api/debug), but remove dependence on test-only startup wiring.

### Planned code changes (module-by-module)

1. Runtime entry surface (new non-test module + binary)
- Add `src/runtime/mod.rs` and `src/runtime/node.rs`.
- Expose public entry functions from library:
  - `run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError>`
  - `run_node_from_config_path(path: &Path) -> Result<(), RuntimeError>`
- Add `RuntimeError` typed enum covering:
  - config load/validation
  - startup decision failures
  - process/bootstrap execution failures
  - worker spawn/join failures
  - API bind/DCS connectivity failures
- Add `src/bin/pgtuskmaster.rs`:
  - parses `--config <path>`
  - loads config via `config::load_runtime_config`
  - invokes runtime entry API
  - exits non-zero with descriptive stderr on failure

2. Startup decision layer (non-test, explicit, deterministic)
- Add `StartupPlanner` and `StartupMode` in `src/runtime/node.rs`:
  - `StartupMode::InitializePrimary`
  - `StartupMode::CloneReplica { leader: PgConnInfo }`
  - `StartupMode::ResumeExisting`
- Add local data-dir inspector helper:
  - detect `PG_VERSION` presence for existing cluster
  - detect empty/missing dir for fresh bootstrap
  - reject ambiguous partial state (non-empty dir without valid PG marker) with typed error
- Add DCS bootstrap snapshot read helper for startup planning:
  - use existing DCS decode/key logic to build one-time cache view before worker startup
  - derive current leader/member metadata from cache
- Decision rules:
  - if local PGDATA is valid => `ResumeExisting`
  - else if upstream healthy leader metadata is available => `CloneReplica`
  - else => `InitializePrimary`

3. Process surface extension for basebackup bootstrap
- Extend `config::schema::BinaryPaths` to include `pg_basebackup`.
- Update:
  - `src/config/schema.rs`
  - `src/config/defaults.rs`
  - `src/config/parser.rs` validations
  - all config fixtures in tests/examples that construct `BinaryPaths`
  - `src/test_harness/binaries.rs` to resolve `pg_basebackup`
- Add process job support:
  - new job spec `BaseBackupSpec` in `src/process/jobs.rs`
  - add `ProcessJobKind::BaseBackup`
  - extend active kind/timeout mapping and command builder in `src/process/worker.rs`
  - command shape: `pg_basebackup -h <host> -p <port> -U <user> -D <data_dir> -Fp -Xs -R`
  - retain strict `Result` propagation and typed `ProcessError` mapping

4. Startup executor (reusing process command semantics)
- Add startup executor that runs bootstrap jobs before steady-state worker loops.
- Reuse process job command-building path (do not duplicate shell command composition in two places):
  - extract shared command build helper from process worker into reusable internal function if needed
  - execute with timeout handling and explicit failure context
- Startup action sequence:
  - `InitializePrimary`:
    - run bootstrap/initdb job
    - run start-postgres job
  - `CloneReplica`:
    - run basebackup job against leader conninfo
    - run start-postgres job
  - `ResumeExisting`:
    - run start-postgres job (idempotent handling remains via process outcome)

5. Unified worker/channel wiring (non-test constructors)
- Add non-test constructors (not `contract_stub`) for worker contexts so runtime can wire everything from config:
  - `ProcessWorkerCtx::new_runtime(...)`
  - `HaWorkerCtx::new_runtime(...)`
  - `ApiWorkerCtx::new_runtime(...)` (or equivalent constructor wrapper)
  - `DebugApiCtx::new_runtime(...)`
- Runtime wiring responsibilities (inside entrypoint only):
  - create initial state channels
  - create DCS stores for dcs/ha/api workers
  - create process inbox channel
  - wire debug snapshot subscriber into API worker
  - set production clocks (`system_now_unix_millis`/system wall time)
  - spawn and run long-lived worker loops

6. Bootstrap/reconcile alignment with existing coordination model
- Keep HA decide/dispatch semantics intact for steady state.
- Ensure startup planner only performs pre-loop bootstrap selection and does not introduce test-only override paths.
- Use DCS snapshot metadata for leader-aware replica bootstrap decision, then let HA loop own ongoing failover/fencing transitions.

7. Contract updates for API/debug/runtime
- Ensure API `GET /ha/state` continues to work by always wiring debug snapshot subscriber during runtime startup.
- Ensure debug worker starts with `AppLifecycle::Running` once runtime is fully bootstrapped and workers are active.

### Required tests and validation additions

1. New unit tests
- startup planner matrix tests:
  - empty data dir + no leader => initialize
  - empty data dir + leader => clone replica
  - existing data dir => resume
  - ambiguous/partial data dir => typed error
- process command tests for `BaseBackupSpec`.
- config parser/default tests for new `pg_basebackup` binary path.

2. New integration/e2e tests (non-optional, real binaries where required)
- add runtime integration test that launches node via new unified entrypoint with config-only input:
  - fresh primary bootstrap path
  - replica bootstrap-from-leader path (basebackup)
  - restart/resume path using pre-existing PGDATA
- ensure tests assert no direct harness-only pre-initialization shortcut for these flows.

3. Binary smoke tests
- add/extend `tests/cli_binary.rs` style checks to verify `pgtuskmaster` binary exists and runs help/config parse path.

### Execution phases (for `NOW EXECUTE`)

Phase 1: Runtime skeleton + binary entry
- Add runtime modules, runtime error type, and `pgtuskmaster` binary.
- Export runtime API from `src/lib.rs`.

Phase 2: Config and process surface for basebackup
- Add `pg_basebackup` to config schema/defaults/parser + fixture updates.
- Add `BaseBackupSpec` and process job support with unit tests.

Phase 3: Startup planner + executor
- Implement data-dir inspection, DCS startup snapshot read, decision matrix, and startup action execution.
- Add planner unit tests and startup execution tests.

Phase 4: Production worker wiring constructor path
- Add non-test context constructors and unified runtime wiring.
- Ensure API/debug subscribers are correctly connected and loop launch is centralized.

Phase 5: Integration coverage and gate hardening
- Add/adjust real integration tests for init/basebackup/resume through unified entrypoint.
- Remove any new startup shortcuts introduced during implementation.
- Run required gates and update task checklist only after all pass.

### Skeptical-risk checklist to enforce during implementation
- Do not bypass process worker command semantics with ad-hoc shell calls.
- Do not add any unwrap/expect/panic/todo/unimplemented in runtime or tests.
- Treat partial PGDATA as explicit failure, not implicit destructive cleanup.
- Ensure startup decision uses concrete DCS evidence; avoid assuming leader availability without member metadata.
- Ensure all new config fields are covered in examples/tests outside `src/` to prevent `--all-targets` breakages.
- Keep startup logic additive and non-test; avoid coupling with `test_harness`.

### Completion gate checklist (to tick in execute phase)
- [x] Runtime entry module(s) implemented and exported.
- [x] Startup decision layer implemented with init/basebackup/resume matrix tests.
- [x] Worker/channel wiring centralized in unified runtime entry path.
- [x] Binary/main path implemented for `pgtuskmaster --config ...`.
- [x] Config schema/default/parser updates complete (including external fixtures/examples/tests).
- [x] `make check` passes.
- [x] `make test` passes.
- [x] `make test` passes.
- [x] `make lint` passes.

## Deep Skeptical Verification (Draft 2 Delta)

Changed items after deep verification against current code structure:

1. Runtime wiring strategy changed (important alteration)
- Draft 1 required adding `new_runtime(...)` constructors for multiple worker contexts.
- Revised plan: do not force constructor churn first. Build runtime contexts directly in `src/runtime/node.rs` using existing crate-visible context structs/fields, and only add minimal constructor wrappers when direct construction becomes repetitive.
- Rationale: all required context types are crate-visible already, while constructor churn across api/ha/process/debug would create avoidable surface-area risk.

2. DCS startup evidence source changed
- Draft 1 proposed a generic DCS startup snapshot helper abstracted through existing traits.
- Revised plan: use an explicit startup probe backed by `EtcdDcsStore::connect(...)` and immediate `drain_watch_events()` to decode one-time leader/member evidence with existing `refresh_from_etcd_watch(...)`.
- Rationale: current `DcsStore` trait has watch-drain semantics but no dedicated point-in-time snapshot API; Etcd store connect path already bootstraps initial snapshot events.

3. Resume/start behavior hardened
- Add explicit pre-start guard in startup executor for `ResumeExisting`:
  - if `postmaster.pid` exists, skip initial `StartPostgres` and let `pginfo`/HA reconcile.
  - if marker absent, execute `StartPostgres`.
- Rationale: existing `pg_ctl start` command path does not currently expose stderr details, so blind restart attempts can fail noisily on already-running clusters.

4. Test placement clarified to match module visibility
- Keep planner/executor unit tests in `src/runtime/node.rs` (crate-internal tests).
- Keep binary smoke tests in `tests/cli_binary.rs` style external integration tests.
- Add real-binary runtime entry tests as crate tests to access crate-private HA/DCS/process/debug internals without widening visibility.

Execution confirmation:
- Plan is now internally consistent with current crate visibility and store semantics.
- `NOW EXECUTE` phase should implement exactly this revised plan with no additional exploration beyond implementation details.

NOW EXECUTE
