# About Replica Recovery Paths

Replica recovery in `pgtuskmaster` is deliberately not a single "fix replica" action. The HA subsystem distinguishes rewind, base backup, bootstrap, and fencing because each path answers a different kind of failure or divergence.

## Recovery is chosen from the world state

The HA decision layer derives facts from PostgreSQL state, DCS state, and process state. It asks whether PostgreSQL is reachable, whether another usable primary exists, whether rewind is required, and whether previous recovery work succeeded or failed.

This means recovery is not modeled as a static preference order. It is modeled as a response to evidence about the local data directory, the available leader, and the last process outcome.

## Why rewind exists as its own path

Rewind is used when the node still has a usable data directory shape but must converge back to another primary's history. In the decision state machine, rewind is chosen when a replica sees an active leader and `rewind_required` is true.

That makes rewind the most conservative recovery path. It assumes the local node is close enough to a valid replica state that it should be repaired rather than discarded.

## Why base backup is different

Base backup is chosen when the node needs a full replica copy from a specific leader. In the lowered effect plan, base backup is not just one action: it wipes the data directory first and then starts the base backup job. The two-step lowering makes the destructive boundary explicit.

Base backup therefore represents a stronger claim than rewind. The runtime is no longer preserving local replica state; it is replacing it from another member.

## Why bootstrap is different again

Bootstrap is the recovery path with no external source member. It appears when the runtime has to create a fresh primary data directory rather than copy from a leader. Like base backup, bootstrap is lowered as wipe-then-run, which marks it as a full replacement of local state rather than a repair of existing state.

## Why fencing is adjacent to recovery

Fencing is not replica recovery, but it sits next to these paths because it handles the unsafe cases that recovery should not paper over. When HA sees a conflicting foreign leader while local PostgreSQL is primary, or when safety requires an immediate stop, it uses fencing-oriented actions instead of trying to recover in place.

The important distinction is that rewind, base backup, and bootstrap are attempts to restore a coherent replica or primary role, while fencing is the system admitting that safe continuation requires first stopping or isolating the local node.

## The design implication

Splitting recovery into separate paths keeps the HA state machine honest about how much local state it is willing to trust:

- rewind preserves and repairs
- base backup replaces from a known leader
- bootstrap creates fresh cluster state
- fencing protects the cluster when role continuity is unsafe

That separation is why the effect plan and process dispatch layers remain explicit about which jobs run, which ones require a leader source, and which ones begin by wiping the local data directory.

See also:

- [HA Reference](../reference/ha.md)
