# Why DCS trust and coordination shape the cluster

DCS is more than a key-value transport. It is a shared coordination state that cluster members must actively publish into and continuously validate. In pgtuskmaster, the DCS worker publishes the local member record before evaluating trust. If that publication fails, trust immediately becomes NotTrusted. The worker drains watch events into a cache and marks the store unhealthy when write, drain, or refresh operations fail. When the store is unhealthy, the published DcsState uses WorkerStatus::Faulted and forces DcsTrust::NotTrusted.

## Trust as a precondition for authority

The trust model—FullQuorum, FailSafe, NotTrusted—determines whether a member can assert authority over the cluster. Trust is not derived from passive observation; it requires successful local member publication and a healthy DCS store. The DCS worker’s ability to write its own record and process incoming events forms the basis of trust. Without these mechanisms, a member cannot safely participate in consensus or failover.

## How trust changes fail-safe behavior

HA decision logic begins by checking trust. If trust is not FullQuorum and Postgres is primary, the next phase is FailSafe with EnterFailSafe { release_leader_lease: false }. If trust is not FullQuorum and Postgres is not primary, the next phase is FailSafe with NoChange. These paths show that trust directly influences whether the system attempts to retain leadership or freezes further action.

## Consequences for APIs, tests, and operators

The API controller writes switchover requests into DCS-backed scope paths rather than directly assigning leadership. Operators cannot bypass this indirection. The policy test allows post-start observation through GET /ha/state and admin switchover requests, but it forbids direct DCS writes, internal worker calls, and other internal steering after startup. This design forces all control-plane interactions through the API surface and treats the DCS as the sole source of coordination truth.
