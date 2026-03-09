# HA Authority Reconciliation

This page describes the distributed consensus authority model that governs cluster lifecycle decisions. All nodes derive identical cluster states and desired actions from a shared set of DCS facts using deterministic rules.

## DCS Authority Facts

The runtime registers four authoritative fact families under the cluster scope prefix:

| Path | Type | Description |
|------|------|-------------|
| `/cluster/initialized` | durable | Singleton record; written once after first bootstrap completes |
| `/cluster/identity` | durable | Singleton record; authoritative `system_identifier`, bootstrap winner, and timestamp |
| `/bootstrap` | leased | Singleton lock; held only during active bootstrap |
| `/leader` | leased | Singleton lock; held only by current elected primary |

Member observations remain under `/member/{member_id}`. Member records serve as the live observation surface and carry the pre-election descriptor for offline ranking.

**Record Definitions**

```rust
struct ClusterInitializedRecord {
    initialized_by: MemberId,
    initialized_at: UnixMillis,
}

struct ClusterIdentityRecord {
    system_identifier: SystemIdentifier,
    bootstrapped_by: MemberId,
    bootstrapped_at: UnixMillis,
}

struct BootstrapLockRecord {
    holder: MemberId,
}

struct LeaderRecord {
    member_id: MemberId,
}
```

## Trust Vocabulary

`DcsTrust` compresses store health and quorum freshness into three states:

```rust
enum DcsTrust {
    NotTrusted,      // store unreachable or unhealthy
    NoFreshQuorum,   // store reachable but insufficient fresh members
    FreshQuorum,     // store healthy with enough fresh members
}
```

### Trust Evaluation

Trust derives per tick from etcd health and member freshness. A member is fresh if its `updated_at` is within `ha.lease_ttl_ms`. Fresh quorum requires:

- Single-member observed set: exactly one fresh member
- Multi-member observed set: at least two fresh members

```rust
fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsCache,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust
```

Startup and steady-state HA obey the same rule: an initialized cluster only runs writable primary behavior when trust is `FreshQuorum`.

## Derived Cluster Modes

Each tick, the runtime maps DCS facts into exactly one cluster-wide mode:

```rust
enum ClusterMode {
    UninitializedNoBootstrapOwner,
    UninitializedBootstrapInProgress { holder: MemberId },
    InitializedLeaderPresent { leader: MemberId },
    InitializedNoLeaderFreshQuorum,
    InitializedNoLeaderNoFreshQuorum,
    DcsUnavailable,
}
```

## Desired Node States

Each tick, the runtime derives one local desired state from cluster mode and local physical facts:

```rust
enum DesiredNodeState {
    Bootstrap(BootstrapPlan),
    Primary(PrimaryPlan),
    Replica(ReplicaPlan),
    Quiescent(QuiescentReason),
    Fence(FencePlan),
}

enum BootstrapPlan {
    InitDb,
}

enum PrimaryPlan {
    KeepLeader,
    AcquireLeaderThenStartOrPromote,
}

enum ReplicaPlan {
    DirectFollow { leader: MemberId },
    RewindThenFollow { leader: MemberId },
    BasebackupThenFollow { leader: MemberId },
}

enum QuiescentReason {
    WaitingForBootstrapWinner,
    WaitingForAuthoritativeLeader,
    WaitingForFreshQuorum,
    UnsafeUninitializedPgData,
}

enum FencePlan {
    StopAndStayNonWritable,
}
```

These summaries are not persisted back into DCS.

## Deterministic Ordering for Leader Election

When `cluster_initialized = true` and no `/leader` lease exists, candidates rank in strict order:

1. **Matching `system_identifier`** – only members whose `system_identifier` matches `/cluster/identity`
2. **Promotable eligibility** – `state_class` indicates promotable state
3. **Higher `timeline_id`** – larger `timeline_id` preferred
4. **Higher `durable_end_lsn`** – larger LSN preferred
5. **Runtime class** – `running_healthy` over `offline_inspectable`
6. **Lexical `member_id`** – lowest member ID breaks ties

The ranking narrows the race for `/leader`; the leased key remains the final serialization point.

### Bootstrap Race Rules

When `cluster_initialized = false`, only nodes with `data_dir_kind = missing | empty` may race for `/bootstrap`. Nodes with unexpected non-empty `PGDATA` stay quiescent and surface a hard operator error.

## Local Physical Inspection Contract

Cold startup and steady-state HA share a single local physical inspection path. It publishes at least:

| Field | Source | Description |
|-------|--------|-------------|
| `data_dir_kind` | filesystem | `Missing`, `Empty`, `Initialized`, or `InvalidNonEmptyWithoutPgVersion` |
| `system_identifier` | pg_controldata | Control file system identifier |
| `pg_version` | `PG_VERSION` file | Major version number |
| `timeline_id` | pg_controldata | Current timeline identifier |
| `durable_end_lsn` | pg_controldata | `Minimum recovery ending location` or `Latest checkpoint location` |
| `control_file_state` | pg_controldata | Raw cluster state string |
| `was_in_recovery` | pg_controldata | True if control file contains "recovery" |
| `signal_file_state` | filesystem | `standby.signal`, `recovery.signal`, both, or neither |
| `eligible_for_*` | derived | Booleans for bootstrap, follow, rewind, basebackup |

`pg_controldata` output is authoritative. Signal-file inspection is supplemental and must not replace control-file facts.

### Eligibility Flags

```rust
struct LocalPhysicalState {
    // ... other fields ...
    eligible_for_bootstrap: bool,      // true if Missing or Empty
    eligible_for_direct_follow: bool, // true if Initialized without conflicting signals
    eligible_for_rewind: bool,          // true if Initialized without conflicting signals
    eligible_for_basebackup: bool,     // false only if InvalidNonEmptyWithoutPgVersion
}
```

The inspection must be performed with the same PostgreSQL binary version that will run the instance.
