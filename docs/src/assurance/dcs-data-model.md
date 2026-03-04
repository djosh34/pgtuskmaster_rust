# DCS Data Model and Write Paths

Yes, the docs explicitly include what is written into DCS and how ownership works.

The DCS namespace is scoped under `/<scope>/...` and stores coordination records with clear writers.

## Core keys and writers

| Key pattern | Typical writer | Purpose |
|---|---|---|
| `/<scope>/member/<member_id>` | DCS worker | publish local member state |
| `/<scope>/leader` | HA worker | leader lease / primary coordination |
| `/<scope>/switchover` | Node API (intent), HA worker (clear) | planned transition intent and completion |
| `/<scope>/config` | startup/bootstrap path (optional seed) | cluster config seed record |
| `/<scope>/init` | startup/bootstrap path | initialization lock/coordinator record |

## How writes happen

- DCS worker writes local membership updates as PostgreSQL state changes.
- HA worker updates leader coordination as lifecycle decisions change.
- API writes switchover intent from operator requests.
- HA clears switchover intent when transition handling completes.
- Startup path may seed init/config records before steady-state workers run.

## Why this exists

Explicit ownership prevents hidden coordination side effects. If a record is wrong, operators and contributors can identify which component is responsible.

## Tradeoffs

Central coordination records simplify reasoning but require trust evaluation. If DCS trust degrades, role logic must become more conservative even when local PostgreSQL appears healthy.

## When this matters in operations

When behavior is unexpected, inspect key ownership before assuming broad system failure. For example, stale switchover intent and conflicting leader records have different writers and different corrective paths.
