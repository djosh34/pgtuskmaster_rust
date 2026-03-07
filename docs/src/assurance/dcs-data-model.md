# DCS Data Model and Write Paths

The DCS namespace is scoped under `/<scope>/...` and stores coordination records with clear writers.

## Core keys and writers

| Key pattern | Typical writer | Purpose |
| --- | --- | --- |
| `/<scope>/member/<member_id>` | DCS worker | publish local member state |
| `/<scope>/leader` | HA worker | leader lease and primary coordination |
| `/<scope>/switchover` | Node API (intent), HA worker (clear) | planned transition intent and completion |
| `/<scope>/config` | startup or bootstrap path (optional seed) | cluster config seed record |
| `/<scope>/init` | startup or bootstrap path | initialization lock or coordinator record |

## How to use this during diagnosis

Explicit ownership prevents hidden coordination side effects. If a record is wrong, start with the component that writes it:

- stale member record: DCS worker or its PostgreSQL inputs
- stale leader record: HA worker and lease logic
- stale switchover record: API intent write or HA clear path

That makes it easier to tell the difference between a cluster-wide coordination problem and one broken write path.
