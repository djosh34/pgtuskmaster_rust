# Safety Case

This chapter makes a cautious argument for why the architecture reduces split-brain risk and unsafe rejoin risk under the failure modes it is designed to handle. It is not a proof of correctness for all environments. It is a structured claim about what the current implementation is trying to preserve and why its design choices support that goal.

## Main claim

The system reduces the chance of concurrent primaries and unsafe rejoin by making leadership and recovery decisions depend on current local PostgreSQL evidence, current DCS evidence, explicit trust posture, and explicit lifecycle guards rather than on optimistic one-shot assumptions.

## Supporting argument

The claim is supported by several linked pieces of behavior:

- promotion is conditional, not automatic on leader absence alone
- degraded trust constrains ordinary promotion behavior through fail-safe posture
- conflicting leader evidence can trigger fencing or demotion-oriented work instead of continued primary behavior
- switchover intent enters the same decision model as automatic behavior instead of bypassing it
- recovery paths such as rewind, base backup, and bootstrap are explicit before rejoin is considered safe
- startup planning is restrictive enough to avoid inventing a new local history when coordination evidence says that would be unsafe

None of those points is sufficient alone. Together, they form a coherent bias toward "do less when the evidence is weak."

## Assumptions the claim depends on

This safety argument has explicit assumptions:

- etcd endpoints, scope, and cluster wiring are configured correctly for the intended cluster
- PostgreSQL binaries, authentication identities, and filesystem paths are provisioned correctly
- the network and clocks behave within operationally realistic bounds for the deployment
- operators use the documented API or CLI surfaces for planned transitions rather than mutating coordination records directly
- the node can still obtain enough observation from PostgreSQL and the DCS to tell the difference between trustworthy and untrustworthy conditions

If those assumptions are badly violated, the safety case weakens. The design can still fail conservatively in many cases, but the confidence in its conclusions drops accordingly.

## What this argument does not guarantee

This chapter does not guarantee:

- zero downtime
- immediate failover under every partial failure
- immunity to extreme multi-fault scenarios or severe misconfiguration
- correctness if external automation bypasses the documented control surfaces and edits DCS state directly
- operator understanding without observability; the system can publish bounded state, but it cannot force correct human interpretation

Those non-guarantees matter because overstating the design would make the docs less trustworthy than the code.

## Residual risk

Residual risk remains in several classes:

- severe or prolonged coordination failure may leave the cluster conservative for longer than operators want
- misconfiguration of scope, auth, or binary paths can block valid lifecycle transitions
- extreme combinations of network partition, stale observation, and operator action can still create hard-to-diagnose states
- recovery can still fail if rewind, clone, or bootstrap prerequisites are not actually present

The design response to those risks is not to pretend they do not exist. It is to bias the runtime toward observability and conservative action when the evidence is too weak to support stronger claims.

## Why this matters

A written safety case improves both operator trust and contributor discipline. Operators gain a clearer explanation for why certain conservative outcomes are expected. Contributors gain a standard against which new features should be judged: if a new behavior weakens these arguments by bypassing trust, blurring write ownership, or skipping explicit recovery gates, it should be treated with skepticism.
