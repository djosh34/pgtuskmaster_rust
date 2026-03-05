# Observability and Day-2 Operations

Operational confidence depends on three simultaneous views: local PostgreSQL state, DCS trust and cache state, and HA decision output.

```mermaid
flowchart LR
  PG[(PostgreSQL)] --> PgView[PgInfo view]
  DCS[(DCS)] --> DcsView[DCS cache + trust]
  PgView --> HA[HA decision]
  DcsView --> HA
  HA --> State[API state]
  HA --> Action[Process actions]
```

## Why this exists

No single surface explains HA behavior. Correlate `/ha/state` with debug payloads (like `/debug/verbose` when enabled), recent logs, and DCS record views rather than relying on any single view.

## Tradeoffs

Richer observability creates more data to read. The benefit is that operators can reconstruct decision context without guessing hidden state.

## When this matters in operations

When a node appears "stuck," you need to determine whether it is unhealthy, waiting on trust, or blocked on a safety precondition. These cases look similar from a distance but require different responses.

## Day-2 operator routine

- Check `/ha/state` for current phase and trust posture.
- Correlate with recent logs around phase transitions and action attempts.
- Inspect DCS records for leader and switchover intent coherence.
- Validate PostgreSQL reachability and readiness on the local node.
- If behavior is conservative, confirm whether trust degradation is the trigger.

## Useful command surfaces

```console
pgtuskmasterctl ha state
pgtuskmasterctl switchover --to <member-id>
pgtuskmasterctl switchover cancel
```

Use planned switchover workflows for controlled role transitions. Avoid ad-hoc out-of-band interventions unless the documented lifecycle path is confirmed blocked.

## Backup / restore observability

pgtuskmaster runs backup/restore operations as PgTool subprocess jobs and captures stdout/stderr into log records. Use those PgTool records (for example `job_kind=pgbackrest_restore`) plus the local PostgreSQL logs to debug recovery bootstrap and backup failures.

pgtuskmaster does not currently generate its own structured `archive_command` / `restore_command` invocation events, and it does not manage Postgres `archive_command` / `restore_command` wiring in this version.

## Postgres log ingest health

The Postgres ingest worker tails configured inputs and emits internal diagnostic records when ingestion or cleanup encounters errors (instead of failing silently).

What to look for in logs:

- internal log records with `origin=postgres_ingest`
- stable tags in the message payload: `stage=... kind=... path=...`
- `suppressed=N` when repeated identical failures are rate-limited
