# Worker Wiring and State Flow

The runtime is a set of long-lived workers wired together with typed latest-snapshot channels.

The central idea is simple:

> observation feeds decision, decision feeds action, and action outcomes feed the next decision

This chapter explains the actual wiring: which worker owns which state, how updates propagate, and what "latest" means in the code rather than in architecture diagrams.

## The state-channel contract

Workers communicate by publishing a `Versioned<T>` snapshot into a `tokio::sync::watch` channel.

In `src/state/watch_state.rs`, `new_state_channel(initial, now)` creates:

- one `StatePublisher<T>` owned by exactly one writer
- one `StateSubscriber<T>` that can be cloned freely for readers

Publishing:

- increments the monotonic `Version`
- stamps `updated_at`
- replaces the channel value atomically

Consuming:

- `latest()` returns the current snapshot immediately
- `changed().await` waits for a new version, then lets the consumer read the latest value

This is a latest-value model, not an event log. Missing intermediate versions is acceptable because the contract is "what does the system believe right now?" rather than "show me every micro-transition".

## Where wiring happens

All steady-state wiring is centralized in `src/runtime/node.rs::run_workers(...)`.

That function:

- creates the shared channels for config, pginfo, dcs, process, ha, and debug snapshot state
- builds worker contexts from publishers, subscribers, and side-effect handles
- creates three separate `EtcdDcsStore` handles for DCS watch/publication, HA writes, and API intent writes
- starts the worker loops with `tokio::try_join!`

The runtime graph is:

```text
pginfo ----\
            \
             -> dcs ----\
                          \
process ------------------> ha ----> process
                             \
config -----------------------\------> debug snapshot ---> api
```

That graph is small on purpose. If a new edge feels convenient but hard to explain, it is probably a design smell.

## Ownership rule

The most important contributor rule is:

> if you need to change how some shared state is computed, change the worker that publishes it

Do not rebuild shared state ad hoc inside consumers. That creates multiple truths and makes HA, API, and tests disagree about the same world.

## Worker-by-worker ownership

### `pginfo`

Owns `StatePublisher<PgInfoState>`.

It observes local Postgres, classifies reachability and readiness, and publishes a typed snapshot for everyone else. If HA or API needs local Postgres truth, it should flow through pginfo first.

### `dcs`

Owns `StatePublisher<DcsState>`.

It reads watch events, maintains `DcsCache`, evaluates `DcsTrust`, and writes the local member record derived from pginfo. If the store becomes unhealthy or refresh logic fails, the published trust becomes `NotTrusted`.

### `process`

Owns `StatePublisher<ProcessState>`.

It receives job requests over an unbounded inbox and reports whether a relevant action is running, succeeded, failed, or timed out. The process worker is intentionally boring; it should execute jobs, not decide policy.

### `ha`

Owns `StatePublisher<HaState>`.

It reads config, pginfo, dcs, and process snapshots; selects the next phase and decision; publishes the next state; and then applies the lowered effect plan. If dispatch fails, it republishes the same phase as `Faulted(...)` so readers can see both the selected decision and the failure.

### `debug_api`

Owns `StatePublisher<SystemSnapshot>`.

It composes the read model that the API and debug clients consume. It also keeps the bounded change and timeline history that powers `/debug/verbose`.

### `api`

Owns no shared state channel.

It is the external edge: one connection at a time, auth, routing, intent writes, and read responses from the composed snapshot.

## Wakeup strategy

Workers do not all wake up the same way:

- HA uses `tokio::select!` on upstream channel changes plus an interval tick.
- DCS and debug snapshot use poll-and-sleep loops.
- API also uses a poll-style loop, but its unit of work is a timed `accept()` plus one request/response cycle before the next sleep.

That difference matters when you are debugging apparent latency. HA reacts quickly to upstream state changes. DCS, debug, and API responsiveness are bounded partly by poll intervals.

## Structured logs are part of the wiring story

In addition to state channels, workers emit structured events through `LogHandle::emit_app_event(...)`.

Contributors should preserve:

- explicit `event.name`, `event.domain`, and `event.result`
- correlation fields such as `scope`, `member_id`, `ha_tick`, `job_id`, and `api.peer_addr`
- fail-closed behavior when an error breaks invariants

Those events are how operators and tests reconstruct a path like intent -> decision -> dispatch -> outcome without attaching a debugger.

## How to change this area safely

- Keep one publisher per shared state type.
- Prefer new subscribers over new direct probes.
- Add new cross-worker edges only when you can state the ownership contract plainly.
- Preserve the three-store split for DCS, HA, and API unless you are deliberately changing a concurrency boundary.
- Update the debug snapshot worker when a new shared state must become client-visible.

## Adjacent subsystem connections

- Read [HA Decision and Effect-Plan Pipeline](./ha-pipeline.md) to see how HA interprets pginfo, DCS, and process snapshots.
- Read [API and Debug Contracts](./api-debug-contracts.md) to see how the composed debug snapshot becomes client-facing reads.
- Read [Harness Internals](./harness-internals.md) to see how real-binary tests exercise the same wiring across multiple nodes.

## Evidence pointers

- `src/runtime/node.rs`
- `src/state/watch_state.rs`
- `src/dcs/worker.rs`
- `src/ha/worker.rs`
- `src/debug_api/worker.rs`
- `src/api/worker.rs`
