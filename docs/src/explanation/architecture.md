# Architecture

pgtuskmaster is a high-availability orchestrator for PostgreSQL that prioritizes split-brain prevention through a state-driven decision loop, explicit trust modeling, and clear boundaries between DCS state, HA decisions, and API projection.

## Core Design Principles

The HA worker is structured as a deterministic state machine. Each decision tick derives `DecisionFacts` from the current world snapshot and emits a single `HaDecision`. The decision logic in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) is phase-driven rather than ad hoc, which makes promotion, following, rewinding, fencing, and fail-safe handling explicit.

Safety is enforced through trust gating. The DCS trust model in [`src/dcs/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) can downgrade the node to `FailSafe` or `NotTrusted` when etcd health, member freshness, or leader freshness is not good enough for normal HA decisions. That means the system does not treat DCS availability as a convenience layer; it is a prerequisite for leadership behavior.

The module split in [`src/lib.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/lib.rs) also reflects those boundaries. `ha` owns decision logic, `dcs` owns cluster-state storage concerns, `config` owns runtime configuration, and `api` maps internal state into response types and switchover control entry points.

```mermaid
flowchart LR
    Config[Runtime config]
    DCS[DCS cache and trust]
    PG[PostgreSQL state]
    HA[HA decide phase machine]
    API[API state and switchover surface]

    Config --> DCS
    Config --> HA
    PG --> HA
    DCS --> HA
    HA --> API
    API --> DCS
```

## Component Organization

The crate root exposes the major public areas: `api`, `cli`, `config`, `dcs`, `pginfo`, `runtime`, and `state`. Internal modules such as `ha`, `logging`, `process`, `postgres_managed`, and `tls` carry the coordination and implementation details.

The HA layer produces decisions such as:
- `WaitForPostgres`
- `WaitForDcsTrust`
- `AttemptLeadership`
- `FollowLeader`
- `BecomePrimary`
- `RecoverReplica`
- `StepDown`
- `FenceNode`
- `ReleaseLeaderLease`
- `EnterFailSafe`

Those decisions are visible in [`src/ha/decision.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs) and are emitted by the phase handlers in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs).

The DCS layer stores membership and leadership data in a cache that contains member records, leader state, switchover state, the runtime config snapshot, and an optional init lock. Member records include role, readiness, SQL state, timeline, WAL position fields, update timestamp, and PostgreSQL version. Trust evaluation uses that cache together with store health to decide whether the HA layer can act normally.

The API layer in [`src/api/controller.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/controller.rs) has two main responsibilities:
- project internal HA and DCS state into stable response enums and structs
- write or clear switchover requests in the DCS namespace

That keeps the API as a control and observability surface rather than the place where HA decisions are computed. The switchover request itself is generic; the HA loop still chooses the eligible successor from observed cluster state.

## HA Phase Machine

The HA state includes these phases:
- `Init`
- `WaitingPostgresReachable`
- `WaitingDcsTrusted`
- `WaitingSwitchoverSuccessor`
- `Replica`
- `CandidateLeader`
- `Primary`
- `Rewinding`
- `Bootstrapping`
- `Fencing`
- `FailSafe`

The phase handlers in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) show the intended flow:
- startup moves from `Init` toward PostgreSQL reachability and then DCS trust
- a trusted node without a leader can move into `CandidateLeader`
- a trusted node with a follow target stays or becomes `Replica`
- a node that holds leadership moves into `Primary`
- degraded trust can move the node into `FailSafe`
- unsafe leader situations can move the node into `Fencing`
- replica recovery can route through `Rewinding` or `Bootstrapping`

```mermaid
flowchart TD
    Init --> WaitingPostgresReachable
    WaitingPostgresReachable --> WaitingDcsTrusted
    WaitingDcsTrusted --> Replica
    WaitingDcsTrusted --> CandidateLeader
    CandidateLeader --> Primary
    Replica --> CandidateLeader
    Replica --> Rewinding
    Primary --> WaitingSwitchoverSuccessor
    Primary --> Fencing
    Primary --> Rewinding
    Rewinding --> Replica
    Rewinding --> Bootstrapping
    Bootstrapping --> WaitingPostgresReachable
    Fencing --> WaitingDcsTrusted
    WaitingSwitchoverSuccessor --> Replica
    WaitingDcsTrusted --> FailSafe
```

