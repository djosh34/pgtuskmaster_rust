# Architecture

pgtuskmaster is a high-availability orchestrator for PostgreSQL that prioritizes
split-brain prevention over raw availability. Its architecture reflects this
choice through a state-driven decision loop, explicit trust modeling, and
modular separation between observation, decision, and action.

## Core Design Principles

The system operates as a deterministic state machine rather than an event
processor. Each tick of the high-availability (HA) worker ingests a complete
world snapshot---a composite of DCS state, PostgreSQL status, and local process
outcomes---and emits a single decision.

Safety guides every architectural layer. The system enters a fail-safe state
when it cannot verify cluster membership freshness through the DCS, even if this
means rejecting leadership. The `DcsTrust` evaluation deliberately downgrades
trust when member records are stale or incomplete, ensuring that leadership
decisions are never made on fragmented information.

Modularity enforces boundaries. The `ha` module owns decision logic but not
API concerns. The `api` module translates internal state into external
observability, and the `dcs` layer abstracts store interactions behind a trait.

## Component Organization

The crate root in `src/lib.rs` exposes public modules and internal modules.

[diagram about module hierarchy showing public surface vs internal implementation,
**diagram shows `api`, `cli`, `config`, `dcs`, `runtime`, `state` as public; `ha`,
`logging`, `postgres_managed`, `postgres_managed_conf`, `process`, `tls`,
`debug_api` as internal**]

The `ha` module contains the phase-based decision engine. It consumes current
facts derived from the world and produces a `HaDecision`.

The `dcs` module maintains cluster membership and leader records in etcd. It
evaluates trust by checking freshness of the local member record, the leader
record (if any), and the overall member count. Freshness depends on a
configurable lease TTL; records older than this threshold degrade trust.

The `process` module participates in subprocess execution concerns used by the
rest of the system.

The `api` module exposes a read-only view of the system snapshot and accepts
control actions like switchover requests. It writes requests into DCS but does
not participate in the decision loop---a deliberate separation that keeps
control-plane concerns distinct from the data-plane HA engine.

## HA Phase Machine

HA progression follows a discrete set of phases. Each phase represents a
stable operational context with specific exit conditions.

[diagram about phase transitions showing allowed paths,
**diagram shows nodes: Init → WaitingPostgresReachable → WaitingDcsTrusted →
(Replica | CandidateLeader) → Primary, with failure paths into Rewinding,
Bootstrapping, Fencing, FailSafe**]

*   **Init**: System bootstrap; immediately transitions to waiting for PostgreSQL.
*   **WaitingPostgresReachable**: PostgreSQL must be reachable or successfully
    started. If PostgreSQL is down but start is allowed, the decision requests
    process startup. Once reachable, transitions to waiting for DCS trust.
*   **WaitingDcsTrusted**: Awaits `DcsTrust::FullQuorum`. If trust is degraded,
    the system may enter `FailSafe`. If trust is full and PostgreSQL remains
    reachable, the system becomes either a replica (following a known leader) or a
    candidate leader (if no leader exists).
*   **Replica**: Follows an established leader. If the leader disappears or
    becomes unhealthy, the system becomes a candidate. If replication lag
    requires rewinding, transitions to `Rewinding`.
*   **CandidateLeader**: Attempts to acquire the leader lease. If successful,
    becomes primary without promoting PostgreSQL if it is already primary.
*   **Primary**: Holds the leader lease and serves as the write target. On
    switchover request, demotes and transitions to `WaitingSwitchoverSuccessor`.
    If another active leader appears, enters `Fencing` and steps down.
*   **Rewinding**: Recovery behavior in this phase depends on process activity and follow-target evaluation.
*   **Bootstrapping**: Recovery behavior in this phase depends on current facts and selected strategy.
*   **Fencing**: This phase exists to handle unsafe leader situations and step-down plans.
*   **FailSafe**: Quiescent state when DCS trust is absent. No leadership
    actions are taken. Exits only when trust is restored.
*   **WaitingSwitchoverSuccessor**: Primary demotion is complete; waits for a new
    leader to be elected before transitioning to replica.

Every phase transition is guarded by preconditions checked against the world
snapshot. No transition occurs based on timers or events alone.

## Trust and Safety Model

DCS trust evaluation is the cornerstone of split-brain prevention. The function
`evaluate_trust` in `src/dcs/state.rs` enforces three progressive thresholds:

