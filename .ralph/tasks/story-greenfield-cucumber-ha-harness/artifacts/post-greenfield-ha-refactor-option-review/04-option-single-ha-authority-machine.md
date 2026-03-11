# Option C: Single HA authority machine

## Design goal

Collapse startup, steady-state leadership, recovery, and rejoin into one larger HA authority machine. This is the structural option.

Instead of having:

- startup planner in `src/runtime/node.rs`
- steady-state decision logic in `src/ha/decision.rs` and `src/ha/decide.rs`
- worker-local dedup in `src/ha/worker.rs`
- rejoin visibility rules spread across recovery and observer behavior

this option moves all authority-bearing lifecycle decisions into one HA state machine and treats startup as just the earliest HA phases.

## Expected behavior changes

- startup is no longer a separate one-shot authority path
- leadership, fail-safe, recovery, and rejoin all advance through one explicit state machine
- phase transitions carry effect identity and completion state, so dedup becomes unnecessary
- operator-visible state derives from one lifecycle model instead of multiple partial models

## Concrete code areas likely to change

- [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)
- [src/ha/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/state.rs)
- [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs)
- [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs)
- [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs)
- [src/ha/lower.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/lower.rs)
- [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs)
- adjacent process / recovery modules

## How this option fixes the full failure set as one design

Quorum and majority:

- A single machine owns configured-membership majority rules from bootstrap onward.
- No separate startup and steady-state interpretations remain.

Trust versus election:

- Trust degradation becomes phase transitions inside the machine rather than a coarse external gate.
- The machine can represent:
  - can write
  - can follow
  - can recover
  - must stop
  - waiting for stronger authority evidence

Deterministic durability ranking:

- Candidate ranking becomes a machine-owned transition guard used for automatic failover and requested switchover.

Startup alignment:

- Full alignment because startup is no longer separate.

Uncertainty modeling:

- Observer/API, DCS, and SQL uncertainty become typed machine facts that determine which transitions are legal.

Dedup removal:

- Dedup disappears naturally because every phase/action transition carries its own in-flight action state.
- Repeated ticks simply re-observe the same in-flight phase instead of skipping work through a worker-local branch.

Recovery and rejoin:

- Recovery becomes first-class machine behavior.
- Rejoin is only published when the machine advances into an integrated-replica phase with explicit readiness proofs.

## Likely scenario coverage

This option covers the entire failure set and leaves the fewest architectural leftovers.

## Tradeoffs

- Cleanest end-state
- Highest implementation cost
- Highest short-term risk
- Largest test churn

## Risks

- Easy to overbuild if the authority-state vocabulary is not carefully scoped
- Harder to stage incrementally than the other options
- Greater chance of destabilizing unrelated behavior during the migration

## Migration size

Large.

## Why it stays compatible with the current functional style

- still compatible with `ha_loop`
- still compatible with pure decision functions
- the machine can be implemented as immutable state transitions rather than mutable controllers
- no unwrap / panic / expect shortcuts are needed
- avoids edge-case spray by absorbing lifecycle differences into a single typed phase model
