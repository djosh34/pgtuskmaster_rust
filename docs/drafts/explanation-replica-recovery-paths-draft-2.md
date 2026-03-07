# Draft: About Replica Recovery Paths

Compass classification: `cognition + acquisition` because this page is intended to help the reader reason about recovery strategy boundaries, not to tell them how to trigger recovery.

The recovery logic in `pgtuskmaster` is easiest to understand as a scale of trust in local state. The runtime does not have one generic recovery action because it does not make one generic assumption about what remains valid after a failure.

## Four different responses to different failures

The HA state machine and the lowering layer divide recovery-adjacent behavior into four categories:

- `rewind` when local state may still be salvageable against another leader
- `base backup` when a healthy leader can provide a fresh replica copy
- `bootstrap` when the node must create fresh local state without cloning from a leader
- `fencing` when the priority is to stop unsafe participation rather than preserve continuity

These are not just different commands. They express different judgments about whether local history can be kept, replaced, created anew, or must be stopped before anything else.

## Rewind is the least destructive repair

Rewind depends on a specific leader member and is chosen only when HA believes that leader is usable and `rewind_required` is true. That places rewind close to ordinary follow-leader behavior: the node is still orienting itself around another primary, but it has diverged enough that a simple follow configuration is not sufficient.

## Base backup and bootstrap cross a stronger boundary

Both base backup and bootstrap are lowered into a wipe followed by a long-running job. The effect plan makes that visible by counting them as two-step recoveries. This is an important design clue: these paths are not repairs of local state but replacements of it.

The difference between them is where the replacement state comes from. Base backup has an external source leader. Bootstrap does not.

## Fencing says "stop trusting continuity"

Fencing appears when the runtime needs to stop a dangerous situation, such as a conflicting foreign leader or a safety-driven shutdown. It belongs next to recovery in the architecture because it handles cases where any attempt to continue as-is would make later recovery less trustworthy.

## Why the split matters

If these paths were collapsed into one "recover replica" concept, the system would hide crucial distinctions:

- whether a source leader is required
- whether the local data directory is preserved or wiped
- whether the goal is to repair role alignment or to stop unsafe participation

The code keeps them separate so the HA decision, the lowered effect plan, and the process worker all remain explicit about what kind of trust in local state still exists.

See also:

- [HA Reference](../src/reference/ha.md)