The exact next state still depends on the current facts. For example, `Primary` can step down on switchover, enter fencing when another active leader is detected, or release leadership when PostgreSQL becomes unreachable.

## Trust and Safety Model

Trust evaluation in [`src/dcs/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) is one of the key architectural constraints:
- `NotTrusted` is used when the backing store is unhealthy
- `FailSafe` is used when the store is healthy but the local member is missing or stale, or when a multi-member view lacks enough fresh members
- `FullQuorum` is used only when the store is healthy and the cache is fresh enough to support normal HA behavior

Freshness is checked against `ha.lease_ttl_ms`, using each member record's `updated_at` timestamp. In multi-member caches, the code requires at least two fresh members before returning `FullQuorum`.

Leader liveness is handled separately from trust. The etcd-backed store writes `/{scope}/leader` under an etcd lease whose TTL is also derived from `ha.lease_ttl_ms`. When the owning node dies hard and keepalive stops, etcd expires the lease and the watch-fed DCS cache drops the leader record automatically. That keeps leader expiry in the DCS/store layer instead of duplicating a second expiry clock inside the HA phase machine.

The HA decision logic uses that trust result immediately. At the top of `decide_phase`, if trust is not `FullQuorum`, the node enters `FailSafe`; if it is still primary at that moment, the decision carries `EnterFailSafe { release_leader_lease: false }`.

```mermaid
flowchart TD
    EtcdHealthy{Store healthy?}
    SelfPresent{Self member present?}
    SelfFresh{Self member fresh?}
    EnoughFresh{At least two fresh members when cluster has multiple members?}
    NotTrusted[NotTrusted]
    FailSafe[FailSafe]
    FullQuorum[FullQuorum]

    EtcdHealthy -- no --> NotTrusted
    EtcdHealthy -- yes --> SelfPresent
    SelfPresent -- no --> FailSafe
    SelfPresent -- yes --> SelfFresh
    SelfFresh -- no --> FailSafe
    SelfFresh -- yes --> EnoughFresh
    EnoughFresh -- no --> FailSafe
    EnoughFresh -- yes --> FullQuorum
```

This trust gate is the reason the architecture puts so much weight on DCS freshness instead of only local PostgreSQL status. Local primary state is not enough on its own.

## Configuration in the Architecture

Runtime configuration shapes almost every subsystem. The schema in [`src/config/schema.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/schema.rs) defines top-level sections for:
- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`

That config controls cluster identity, DCS scope, PostgreSQL connection and authentication details, process binary paths, HA timing, log sinks, and API security settings. The Docker example at [`docker/configs/cluster/node-a/runtime.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docker/configs/cluster/node-a/runtime.toml) shows the sections together in a complete runtime config file.

Because `DcsCache` carries a `RuntimeConfig`, configuration is not just startup input. It is part of the state used by trust evaluation and other worker behavior.

## Observability and Control

The controller surface in [`src/api/controller.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/controller.rs) maps internal state into API responses that include:
- cluster name
- scope
- self member id
- leader
- switchover pending
- member count
- DCS trust
- HA phase
- HA decision
- snapshot sequence

The same controller also accepts a generic switchover request and writes it into the DCS namespace. That means operator intent enters through the API, but the HA loop still decides what to do with that request and which member becomes the new primary.

The HA observer in [`tests/ha/support/observer.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/observer.rs) gives another useful architectural clue: the project explicitly measures whether more than one primary is observed during a sample window. Split-brain avoidance is not implicit; it is a tested operational invariant.

## Summary

The core architectural pattern is: collect state, evaluate trust, run the HA phase machine, then expose the result through the API. The important constraint is that leadership is trust-gated. DCS freshness, member freshness, and explicit phase transitions are the mechanisms that keep high availability behavior explainable and defensive.
