# How HA turns shared state into actions

The High Availability (HA) worker transforms cluster state into concrete actions through a four-stage pipeline: **observe, decide, lower, and apply**. Each stage serves a distinct purpose in keeping Postgres highly available while making the system easier to reason about, test, and evolve.

## The pipeline stages

### Observe: build a world snapshot

At the start of every HA step, the worker collects inputs from four subscribers—`pginfo`, DCS, process, and config—plus a periodic poll interval. This snapshot freezes a consistent view of the cluster: local Postgres health, peer state from the DCS, in-flight process actions, and current configuration. By snapshotting first, the decision logic never races against mid-step updates and engineers can inspect exactly what data drove a particular decision.

### Decide: select an intent

With the world snapshot in hand, the *decide* layer selects the next HA phase and decision. A global trust override gates all logic, then phase-specific rules evaluate the frozen snapshot to produce a chosen transition. This step is **pure**: it reads state and outputs intent without side effects. Keeping decision separate from execution makes unit tests fast (no subprocesses, no network calls) and ensures the reasoning is reproducible for a given snapshot.

### Lower: translate intent into effects

Decision outputs are high-level intent (“I want to be leader” or “I need to wait for sync”). The *lowering* pipeline maps that intent onto a concrete *effect plan*:

- Postgres actions (restart, reload, promote)
- DCS lease changes (acquire, release, update LSN)
- Switchover cleanup and replication configuration
- Safety actions (pause/resume, fencing)

Separating lowering from deciding lets the same decision drive multiple independent effect buckets. It also localizes knowledge of how to execute a particular intent, so adding a new Postgres operation does not ripple into the core state machine.

### Apply: dispatch and monitor

Finally, the worker applies the effect plan. Application is **not transactional**: if a Postgres restart succeeds but a lease update fails, the earlier effect remains applied. The worker publishes the *new HA state* before dispatch finishes, then attempts the effects. If any dispatch returns an error, the worker republishes the same phase and decision but marks the worker status as **faulted**, signaling that intent and reality have diverged.

## Tradeoffs of publishing intent first

Publishing the chosen state before all side effects complete has two consequences:

1. **Observability**: external tools see the intended role immediately, even if Postgres is still restarting. This makes failover progress visible rather than hidden behind a black box.
2. **Failure handling**: because effects are not rolled back, a partial failure leaves the system in a known but possibly inconsistent state. The faulted flag alerts operators that manual reconciliation may be needed.

The alternative—making the entire step atomic—would require a distributed transaction across Postgres, DCS, and local processes. That would couple the components more tightly and hide partial progress, making automation and debugging harder.

## Redundant dispatch skipping

Some decisions (e.g., waiting for sync, observing steady state) would repeatedly trigger the same process action on every poll. The worker explicitly skips redundant dispatch when the phase and decision are unchanged and the chosen state is on a waiting or recovery path. This avoids CPU churn and spurious log noise while preserving the clear “intent-first” semantics.

## What this means for debugging and evolution

When a failover appears stuck, the four stages give you clear checkpoints:

1. **Snapshot** – inspect the `world` struct to confirm inputs are as expected.
2. **Decision** – trace the `decide` function to see why the state machine chose its path.
3. **Lowering** – verify the effect plan matches the intent.
4. **Apply** – check process logs and the `faulted` flag to see which effect failed.

Evolution also benefits: new HA strategies can be prototyped by altering only the *decide* logic, while new Postgres operations can be added in *apply* without touching the state machine core. The separation of concerns keeps the system modular and testable at each boundary.

---
