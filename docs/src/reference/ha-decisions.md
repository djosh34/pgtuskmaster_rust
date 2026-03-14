# HA State Reference

## Overview

The `ha` section of `GET /state` exposes the current high-availability engine state. This reference describes each field, its type, and the structured planned-action view the HA worker publishes after every reconciliation pass.

## Top-Level Fields

```text
ha: {
  "worker": WorkerStatus,
  "tick": u64,
  "required_roles_ready": bool,
  "publication": PublicationState,
  "role": TargetRole,
  "world": WorldView,
  "clear_switchover": bool,
  "planned_actions": PlannedActions
}
```

- `worker` - Current HA worker status
- `tick` - Monotonic reconciliation counter
- `required_roles_ready` - Whether local required PostgreSQL roles are ready
- `publication` - Operator-facing authority projection
- `role` - Target local HA role for this node
- `world` - Derived local/global worldview used for the current decision
- `clear_switchover` - Whether the node wants to clear a pending switchover request
- `planned_actions` - Structured immediate next work split by action family

## ha.role

`role` describes the node's target local behavior.

Variants:

- `leader` - Hold lease and act as primary
- `candidate` - Attempt to acquire lease
- `follower` - Replicate from a leader
- `fail_safe` - Enter safety mode due to degraded trust
- `demoting_for_switchover` - Release leadership for a specific target
- `fenced` - Stop unsafe primary behavior
- `idle` - Wait for conditions before acting

## ha.publication

`publication` conveys the operator-facing authority view.

### PublicationState

Enum with two top-level shapes:

- `unknown` - Cold-start value before HA produces a stronger publication
- `projected` - Wraps an `AuthorityProjection`

### AuthorityProjection

Enum with two variants:

- `primary` - Carries `LeaseEpoch` for the authoritative primary
- `no_primary` - Carries `NoPrimaryProjection` with a structured reason

### NoPrimaryProjection

Variants:

- `lease_open` - No lease currently held
- `recovering` - Cluster is recovering; may include epoch and fence
- `dcs_degraded` - DCS trust is not full quorum; may include fence
- `stale_observed_lease` - Observed lease holder is invalid; carries epoch and reason
- `switchover_rejected` - Switchover target is invalid; carries blocker reason

### Fence Information

Fence data appears only inside `no_primary` projections as `NoPrimaryFence`:

- `none` - No fence required
- `cutoff` - Contains `FenceCutoff { epoch, committed_lsn }`

## ha.planned_actions

`planned_actions` is a structured read model of the immediate next work. It is not an ordered mixed command list.

Fields:

- `publication: Option<PublicationAction>` - Publication updates for operator-visible authority
- `coordination: Option<CoordinationAction>` - Lease and switchover coordination work
- `local: Option<LocalAction>` - Local node maintenance work
- `process: Option<ProcessIntent>` - PostgreSQL/process intent for the process worker boundary

### PublicationAction

- `publish(PublicationGoal)` - Publish a new authority projection

### CoordinationAction

- `acquire_lease(Candidacy)` - Attempt to become lease holder
- `release_lease` - Relinquish the lease
- `clear_switchover` - Remove the current switchover request

### LocalAction

- `ensure_required_roles` - Create or verify required PostgreSQL roles

### ProcessIntent

- `bootstrap` - Initialize a new data directory
- `provision_replica` - Rebuild replica data from a leader
- `start` - Start PostgreSQL in a specific mode
- `promote` - Promote a replica to primary
- `demote` - Stop PostgreSQL for demotion/fencing

#### ReplicaProvisionIntent

- `base_backup { leader }` - Clone from a leader with `pg_basebackup`
- `pg_rewind { leader }` - Repair divergence with `pg_rewind`

#### PostgresStartIntent

- `primary` - Start as primary
- `detached_standby` - Start as detached standby
- `replica { leader }` - Start as a replica following a specific leader

## ha.world

`world` exposes the raw derived worldview that produced the results.

### LocalKnowledge Fields

- `data_dir: DataDirState` - Local data directory status
- `postgres: PostgresState` - Observed PostgreSQL runtime state
- `process: ProcessState` - Current process worker status
- `storage: StorageState` - Storage health assessment
- `required_roles_ready: bool` - Whether required roles exist
- `publication: PublicationState` - Last published authority
- `observation: ObservationState` - Timing metadata for PostgreSQL state changes

### GlobalKnowledge Fields

- `coordination: CoordinationState` - DCS lease and trust status
- `switchover: SwitchoverState` - Active switchover requests
- `peers: BTreeMap<MemberId, PeerKnowledge>` - Observed peer status
- `self_peer: PeerKnowledge` - This node's eligibility and visibility

### CoordinationState

- `trust: DcsTrust` - DCS trust level (full_quorum, degraded, not_trusted)
- `leadership: LeadershipView` - Current lease holder view
- `primary: PrimaryObservation` - Observed primary member

### LeadershipView

- `open` - No lease currently held
- `held_by_self` - This node holds the lease
- `held_by_peer` - Another node holds the lease; includes epoch and leader state
- `stale_observed_lease` - Observed lease is invalid; includes epoch and reason

### PrimaryObservation

- `absent` - No primary observed
- `observed` - Wraps `ObservedPrimary` with member, timeline, and system identifier

### DataDirState

- `missing` - Data directory does not exist
- `initialized` - Wraps `LocalDataState`

### LocalDataState

- `bootstrap_empty` - Data directory exists but is uninitialized
- `consistent_replica` - Data is consistent with observed primary
- `diverged` - Data has diverged; wraps `DivergenceState`

### DivergenceState

- `rewind_possible` - Divergence can be repaired with pg_rewind
- `basebackup_required` - Divergence requires full rebuild

**System Identifier Mismatch Rule**

When both the local and observed primary system identifiers are present and different, the data directory is classified as `diverged(DivergenceState::basebackup_required)`.

### PostgresState

- `offline` - PostgreSQL is not running
- `primary` - PostgreSQL is running as primary with committed LSN
- `replica` - PostgreSQL is running as replica with upstream and replication state

### ReplicationState

- `streaming` - Actively receiving WAL with position
- `catching_up` - Replaying WAL with position
- `stalled` - Replication is blocked or lagging

### RecoveryPlan

Used by `FollowGoal` to determine recovery actions:

- `none` - No recovery needed; start streaming
- `start_streaming` - Start PostgreSQL as replica
- `rewind` - Rewind diverged data before starting
- `basebackup` - Rebuild data directory from leader

The HA engine selects recovery plans based on `LocalDataState` and divergence assessment.
