# DCS Module — Design Overview

The **DCS** (Distributed Coordination Store) module is the component that
connects to an external consensus-backed store (currently etcd) and
provides the rest of the system with a consistent, eventually-converging
view of the cluster.

## Responsibilities

| Concern | What the module does for callers |
|---|---|
| **Cluster membership** | Maintains per-member lease records in etcd. Other nodes in the cluster see each other through these records. |
| **Leader election** | Acquires, renews and releases a leader lease in etcd on behalf of the local node. Exposes the current leadership epoch to HA decision-making. |
| **Switchover coordination** | Publishes and clears switchover requests in etcd so every node converges on the same switchover intent. |
| **DCS view projection** | Combines raw etcd state into a `DcsView` snapshot that encodes the coordination mode (`Coordinated`, `Degraded`, `NotTrusted`) and the cluster topology. This view is consumed by the HA worker, the API layer and the CLI. |
| **Postgres state advertisement** | Periodically writes the local node's Postgres state (role, WAL position, readiness) to etcd so that other members can evaluate replication topology. |

## Key Public Types

| Type | Role |
|---|---|
| `DcsView` | Read-only snapshot of the cluster state projected from etcd. Consumed by HA, API and CLI. |
| `DcsHandle` | Write-side handle used by HA and API to request leadership changes and switchover operations. |
| `DcsMode` | Enum (`Coordinated` / `Degraded` / `NotTrusted`) representing the current trust level of the DCS data. |
| `ClusterView` | The set of all known cluster members plus leadership and switchover observations. |
| `ClusterMemberView` | Per-member Postgres state and connection target as seen by the DCS. |
| `MemberPostgresView` | Enum describing what DCS knows about a member's Postgres role (Unknown / Primary / Replica). |
| `LeadershipObservation` | Whether a leader lease is currently held and by whom. |
| `SwitchoverView` | Whether a switchover has been requested and its target. |

## Startup

| Symbol | Purpose |
|---|---|
| `startup::bootstrap()` | Connects to etcd and returns a `DcsRuntime` containing the state subscriber, command handle and background worker. |
| `startup::DcsRuntimeRequest` | Configuration struct consumed by `bootstrap`. |
| `startup::DcsAdvertisedEndpoints` | Resolved Postgres listen address to advertise in etcd. |
| `startup::DcsWorker` | Background task that runs the etcd poll/watch loop. |

## Integration Points

```text
                 ┌────────────┐
                 │   etcd     │
                 └─────┬──────┘
                       │
              ┌────────┴────────┐
              │   DCS worker    │  (poll / watch / lease keep-alive)
              └────────┬────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
  StateSubscriber   DcsHandle    LogSender
   <DcsView>       (commands)    (diagnostics)
        │              │
   ┌────┴───┐     ┌────┴────┐
   │ HA     │     │  API    │
   │ worker │     │  layer  │
   └────────┘     └─────────┘
```

*The DCS worker publishes a `DcsView` through a `StateSubscriber` channel.*
*The HA worker and API layer send commands (acquire/release lease,*
*publish/clear switchover) through the `DcsHandle`.*
