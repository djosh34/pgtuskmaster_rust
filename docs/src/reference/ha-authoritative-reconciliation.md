# HA Authority Reconciliation

This page freezes the authority model for the HA startup, bootstrap, and rejoin rewrite while the broader HA state-machine replacement is still in progress.

It is intentionally narrower and more implementation-facing than the operator guides.

## DCS authority facts

The runtime uses four authoritative fact families under `/{scope}`:

| Path | Kind | Meaning |
| --- | --- | --- |
| `/cluster/initialized` | durable | Written only after successful first bootstrap |
| `/cluster/identity` | durable | Authoritative cluster `system_identifier`, bootstrap winner, and bootstrap completion time |
| `/bootstrap` | leased | Held only while one node actively owns first-cluster bootstrap |
| `/leader` | leased | Held only by the current elected leader |

Member observations remain under `/member/{member_id}`.

Member records are still the live observation surface and now also carry the pre-election descriptor used for offline candidate ranking.

## Trust vocabulary

`DcsTrust` is intentionally compressed to three authority states:

- `NotTrusted`: the backing store is unavailable or unhealthy
- `NoFreshQuorum`: the store is reachable, but there is not enough fresh authority to elect or preserve a writable primary
- `FreshQuorum`: the store is healthy and fresh enough for bootstrap and leader-election decisions

Startup and steady-state HA obey the same rule: an initialized cluster may only run writable primary behavior while trust is `FreshQuorum`.

## Derived cluster and node plans

Each tick, the runtime derives one cluster summary and one local desired state from DCS facts plus local physical facts.

These summaries are computed, not persisted.

### Cluster summary

- `UninitializedNoBootstrapOwner`
- `UninitializedBootstrapInProgress { holder }`
- `InitializedLeaderPresent { leader }`
- `InitializedNoLeaderFreshQuorum`
- `InitializedNoLeaderNoFreshQuorum`
- `DcsUnavailable`

### Desired node state

- `Bootstrap(InitDb)`
- `Primary(KeepLeader)`
- `Primary(AcquireLeaderThenStartOrPromote)`
- `Replica(DirectFollow)`
- `Replica(RewindThenFollow)`
- `Replica(BasebackupThenFollow)`
- `Quiescent(WaitingForBootstrapWinner)`
- `Quiescent(WaitingForAuthoritativeLeader)`
- `Quiescent(WaitingForFreshQuorum)`
- `Quiescent(UnsafeUninitializedPgData)`
- `Fence(StopAndStayNonWritable)`

The top-level shape is intentionally compressed:

- few node states
- replica convergence represented as one ordered reconciliation family
- no separate startup-only authority model

## Deterministic ordering

When `cluster_initialized = true` and no authoritative leader lease exists, candidates are ranked in this exact order:

1. matching expected cluster `system_identifier`
2. promotable and otherwise eligible state
3. higher `timeline_id`
4. higher `durable_end_lsn`
5. `postgres_runtime_class = running_healthy` over `offline_inspectable`
6. lowest lexical `member_id`

That ordering narrows which node may race for `/leader`, but the leased leader key remains the final serialization point.

When `cluster_initialized = false`, only nodes with `data_dir_kind = Missing | Empty` may race for `/bootstrap`.

Nodes with unexpected non-empty `PGDATA` stay quiescent and surface a hard operator error.

## Local physical inspection contract

Cold startup and steady-state HA share one authoritative local physical inspection path.

It publishes at least:

| Field | Source |
| --- | --- |
| `data_dir_kind` | filesystem shape |
| `system_identifier` | `pg_controldata` |
| `pg_version` | `PG_VERSION` |
| `timeline_id` | `pg_controldata` |
| `durable_end_lsn` | `pg_controldata` |
| `control_file_state` | `pg_controldata` |
| `was_in_recovery` | `pg_controldata` |
| `signal_file_state` | standby/recovery signal files |
| `eligible_for_bootstrap` | derived from inspected facts |
| `eligible_for_direct_follow` | derived from inspected facts |
| `eligible_for_rewind` | derived from inspected facts |
| `eligible_for_basebackup` | derived from inspected facts |

`pg_controldata` output is the authoritative source for control-file facts.

Signal-file inspection is supplemental and must not replace control-file facts.
