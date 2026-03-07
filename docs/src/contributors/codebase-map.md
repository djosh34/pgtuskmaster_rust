# Codebase Map

This project is structured around runtime responsibilities. The modules are not a generic controller-service-repository stack; they are closer to a set of workers with explicit publishers, subscribers, and a small number of side-effect boundaries.

If you are new to the codebase, the fastest way to build a correct mental model is:

1. understand where startup happens and where steady-state begins
2. understand which worker owns which piece of state
3. understand which components are allowed to perform side effects

## The top-level runtime shape

At a high level, the node runtime does three things:

1. load and validate config
2. plan and execute startup (bootstrap, clone, or resume)
3. run steady-state workers concurrently (`pginfo`, `dcs`, `process`, `ha`, `debug_api`, and `api`)

The canonical entrypoint is `src/runtime/node.rs`:

- `run_node_from_config_path(...)` -> `run_node_from_config(...)`
- `plan_startup(...)` and `plan_startup_with_probe(...)`
- `execute_startup(...)`
- `run_workers(...)`

The CLI binary in `src/bin/pgtuskmaster.rs` is intentionally thin. If you are tracing real runtime behavior, open `src/runtime/node.rs` first.

## Startup ends before steady-state workers exist

The most important ownership boundary in the whole codebase is the split between startup and steady-state.

Startup in `src/runtime/node.rs` owns:

- data-dir inspection (`inspect_data_dir(...)`)
- the one-shot DCS probe (`probe_dcs_cache(...)`)
- startup-mode selection (`select_startup_mode(...)`)
- init-lock claiming and config seeding
- the initial bootstrap, basebackup, or start-postgres jobs

Steady-state begins only after `run_workers(...)` creates the shared channels and starts the long-lived worker loops.

If a behavior must happen before the API listener comes up or before HA begins ticking, it belongs in startup. If it must react continuously to changing evidence, it belongs in a worker.

## Module ownership map

### `src/runtime/`

Owns orchestration:

- startup planning and execution
- shared state-channel creation via `state::new_state_channel(...)`
- worker-context construction
- API listener binding and TLS setup

This module should coordinate; it should not quietly absorb HA or DCS business logic.

### `src/state/`

Owns the latest-snapshot transport and version semantics:

- `StatePublisher<T>` and `StateSubscriber<T>`
- `Versioned<T>`, `Version`, and `UnixMillis`
- shared status and worker-error types

If you are debating what "latest" means in this codebase, the answer belongs here.

### `src/pginfo/`

Owns local Postgres observation:

- SQL reachability and readiness
- role and timeline evidence
- the typed `PgInfoState` that other workers consume

If another subsystem needs local Postgres truth, it should read the published pginfo state instead of probing the database ad hoc.

### `src/dcs/`

Owns the distributed coordination read model and DCS trust:

- watch-event decoding
- the `DcsCache`
- member, leader, switchover, config, and init-lock records
- `DcsTrust`
- publishing this node's local member record

One concrete runtime detail matters here: `run_workers(...)` creates separate `EtcdDcsStore` handles for the DCS worker, HA worker, and API worker. That keeps each boundary explicit instead of sharing one mutable store object across the whole runtime.

### `src/ha/`

Owns lifecycle control:

- the HA phase machine (`HaPhase`, `HaState`)
- pure decision logic (`decide.rs`, `decision.rs`)
- lowering decisions into a bucketed plan (`lower.rs`)
- applying DCS and process side effects (`apply.rs`, `process_dispatch.rs`)

The design contract is deliberate:

- decision logic should stay pure and testable
- effect application is a failure-prone boundary and should stay explicit

### `src/process/`

Owns local side effects against the host:

- `pg_ctl`, `pg_rewind`, `pg_basebackup`, `initdb`, and related subprocess work
- the process job state machine
- timeout and output handling

If a change touches the local machine or Postgres data directory, it probably belongs here rather than in HA orchestration.

### `src/api/` and `src/debug_api/`

Own the external and projected read surfaces:

- request parsing, auth, and routing in `src/api/worker.rs`
- small stable read models in `src/api/controller.rs`
- debug snapshot building in `src/debug_api/worker.rs` and `src/debug_api/snapshot.rs`
- verbose debug projection in `src/debug_api/view.rs`

The API reads from the composed debug snapshot instead of stitching together raw worker channels in every handler. That single projection path is part of the contract.

### `src/test_harness/` and `tests/`

Own executable proof:

- namespace, port, etcd, Postgres, and proxy fixtures under `src/test_harness/`
- black-box HTTP tests in `tests/`
- focused HA real-binary scenarios in `tests/ha_*`

The harness is not optional scaffolding. For HA changes, it is part of the correctness story.

## How to find code for common tasks

- Startup behavior looks wrong: start in `src/runtime/node.rs`
- State ownership or version semantics are unclear: start in `src/state/watch_state.rs`
- HA chose an unexpected phase: start in `src/ha/worker.rs`, then `src/ha/decide.rs`
- A switchover or API response looks wrong: start in `src/api/worker.rs` and `src/api/controller.rs`
- A real-binary HA scenario is failing: start in `src/test_harness/ha_e2e/startup.rs` and the matching `tests/ha_*` entrypoint

## Failure behavior

Most failures in this codebase are surfaced through typed state and worker status rather than hidden behind panics:

- startup returns errors early when prerequisites are not satisfied
- steady-state workers publish faulted status when a boundary fails
- HA stays conservative when trust or Postgres reachability is weak

When debugging, start from the boundary that failed and follow the published state forward rather than jumping straight to the biggest module.

## Tradeoffs and sharp edges

- Do not sneak business logic into `runtime/`; keep orchestration separate from decision logic.
- Do not add new DCS write paths casually; coordination writes are one of the highest-risk surfaces in the repo.
- Do not let HA or API perform direct probes that bypass shared state; that creates multiple truths.
- When adding new shared state, decide who owns it and how it is versioned before wiring new consumers.

## Adjacent subsystem connections

- Read [Worker Wiring and State Flow](./worker-wiring.md) to understand publisher/subscriber ownership and the feedback loop.
- Read [HA Decision and Effect-Plan Pipeline](./ha-pipeline.md) to understand how decisions become applied effects.
- Read [API and Debug Contracts](./api-debug-contracts.md) to understand the external edges and projections.
- Read [Testing System Deep Dive](./testing-system.md) to understand which tests protect each boundary.

## Evidence pointers

- `src/runtime/node.rs`
- `src/state/watch_state.rs`
- `src/dcs/worker.rs`
- `src/ha/worker.rs`
- `src/api/worker.rs`
- `src/test_harness/ha_e2e/startup.rs`
