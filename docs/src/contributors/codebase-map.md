# Codebase Map

This project is structured around **runtime responsibilities**. The modules are not “layers” (utils → services → controllers); they are closer to a set of **workers** with explicit inputs/outputs and a small number of side-effect boundaries.

If you are new to the codebase, the fastest way to build a correct mental model is:

1. understand where startup happens and where steady-state begins
2. understand which worker owns which piece of state
3. understand which components are allowed to perform side effects (DCS writes, process jobs).

## The top-level runtime shape

At a high level, the node runtime does three things:

1. **load + validate config**
2. **plan and execute startup** (bootstrap / clone / resume)
3. **run steady-state workers concurrently** (pginfo, dcs, ha, process, api, debug snapshot, log ingest).

The canonical entrypoint is `src/runtime/node.rs`:

- `run_node_from_config_path` → `run_node_from_config`
- `plan_startup` (inspect data dir + probe DCS)
- `execute_startup` (run bootstrap/basebackup/start-postgres as needed)
- `run_workers` (create shared state channels, wire worker contexts, `tokio::try_join!` all workers).

The CLI binary wires this up in `src/bin/pgtuskmaster.rs`.

## Primary modules and what they own

The following directories are the “spine” of the implementation. When you are making a change, start by deciding which of these modules should *own* it.

### `src/runtime/`

Owns:

- startup planning and execution
- construction of state channels (`state::new_state_channel`)
- wiring worker contexts (who receives what, who publishes what)
- binding the Node API listener and configuring TLS.

This module should mostly be about orchestration, not business logic.

### `src/state/`

Owns:

- the “latest snapshot” state channel model (`tokio::sync::watch` wrapped as `StatePublisher`/`StateSubscriber`)
- version and timestamp semantics (`Versioned<T>`, `Version`, `UnixMillis`)
- shared error and status types (`WorkerStatus`).

This is where the system’s “what does latest mean?” semantics live.

### `src/pginfo/`

Owns:

- observing local Postgres via SQL and deriving a typed view (`PgInfoState`)
- classifying reachability/health (`SqlStatus`, `Readiness`)
- publishing a snapshot that other workers can use without doing their own SQL probing.

If a decision depends on local Postgres reality, it should flow through pginfo.

### `src/dcs/`

Owns:

- the DCS cache (`DcsCache`) and “is DCS trustworthy right now?” (`DcsTrust`)
- reading and decoding watch events (`refresh_from_etcd_watch`, key parsing in `dcs/keys.rs`)
- publishing local membership records (member metadata derived from pginfo)
- exposing a small writer interface used by HA and API for *intent/coordination* updates.

The key distinction here is **read model vs write model**:

- reads: watch stream + cache
- writes: small explicit paths (leader lease, switchover intent, config/init records).

### `src/ha/`

Owns:

- the lifecycle state machine (`HaPhase`, `HaState`)
- pure decision logic (`ha/decide.rs`)
- mapping “what should happen” into side effects (`ha/worker.rs` dispatch: DCS writes/deletes and process job requests).

The HA module is deliberately split:

- `decide(...)` is intended to be pure and testable
- dispatch is a boundary that can fail and is surfaced via worker status.

### `src/process/`

Owns:

- concrete action execution against the local host: running `pg_ctl`, `pg_rewind`, `pg_basebackup`, `initdb`, etc.
- the state machine for in-flight work (`ProcessState`) and job kinds (`ProcessJobKind`)
- timeouts and output capture for subprocesses.

This module is the “side effects for Postgres” boundary.

### `src/api/` and `src/debug_api/`

Owns:

- operator-facing HTTP request routing (`api/worker.rs`)
- controller logic and DCS intent writes (for example switchover requests in `api/controller.rs`)
- debug snapshot building (`debug_api/worker.rs`, `debug_api/snapshot.rs`)
- debug projection for “verbose” client payloads (`debug_api/view.rs`).

The API reads from the **debug snapshot** (a composed view) rather than each worker’s raw channel, so that “what clients see” has a single owned projection path.

