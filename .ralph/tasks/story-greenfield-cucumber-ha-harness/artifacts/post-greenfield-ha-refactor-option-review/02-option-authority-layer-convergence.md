# Option A: Authority layer convergence

## Design goal

Make the smallest refactor that still replaces the current authority shortcuts with one shared authority layer used by DCS trust, steady-state HA decisions, switchover validation, and startup planning.

This is the conservative option. It keeps the current `ha_loop`, keeps `decide()` as a pure function, and preserves the existing broad phase structure. The main change is extracting and centralizing the predicates that currently drift across `src/dcs/state.rs`, `src/ha/decision.rs`, `src/ha/decide.rs`, and `src/runtime/node.rs`.

## Expected behavior changes

- quorum becomes majority-of-configured-membership instead of observed-member-count shortcut logic
- trust is split into at least:
  - authoritative majority available
  - degraded but still followable or recoverable
  - must stop / fail-safe
- leader and switchover target eligibility become a single ruleset shared by failover and operator-triggered switchover
- startup no longer assumes primary purely because DCS was unavailable or incomplete
- worker dedup disappears because in-flight effect identity is modeled explicitly in state/apply rather than bypassed in `src/ha/worker.rs`

## Concrete code areas likely to change

- [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs)
- [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs)
- [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs)
- [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs)
- [src/ha/lower.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/lower.rs)
- [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs)
- [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)

## How this option fixes the full failure set as one design

Quorum and trust:

- Introduce configured membership cardinality into the authority layer and compute majority from configuration, not observed cache size.
- Replace the binary `FullQuorum` versus everything-else collapse with richer authority facts used by `decide()`.
- Treat quorum loss, stale leader state, missing observer visibility, and partial rejoin evidence as separate facts instead of flattening them into the same response.

Deterministic durability-based leader ranking:

- Add a shared candidate-ranking function over fresh members on the best known timeline.
- Use `write_lsn` and `replay_lsn` as part of eligibility, not just freshness and readiness.
- Reuse the same ranking for failover and targeted switchover, so isolated or lagging replicas are rejected consistently.

Startup and steady-state alignment:

- Replace startup helper-specific leader checks in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) with calls into the same authority layer used by the HA loop.
- Remove the “DCS unavailable therefore primary by default” path for cases where role authority cannot be reconstructed safely.

Uncertainty without blunt fail-safe collapse:

- Introduce an “uncertain but non-authoritative” state that allows followers and recovery to keep orienting themselves without granting write authority.
- Reserve explicit `FailSafe` for cases where the node must hard-stop authority and cannot safely follow.

Dedup removal:

- Delete the dedup path in [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs).
- Move repeated-action suppression into explicit effect identity and process-progress facts in the effect-plan / apply layer.
- Keep the state loop authoritative by making repeated identical decisions safe rather than silently skipping effect application.

Recovery and rejoin:

- Require the same authority layer to determine when a rejoined node becomes eligible to be counted as online, followable, or switchover-ready.
- Tie “rejoined” and “healthy replica” reporting to genuine readiness and queryability, not merely a restart or DCS record update.

## Likely scenario coverage

This option addresses:

- no-quorum withdrawal and explicit fail-safe reporting
- majority partition election
- lagging or isolated replica promotion / switchover rejection
- mixed DCS/API uncertainty handling
- storage-stall replacement logic
- old-primary post-switchover convergence
- clone / broken-rejoin premature success reporting
- repeated-failover stale-lease and write-safety classes

## Tradeoffs

- Lowest migration cost of the three options
- Easiest to phase incrementally
- Highest risk of leaving some conceptual duplication behind because the existing HA phases and startup planner stay largely intact

## Risks

- The authority layer can become a “shared predicate pile” if it is not given a strong domain model
- If uncertainty states are only partially adopted, some paths may still degrade into ambiguous `WaitForDcsTrust` behavior

## Migration size

Medium-small.

## Why it stays compatible with the current functional style

- `decide()` remains pure
- `ha_loop` remains the orchestration center
- most new structure lives in immutable authority facts and pure ranking helpers
- no `mut`-heavy controller rewrite is needed
- no unwrap / panic / expect shortcuts are part of the design
- edge-case spray is explicitly avoided by centralizing the rules in one authority layer
