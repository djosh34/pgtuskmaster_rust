# HA Decision Engine

The HA loop is now organized around four steps:

1. observe the local and global world
2. decide the desired local role and operator-facing authority
3. reconcile that desired state into ordered actions
4. execute those actions

This replaced the older public story about a separately exposed phase machine plus semantic decision enum. Internally, the engine is now easier to reason about because the important outputs are explicit values instead of being split across several layers.

## The Two Core Outputs

The pure decision step produces a desired state with three parts:

- `role`: what this node should try to be locally
- `publication`: what authority this node should publish for operators
- `clear_switchover`: whether the switchover request should be removed

Those first two outputs answer different questions:

- local role answers "what should this node do?"
- publication answers "who should operators currently trust as primary?"

That distinction is why the runtime can say "follow node-a" locally while still publishing `no_primary` during lease-open or recovery windows.

## Trust Still Gates Everything

The first branch is still the trust gate.

When DCS trust is not `FullQuorum`, normal leadership logic is bypassed:

- a local primary moves into a fail-safe role that may carry a fence cutoff
- a local replica keeps following or waits for quorum
- authority publication moves away from `primary` and toward `no_primary`

Safety wins over availability whenever the node cannot trust the coordination layer.

## Role Taxonomy

The public local-role contract is the `ha_role` field exposed by `/ha/state`:

- `leader`
- `candidate`
- `follower`
- `fail_safe`
- `demoting_for_switchover`
- `fenced`
- `idle`

These roles are enough to cover startup, failover, switchover, degraded-trust handling, and replica recovery without a second public phase enum.

## Authority Taxonomy

The public authority contract is the `authority` field exposed by `/ha/state`:

- `primary`
- `no_primary`
- `unknown`

`primary` carries a lease epoch with `{ holder, generation }`. That generation matters because leadership is term-based: the same member can regain leadership later, but it should not be confused with an older lease.

`no_primary` carries a structured reason such as:

- `dcs_degraded`
- `lease_open`
- `recovering`
- `switchover_rejected`

## Reconcile Turns Intent Into Ordered Work

The pure reconcile step converts the desired state into an ordered action list such as:

- publish authority
- acquire or release the leader lease
- start as primary or replica
- promote
- demote
- rewind or base backup
- clear the switchover request

This is simpler than the old lowering pipeline because the worker consumes one ordered list instead of merging several effect buckets at execution time.

## Why This Shape Matters

This design gives the runtime three practical benefits:

- the stable API now exposes the same concepts operators actually need: authority, local role, and next actions
- the worker is thinner because it no longer owns a separate semantic-to-effect dedup layer
- lease epochs and fence cutoffs are first-class values instead of implied side effects

For operators, the result is a more truthful contract: `/ha/state` tells you both who currently has authority and what the local node intends to do next.