### `src/test_harness/` and `tests/`

Owns:

- real-binary test orchestration (namespaces, port leasing, etcd/postgres process control)
- fault injection primitives (TCP proxy)
- black-box and BDD-style tests that assert external behavior, not internal details
- focused HA integration entrypoints in `tests/ha_multi_node_*.rs` and `tests/ha_partition_*.rs`, with shared scenario support under `tests/ha/support/`.

The harness is part of correctness: HA logic is not “proven” until it survives real process timing and coordination.

## Startup vs steady-state: where behavior lives

A common pitfall when changing this system is accidentally mixing:

- **startup-only behavior** (bootstrap/clone/resume decisions), and
- **steady-state HA behavior** (ongoing leader election, follow/promote/demote, fencing, rewind).

In the current implementation:

- startup planning happens in `runtime/node.rs` via:
  - `inspect_data_dir` (Missing / Empty / Existing)
  - `probe_dcs_cache` (connect etcd, drain watch events, build a snapshot cache)
  - `select_startup_mode` (InitializePrimary / CloneReplica / ResumeExisting)
- startup execution uses process jobs (`ProcessJobKind::Bootstrap`, `BaseBackup`, `StartPostgres`) before workers are started.
- steady-state behavior happens after `run_workers` starts all worker loops.

If you add behavior that must happen before any API is served (for example, ensuring directories exist, seeding DCS init records), it belongs in startup.

If you add behavior that must react continuously to changes (DCS leader record changes, Postgres reachability changes), it belongs in a worker loop.

## Adjacent subsystem connections

This chapter describes “where things live”. The next step is learning “how they are wired” and “how decisions become side effects”:

- Read [Worker Wiring and State Flow](./worker-wiring.md) to understand `StatePublisher`/`StateSubscriber` ownership and the steady-state feedback loop.
- Read [HA Decision and Action Pipeline](./ha-pipeline.md) to understand how `ha::decide` and `ha::worker::dispatch_actions` compose.
- Read [API and Debug Contracts](./api-debug-contracts.md) to understand the intent write path (`/switchover`) and the debug snapshot projection model.
- Read [Testing System Deep Dive](./testing-system.md) to learn which tests protect which boundary, and how to extend coverage safely.

## Failure behavior

Most “things went wrong” paths in this codebase are surfaced through *typed state* and worker status, not implicit panics:

- startup planning returns an error early if prerequisites cannot be satisfied (missing config, unreadable paths, unreachable DCS during probe)
- steady-state workers publish faulted status when a boundary fails (for example DCS connectivity loss, or process job failure)
- HA decisions are conservative when inputs are missing or unhealthy: lack of trusted DCS state or lack of Postgres reachability tends to suppress promotion and prefer fail-safe waiting phases.

When debugging a failure, start from the boundary that failed (API, DCS, pginfo, process) and trace what state it published.

## Tradeoffs / sharp edges

This project chooses explicit ownership and a small number of side-effect boundaries over “everything can call everything” convenience.

Sharp edges to watch for when making changes:

- don’t sneak business logic into `runtime/` orchestration; keep decisions in `ha/decide` and projections in controllers/views
- don’t add new DCS write paths casually; coordination writes are the highest-risk surface
- avoid “helpful” direct probing inside HA or API handlers; that creates split-brain between “what the system believes” and “what the handler did ad hoc”
- when adding new state, decide who owns it and how it is versioned before wiring it to multiple consumers.

## Evidence pointers

If you want to quickly validate the mental model in code, these are the best starting points:

- `src/runtime/node.rs`: startup planning + worker wiring
- `src/state/`: watch-based “latest state” semantics
- `src/ha/decide.rs` and `src/ha/worker.rs`: pure decisions + dispatch boundary
- `src/dcs/worker.rs` and `src/dcs/etcd_store.rs`: watch cache + reconnect semantics
- `src/process/worker.rs`: subprocess boundary, timeouts, and error surfacing
- `tests/bdd_api_http.rs` and `tests/ha_multi_node_*.rs`: external-interface behavior and real-process e2e coverage
