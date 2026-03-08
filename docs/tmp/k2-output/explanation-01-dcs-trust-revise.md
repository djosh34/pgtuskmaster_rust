# Why DCS trust and coordination shape the cluster

A distributed consensus store is not a passive key-value transport for PostgreSQL state. In pgtuskmaster, the DCS is the sole source of truth for membership, health, and leadership claims. Its behavior directly determines whether the local member can act with authority, and how the high-availability loop responds when that authority is in doubt.

## DCS as shared coordination state

The DCS worker in `src/dcs/worker.rs` continuously publishes the local member record, drains watch events into a private cache, and refreshes leadership leases. This is not best-effort gossip: every write, drain, or refresh failure marks the DCS store unhealthy, which in turn surfaces `WorkerStatus::Faulted` and `DcsTrust::NotTrusted` in the published `DcsState`. The local member’s view of the cluster is therefore contingent on its ability to both emit and receive state. If the member cannot publish its own record, trust is immediately downgraded. The healthy case is a feedback loop: the member writes, observes its own write, and infers that the DCS is functional enough to carry authoritative state.

## Trust as precondition for authority

The trust model has three levels: `FullQuorum`, `FailSafe`, and `NotTrusted`. `FullQuorum` means the DCS worker has successfully published, drained, and refreshed; the store is healthy and the member perceives itself as part of a coherent cluster. Only under `FullQuorum` may the HA loop make forward progress such as acquiring or holding the primary lease. `NotTrusted` means the worker has lost confidence in the DCS entirely: writes fail, drainage stalls, or the lease refresh aborts. In this state the member forfeits authority and the HA loop enters a defensive posture.

Trust is evaluated every time the DCS worker emits a state update, and the HA decision logic in `src/ha/decide.rs` consults trust before any other condition. This ordering is deliberate: trust is a gate, not a suggestion. Without it, the member cannot know whether its leadership claim would conflict with another member that has already written to DCS, and it cannot trust the watch cache that informs its decisions.

## How trust changes fail-safe behavior

When trust is `FullQuorum`, the HA loop follows the nominal path: if the member holds the DCS lease and PostgreSQL is healthy, it remains primary; if it is a replica, it follows the upstream. When trust drops to `FailSafe`, the loop stops pursuing new leadership but exhibits different behavior based on local PostgreSQL state.

If trust is not `FullQuorum` and PostgreSQL is already primary, the next phase is `EnterFailSafe { release_leader_lease: false }`. The member keeps the lease frozen and leaves PostgreSQL running, gambling that the DCS outage is transient and that fencing has not yet activated. If trust is not `FullQuorum` and PostgreSQL is not primary, the phase becomes `NoChange`: the replica neither promotes nor shuts down, preserving the status quo while waiting for DCS health to return.

`NotTrusted` is the extreme case where the DCS store is explicitly unhealthy. The HA loop treats this as `FailSafe` because the member can no longer distinguish a network partition from a true loss of quorum. The code does not attempt to reconstitute quorum locally; it defers to the DCS worker’s continuous attempts to heal the store.

## Consequences for APIs, tests, and operators

The API controller in `src/api/controller.rs` respects the trust boundary. It does not directly assign leadership; instead it writes switchover requests into DCS-backed scope paths such as `/requests/switchover/`. The DCS worker treats these requests as regular watch events, and the HA loop acts on them only when trust is `FullQuorum`. This design removes a temptation to bypass coordination and makes every consequential action observable to all healthy members.

Operators interact with the cluster through the same narrow surface. The policy test in `tests/policy_e2e_api_only.rs` codifies this contract: after startup, permissible actions are limited to observation (GET /ha/state) and approved steering (admin switchover requests). Direct DCS writes, internal worker calls, and other escape hatches are forbidden. If an operator attempts to force a promotion by editing DCS keys manually, they will race against the trust decision and will likely be ignored or reverted by the healthy worker on the primary node, which still believes it holds authority.

For engineers, this means that any change to HA behavior must consider trust as the first-order effect. Adding a new decision branch without consulting `dcs_state.trust` will silently break the fail-safe contract. Adding a new API endpoint that writes DCS keys must go through the controller and be covered by the policy test. The system is designed to prefer observed downtime over split-brain, and trust is the mechanism that enforces that preference.
