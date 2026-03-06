# Worker Wiring and State Flow

The runtime is a set of long-lived workers wired together with **typed “latest snapshot” channels**.

The central idea is:

> Observation feeds decision, decision feeds action, and action outcomes feed the next decision.

This chapter explains the actual wiring: which worker owns which state, how updates propagate, and what “latest” means in this codebase.

## The state channel model (what “latest” means)

Workers communicate by publishing a `Versioned<T>` snapshot into a `tokio::sync::watch` channel.

In `src/state/watch_state.rs`, the helper `new_state_channel(initial, now)` creates:

- a `StatePublisher<T>` (owned by exactly one worker), and
- a `StateSubscriber<T>` (cloned and passed to any worker that needs the latest snapshot).

Publishing:

- increments a strictly monotonic `Version` (+1 per publish)
- records an `updated_at` timestamp (`UnixMillis`)
- replaces the channel value atomically.

Consuming:

- `subscriber.latest()` returns the current snapshot immediately
- `subscriber.changed().await` waits for the channel to change and then returns the latest snapshot.

This is a **latest-value** model, not an event stream. If a worker publishes 10 times quickly and you are slow, you may observe only the final value — which is intentional for “current world state” modeling.

## Where channels are created and wired

The wiring is centralized in `src/runtime/node.rs` in `run_workers(...)`.

Conceptually:

```text
               (State channels: watch latest snapshots)

  +--------+       +------+       +-----+       +---------+
  | pginfo | ----> | dcs  | ----> | ha  | ----> | process |
  +--------+       +------+       +-----+       +---------+
       \               \             \              /
        \               \             \            /
         \               \             v          v
          \               +---------> debug snapshot
           \                             |
            \                            v
             +-------------------------- api
```

More concretely, `run_workers`:

- creates channels for:
  - runtime config (currently published once at startup)
  - pginfo state (`PgInfoState`)
  - dcs state (`DcsState`)
  - process state (`ProcessState`)
  - ha state (`HaState`)
  - debug snapshot (`SystemSnapshot`)
- constructs worker contexts that each include:
  - one `StatePublisher<T>` for the state that worker owns, and
  - zero or more `StateSubscriber<U>` inputs.
- starts all worker loops concurrently using `tokio::try_join!`.

## Worker responsibilities and state ownership

The key “ownership” rule is:

> If you need to change how some state is computed, change the worker that publishes that state — do not compute it ad-hoc inside consumers.

## Structured logs (operator-grade reconstruction)

In addition to state channels, workers emit structured runtime events via `LogHandle`.

Contributor expectations:

- Prefer `log.emit_event(...)` with explicit `event.name` / `event.domain` / `event.result`.
- Include correlation attributes (`scope`, `member_id`, plus subsystem ids like `ha_tick`, `job_id`, `api.peer_addr`) so operators can connect intent → dispatch → outcome across workers.
- Do not silently drop errors in hot loops:
  - if the error is ignorable and the loop continues, emit a warn event and continue,
  - if the error breaks invariants, emit an error event and return `Err` so the runtime can fail closed.

### `pginfo` worker (Postgres observation)

Owns: `StatePublisher<PgInfoState>`

Inputs: none (other than config inside its context, like DSN and interval)

Publishes:

- SQL reachability and derived status (`SqlStatus`, `Readiness`)
- basic configuration facts and WAL/replication summaries, as available.

Failure behavior:

- if it cannot query Postgres, the published `PgInfoState` should reflect unreachable/unknown, and downstream workers should degrade gracefully.

### `dcs` worker (watch cache + trust)

Owns: `StatePublisher<DcsState>`

Inputs:

- `StateSubscriber<PgInfoState>` (to publish this node’s member record derived from local Postgres state)
- a concrete store implementation (`EtcdDcsStore` behind `dyn DcsStore`).

Publishes:

- `DcsCache` (decoded view of etcd keys under the cluster scope)
- `DcsTrust` (whether it is safe to make coordination decisions right now).

Failure behavior:

- when the store is unhealthy or decoding fails, the worker marks itself faulted and publishes `DcsTrust::NotTrusted` in its output state.

### `ha` worker (decision loop + dispatch boundary)

Owns: `StatePublisher<HaState>`

Inputs:

- `StateSubscriber<RuntimeConfig>`
- `StateSubscriber<PgInfoState>`
- `StateSubscriber<DcsState>`
- `StateSubscriber<ProcessState>`
- a DCS writer handle (another etcd store instance)
- an unbounded inbox sender to the process worker.

Reads the latest snapshots, runs decision logic, and then dispatches side effects:

- coordination writes/deletes to etcd (leader lease, switchover clear)
- process job requests (start postgres, promote/demote, rewind, fencing, bootstrap).

Failure behavior:

- if dispatch fails (DCS write/delete fails, job send fails, clock fails, managed config cannot be materialized), the HA worker publishes its `WorkerStatus` as `Faulted(...)`.
- the state machine continues to tick; callers should treat faulted HA state as an error signal, not as “no decision selected”.

### `process` worker (side effects against the local host)

Owns: `StatePublisher<ProcessState>`

Inputs:

- an unbounded inbox receiver (`mpsc::unbounded_channel`) of job requests
- process config and default “dispatch” settings (paths, shutdown mode, timeout tuning).

The process worker is intentionally boring: it runs job kinds and reports outcomes. It should not contain HA decision logic.

Failure behavior:

- job failures are surfaced in `ProcessState` as outcomes, which HA consumes to choose follow-up phases (for example after a rewind attempt).

### `debug_api` worker (snapshot composition)

Owns: `StatePublisher<SystemSnapshot>`

Inputs:

- config, pginfo, dcs, process, ha subscribers.

This worker creates an owned “what the world looks like” projection and attaches:

- a monotonic `sequence`
- a limited change/timeline history used by debug clients.

The important architectural consequence: **clients don’t need to understand every internal channel**; they can rely on a composed snapshot.

### `api` worker (request routing + intent writes)

Owns: no state channel; it is a server loop.

Inputs:

- a TCP listener bound at startup
- a DCS store handle used for intent writes (for example `/switchover`)
- a `StateSubscriber<SystemSnapshot>` provided by the debug snapshot worker (used for `/ha/state` and debug views).

Failure behavior:

- `api::worker::run` intentionally keeps serving future requests even if one request/connection cycle fails.

## Update cadence and “event-driven” wakeups

Workers use different wakeup strategies:

- HA uses a `tokio::select!` that wakes on:
  - any upstream state change (`changed().await`)
  - a periodic interval tick
- DCS/debug/API run “poll + sleep” loops (short poll interval, perform one unit of work per step).

As a contributor:

- prefer reacting to upstream channel changes when the work is logically event-driven
- use periodic ticks when you need time-based retries/backoff or when upstream signals are not sufficient.

## Adjacent subsystem connections

This chapter is about wiring and state ownership. To understand the “meaning” of each state transition:

- Read [HA Decision and Action Pipeline](./ha-pipeline.md) to see how `ha::decide` interprets DCS trust, leader records, Postgres reachability, and process outcomes.
- Read [API and Debug Contracts](./api-debug-contracts.md) to see how operator intent enters the system (DCS writes) and how client-visible state is projected.
- Read [Harness Internals](./harness-internals.md) to see how real-binary tests construct the same wiring in multi-node scenarios and how they force failure conditions.