[diagram about trust evaluation flow,
**diagram shows decision tree: etcd_healthy? → members contain self? → self record
fresh? → leader record present and fresh? → fresh member count >= 2? → FullQuorum,
else FailSafe; any false yields NotTrusted or FailSafe**]

1.  **FullQuorum**: The DCS store is healthy, the local member record exists
    and is fresh, and any recorded leader is also fresh. In multi-node caches,
    at least two members must be fresh. This allows leadership activity.
2.  **FailSafe**: The store is healthy but membership information is partial
    or stale. The system avoids leadership decisions but continues normal
    operation if already a replica.
3.  **NotTrusted**: The store is unreachable or unhealthy. All HA decisions are
    suspended.

Freshness uses a TTL derived from `ha.lease_ttl_ms`. A member record is stale
if its `updated_at` exceeds this duration. This simple lease mechanism avoids
dependency on wall-clock synchronization while bounding membership uncertainty.

When trust is lost while a node believes itself primary, the system enters
`FailSafe`. If another active leader is detected while primary logic is running,
the system enters `Fencing` through a step-down plan.

## Configuration Architecture

Runtime configuration is loaded from a TOML file into a `RuntimeConfig` struct.
The schema distinguishes between `InlineOrPath` references and secret-bearing
fields that can use `SecretSource`. Configuration is parsed as a single contract.

Key sections include:

-   **cluster**: Identity (`name`, `member_id`) used for DCS namespacing.
-   **postgres**: Data directory, listen addresses, connection identities,
    TLS settings, roles and authentication, `pg_hba` and `pg_ident` sources.
-   **dcs**: Etcd endpoints and scope prefix for all keys.
-   **ha**: Loop interval and lease TTL, which directly affect trust evaluation.
-   **process**: Timeouts for rewind, bootstrap, and fencing operations;
    binary paths for all PostgreSQL utilities.
-   **api**: Listen address, TLS configuration, optional role-token auth.
-   **debug**: Enables read-only introspection endpoints.

The configuration file in `docker/configs/cluster/node-a/runtime.toml`
demonstrates a complete runtime configuration. Binary paths, role credentials, and
`pg_hba` sources are explicit, allowing operators to bind-mount or inject
secrets without rebuilding images.

## Observability and Control Surfaces

The API provides two categories of endpoints: observability and control.

Observability endpoints translate the system snapshot into JSON responses.
`get_ha_state` maps internal enums such as `DcsTrust` and `HaPhase` into stable
API types, exposing cluster name, leader, member count, trust level, phase,
decision, and snapshot sequence. This lets external monitoring tools inspect
the decision engine without parsing logs.

Control endpoints accept switchover requests. A POST to `/switchover` writes a
`SwitchoverRequest` record into etcd; the HA decision loop later observes this
record and initiates demotion. The API does not execute the switchover directly,
preserving the decision loop's authority. DELETE clears the request, supporting
cancellation before execution.

## Operational Invariants and Evidence

The test harness in `tests/ha/support/observer.rs` codifies the operational
expectations of the architecture. The `HaInvariantObserver` samples API and SQL
states over time and asserts that `max_concurrent_primaries` never exceeds one.
If it does, the observer raises a split-brain error with recent sample history.

This invariant shapes the architecture: the HA engine must guarantee that
leadership is mutually exclusive at all observable points. The trust model,
lease TTL, and fencing logic exist solely to satisfy this invariant. The
observer's requirement for a minimum number of successful samples before
declaring a window safe reflects the architecture's emphasis on consistency
over transient liveness.

## Integration Points

The architecture achieves coherence through well-defined integration points:

-   **DCS store trait**: Store interactions are abstracted behind `DcsStore`,
    enabling test doubles.

These points are not accidental. They enforce loose coupling between
subsystems, allow independent testing, and prevent hidden state sharing that
could undermine determinism.

## Summary

pgtuskmaster's architecture balances three forces: safety, observability, and
operability. Safety is achieved through a trust model that quantifies membership
freshness and a phase machine that refuses leadership when trust is degraded.
Observability is achieved by exposing a coherent snapshot through the API and
by recording detailed state transitions in logs. Operability is achieved through
versioned, file-based configuration and a control plane that writes intentions
into DCS rather than executing them directly.

The design favors explicit state and trust-gated leadership decisions. These
choices are visible in the phase machine and the trust evaluation function.
