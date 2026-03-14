# HA State Reference

## Overview

The `ha` section of `GET /state` exposes the current high-availability engine state. This reference describes each field, its type, and possible values.

## Top-Level Fields

```text
ha: {
  "role": TargetRole,
  "publication": PublicationState,
  "planned_commands": [ReconcileAction],
  "world": WorldView
}
```

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

## ha.planned_commands

`planned_commands` is an ordered list of actions the worker will execute.

Command kinds:

- `init_db` - Initialize a new data directory
- `base_backup` - Clone from a leader using pg_basebackup
- `pg_rewind` - Rewind diverged data directory using pg_rewind
- `start_primary` - Start PostgreSQL in primary mode
- `start_replica` - Start PostgreSQL replicating from a leader
- `promote` - Promote a replica to primary
- `demote` - Demote primary to replica (fast or immediate shutdown)
- `acquire_lease` - Attempt to become lease holder
- `release_lease` - Relinquish lease holder status
- `ensure_required_roles` - Create required PostgreSQL roles
- `publish` - Update published authority projection
- `clear_switchover` - Remove switchover request from DCS

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
