# Option B: Unified authority snapshot and ranker

## Design goal

Build one explicit authority snapshot model that every HA decision consumes. This option is larger than Option A and is the recommended path.

Instead of only sharing predicates, define a first-class immutable snapshot that includes:

- configured membership and majority requirement
- authority state for self and peers
- ranked leader candidates on durability plus readiness
- startup-resume intent facts
- recovery and rejoin integration state
- in-flight effect identity for safe repeated decisions

This option keeps `ha_loop` and pure `decide()`, but gives them a better domain object to reason over.

## Expected behavior changes

- the HA loop stops reasoning directly from raw DCS cache plus process state and instead reasons from a normalized authority snapshot
- startup mode selection becomes a pure function over the same snapshot family
- election and switchover target selection both use one deterministic ranking
- uncertainty becomes explicit and typed, which separates “cannot confirm leadership yet” from “must enter fail-safe”
- rejoin and clone recovery only become operator-visible when the snapshot marks the node as integrated and queryable
- worker dedup disappears because the snapshot includes explicit action-progress identity

## Concrete code areas likely to change

- [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs)
- [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs)
- [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs)
- [src/ha/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/state.rs)
- [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs)
- [src/ha/lower.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/lower.rs)
- [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs)
- [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)
- likely adjacent process-state / recovery-state modules

## How this option fixes the full failure set as one design

Quorum and majority:

- Add configured membership to the snapshot and compute majority from that configured set.
- Mark self and peers as `Authoritative`, `Followable`, `Recovering`, `ObservedButIneligible`, or `UnsafeUnknown` instead of relying on one trust enum and ad hoc checks.

Trust versus election:

- Split “can this node write” from “can this node continue learning and converging.”
- Allow the system to remain non-authoritative without discarding all peer / recovery orientation data.
- Keep `FailSafe` as a terminal write-authority withdrawal mode, not the generic home for every trust reduction.

Deterministic durability-driven leader ranking:

- Build a single ranking over candidates using:
  - timeline
  - `write_lsn` / `replay_lsn`
  - freshness
  - readiness
  - role compatibility
  - explicit switchover target constraints when present
- Make that ranker the only path to `AttemptLeadership`, `BecomePrimary`, and switchover acceptance.

Startup alignment:

- Startup uses the same authority snapshot family to choose `InitializePrimary`, `CloneReplica`, or `ResumeExisting`.
- Remove relaxed leader selection and default-primary fallback from [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs).
- Make startup intent a projection of the same model that governs steady-state authority.

Uncertainty modeling:

- Model DCS partition, observer/API partition, and SQL readiness uncertainty separately.
- Prevent DCS-cut primaries from staying authoritative while still letting healthy peers orient themselves and converge after heal.

Dedup removal:

- Remove the worker-local `ActiveJobKind` shortcut.
- Add explicit in-flight effect identity to the snapshot or adjacent HA state, so repeated ticks can recognize “same effect already in progress” through authoritative state rather than a bypass.

Recovery and rejoin:

- Represent recovery stages explicitly in the snapshot, so “rejoined as replica” means:
  - clone / rewind action finished
  - postgres is reachable
  - readiness is `Ready`
  - proof/query checks are satisfiable
  - the node is safely integrated behind the ranked leader

## Likely scenario coverage

This option cleanly covers every preserved failure class because it attacks the shared authority model directly.

- quorum-loss visibility and explicit fail-safe
- majority-side election
- lagging / isolated candidate rejection
- mixed DCS/API fault handling
- stale leader lease after repeated failover
- storage-stall replacement
- switchover demotion and old-primary convergence
- clone / broken-rejoin premature success reporting
- repeated-failover write survival

## Tradeoffs

- Strongest conceptual cleanliness
- Best chance of preventing future bug clusters instead of just fixing the current ones
- More refactor work than Option A
- Requires deliberate redesign of the HA state domain model, not just helper extraction

## Risks

- Bigger migration surface across HA state, apply/lower, and runtime startup
- Requires careful test updates to keep the model legible and not over-abstract

## Migration size

Medium-large.

## Why it stays compatible with the current functional style

- preserves `ha_loop`
- preserves pure `decide()` over immutable inputs
- converts implicit side paths into typed data, which is a functional improvement rather than a control-flow rewrite
- removes the need for `mut`-heavy orchestration by making authority, ranking, and progress facts explicit
- avoids unwrap / panic / expect design shortcuts by construction
- reduces edge-case spray because new cases become data states, not scattered branches
