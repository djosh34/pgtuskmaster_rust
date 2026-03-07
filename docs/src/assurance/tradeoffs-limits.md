# Tradeoffs and Limits

Every HA design pays for its safety properties somewhere. This system is explicit about paying with caution, configuration discipline, and sometimes slower progress under ambiguity. That does not make it weak. It makes its operating model easier to evaluate honestly.

## Primary tradeoffs

### Safety over immediate availability

The clearest tradeoff is that the runtime prefers not to promote when the evidence is weak. Under ambiguous coordination conditions, an operator may experience that as frustrating delay. The alternative would be faster action with a higher chance of split-brain or bad rejoin decisions. The design chooses the delay.

### Explicit configuration over permissive defaults

The runtime and docs favor explicit runtime config, explicit secret handling, explicit API security choices, and explicit path wiring. That makes early setup more demanding, but it reduces the hidden behavior that often makes recovery harder later. A deployment that fails closed on missing critical config is inconvenient during setup and safer during crisis.

### Reconciliation discipline over one-shot imperative control

Planned switchover, failover, recovery, and startup all run through the same ongoing decision discipline rather than through special imperative shortcuts. This makes the system more coherent, but it also means operators must observe and interpret the lifecycle rather than assuming a single accepted request already completed the operation.

## Practical limits

Several limits are structural rather than accidental:

- coordination quality still depends on etcd health and correct scope usage
- recovery success depends on prerequisites such as binary wiring, auth, source reachability, and timeout budgets
- a node can only be as decisive as the evidence it can currently observe from PostgreSQL and the DCS
- the public API is intentionally small, which keeps the contract tight but also means the richer explanation layer still lives in lifecycle, logs, and debug views

These are not defects in the same sense as a broken route or a panic. They are boundaries of what the architecture can honestly promise.

## Scenario-oriented caveats

### No-quorum or weak-trust incidents

Expect conservative behavior and delayed promotion. That is the design expressing uncertainty honestly. The right operator response is usually to restore coordination quality first rather than demand faster failover from degraded evidence.

### Switchover during a messy cluster state

Expect the difference between accepted request and completed transition to matter. A queued switchover request can be correct while the cluster still waits for a safe successor or stronger evidence.

### Recovery after divergence

Expect rejoin to take more work than "start PostgreSQL again." Recovery prerequisites, source availability, and history compatibility all affect whether rewind, base backup, or bootstrap is even possible.

## Operational interpretation

A conservative decision is not automatically a bug. In many cases it is the intended safe behavior under the current evidence quality. The more useful question is usually not "why didn't it promote faster?" but "which assumption needed for safe promotion was still false?"

That question connects this chapter back to the rest of the book:

- use [System Lifecycle](../lifecycle/index.md) to understand which phase and decision are active
- use [Troubleshooting by Symptom](../operator/troubleshooting.md) to map the observed symptom to concrete checks
- use [Safety Invariants](./safety-invariants.md) when you need to decide whether the behavior is protective or suspect
