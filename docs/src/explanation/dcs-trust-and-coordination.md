# Why DCS trust and coordination shape the cluster

DCS is not just a place to stash cluster metadata. In pgtuskmaster it is the shared coordination surface that every member must both publish into and read back from before HA behavior is allowed to carry authority.

The [DCS reference](../reference/dcs.md) describes the record types and worker loop. This page explains why those mechanics are treated as a gate on cluster behavior rather than as passive background plumbing.

## Trust is a precondition for authority

The DCS worker publishes the local member record before it evaluates trust. If that local publication fails, trust falls to `NotTrusted`. The worker also drains watch events into a cache and treats write, drain, and refresh failures as store-health problems. When the store is unhealthy, the published `DcsState` becomes faulted and trust is forced to `NotTrusted`.

That design answers a simple question: when should a node believe that its cluster view is authoritative enough to guide failover? The answer is "only when it can both contribute its own state and keep up with shared state from the rest of the cluster". A node that cannot publish or cannot reliably refresh the cache might still have partial information, but partial information is not enough to make safe leadership decisions.

## Why trust feeds directly into HA

The [HA state machine](../reference/ha-state-machine.md) begins with a global trust check. If trust is not `FullQuorum` and Postgres is primary, HA moves into `FailSafe` with `EnterFailSafe { release_leader_lease: false }`. If trust is not `FullQuorum` and the node is not primary, HA still moves into `FailSafe`, but with `NoChange`.

This is more than a convenience shortcut. It keeps DCS trust from becoming an advisory metric that operators must interpret by hand. Instead, the trust model becomes part of the policy surface. HA does not first decide what it wants and only later discover that coordination data was shaky. Trust is upstream of authority.

## Why control requests are indirect

The control surface follows the same philosophy. The HTTP controller writes switchover requests into DCS-backed scope paths instead of exposing a direct "make this node leader now" command. That keeps operator intent inside the same coordination path that HA already trusts and reasons about.

The post-start policy tests reinforce this boundary. They allow observation through `GET /ha/state` and supported switchover requests, but they forbid direct DCS writes and internal worker steering after startup. The consequence is deliberate: once the cluster is running, coordination should happen through the same guarded surfaces that production control uses.

## The tradeoff

This posture can look conservative. A node may still have useful local information while being treated as untrusted from the cluster's point of view. That is the cost of preferring coordination safety over aggressive local action.

The benefit is that split-brain protection is not left to operator discipline or to scattered checks in downstream code. Trust is evaluated once, published explicitly, and consumed by HA as a first-order input.
