# Architecture deep summary

This note is a source-backed summary for `docs/src/explanation/architecture.md`.

The crate root in `src/lib.rs` exposes the major subsystems:
- `api`
- `cli`
- `config`
- `dcs`
- `runtime`
- `state`

There are also internal modules that matter to architecture:
- `ha`
- `logging`
- `postgres_managed`
- `postgres_managed_conf`
- `process`
- `tls`
- `debug_api`

The system is organized around a few cooperating concerns:
- runtime configuration loading
- PostgreSQL process management and state observation
- DCS state publication and trust evaluation
- HA decision making
- API projection of current cluster state

HA decision loop facts from `src/ha/decide.rs`:
- each decision tick derives `DecisionFacts` from the current world snapshot
- HA behavior is phase-driven
- main phases include `Init`, `WaitingPostgresReachable`, `WaitingDcsTrusted`, `Replica`, `CandidateLeader`, `Primary`, `Rewinding`, `Bootstrapping`, `Fencing`, `FailSafe`, and `WaitingSwitchoverSuccessor`
- the decision loop increments a tick counter and carries the last decision into the next state

Key safety behavior visible in `decide_phase` and related helpers:
- if DCS trust is not `FullQuorum`, the node does not continue normal leader logic
- if DCS trust is degraded and the local postgres is primary, the node enters `FailSafe` with `EnterFailSafe`
- if DCS trust is degraded and postgres is not primary, the node still enters `FailSafe` but with `NoChange`
- promotion only happens through explicit HA decisions such as `AttemptLeadership` followed by `BecomePrimary`
- when another active leader is detected while the local node is primary, the node enters `Fencing` and performs a `StepDown` plan with `fence = true`
- if primary postgres becomes unreachable while the node still holds leadership, the node releases the leader lease before recovery

Those transitions show the intended split-brain resistance:
- leadership depends on DCS trust
- the node treats foreign leaders as fencing conditions
- fail-safe mode interrupts normal leadership behavior when quorum trust is absent

DCS trust model from `src/dcs/state.rs`:
- trust values are `FullQuorum`, `FailSafe`, and `NotTrusted`
- if the etcd-backed store is unhealthy, trust becomes `NotTrusted`
- if the local member record is missing or stale, trust becomes `FailSafe`
- if a leader record exists but the leader member record is missing or stale, trust becomes `FailSafe`
- in multi-member caches, fewer than two fresh members also downgrades trust to `FailSafe`
- otherwise trust is `FullQuorum`

DCS cached state includes:
- member records
- leader record
- switchover request
- runtime config snapshot
- init lock

Member records carry enough replication state for HA comparisons:
- role
- SQL status
- readiness
- timeline
- write LSN
- replay LSN
- updated timestamp
- PostgreSQL version

API role from `src/api/controller.rs`:
- it writes switchover requests into the DCS namespace
- it deletes switchover requests through the DCS writer helper
- it maps internal HA and DCS state into stable API response enums and structs
- API responses expose cluster name, scope, self member id, leader, member count, DCS trust, HA phase, HA decision, and snapshot sequence

That means the API is mainly a control and observability surface, not the place where HA decisions are computed.
The controller translates internal state; it does not own the HA algorithm.

Operational invariant evidence from `tests/ha/support/observer.rs`:
- the HA observer samples API states and SQL roles over time
- it explicitly tracks `max_concurrent_primaries`
- it raises an error if more than one primary is observed
- it also rejects insufficient evidence windows when there are too few successful samples
- recent sample rings are retained to explain failures

This test harness behavior is useful architectural evidence:
- split-brain avoidance is a first-class invariant
- the system is expected to be judged over time-series observations, not just single snapshots
- API and SQL perspectives are both used to confirm safety

Concrete coordination story supported by the files:
- runtime config defines cluster identity, DCS scope, HA timeouts, postgres connection details, and API settings
- the DCS worker publishes and consumes cluster membership and leader information
- the HA worker consumes world facts built from DCS state and PostgreSQL reachability/role data
- the HA worker outputs decisions such as following a leader, attempting leadership, rewinding, bootstrapping, fencing, or entering fail-safe
- the API layer exposes the resulting state and lets operators request a switchover by writing into DCS

Be careful not to overclaim:
- the files here do not prove a full message-flow diagram or task scheduler topology beyond these module responsibilities
- the exact runtime thread model is not established by these excerpts alone
- any explanation should stay grounded in the visible trust evaluation, phase machine, and API translation responsibilities
