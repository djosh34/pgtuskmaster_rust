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

## Structured runtime event logs

pgtuskmaster emits structured log records designed for “operator-grade reconstruction”.

Most runtime records include a small event taxonomy in attributes:

- `event.name`: a stable event identifier (machine-friendly)
- `event.domain`: subsystem (for example `runtime`, `ha`, `process`, `api`, `dcs`, `pginfo`, `postgres_ingest`)
- `event.result`: outcome label (`ok`, `failed`, `started`, `recovered`, `skipped`, etc.)

Common correlation attributes:

- **Node identity:** `scope`, `member_id`
- **Startup:** `startup_run_id`
- **HA:** `ha_tick`, `ha_dispatch_seq`, `action_id`, `action_kind`, `action_index`
- **Process jobs:** `job_id`, `job_kind`, `binary`
- **Subprocess output:** `stream` (`stdout|stderr`) when captured
- **API:** `api.peer_addr`, `api.method`, `api.route_template`, `api.status_code`

Recommended operator workflow:

1. Start from the high-level lifecycle markers (`runtime.startup.*`, `ha.phase.transition`).
2. Correlate intent → dispatch → outcome:
   - `ha.action.intent` → `ha.action.dispatch` → `ha.action.result`
   - `process.job.started` → `process.job.exited|process.job.timeout|process.job.poll_failed`
3. Use warning/error events as the “why” layer:
   - DCS: `dcs.local_member.write_failed`, `dcs.watch.*_failed`
   - PgInfo: `pginfo.poll_failed`, `pginfo.sql_transition`
   - API: `api.step_once_failed`, `api.tls_handshake_failed`

## Backup / restore observability

pgtuskmaster runs backup/restore operations as PgTool subprocess jobs and captures stdout/stderr into log records. Use those PgTool records (for example `job_kind=pgbackrest_restore`) plus the local PostgreSQL logs to debug recovery bootstrap and backup failures.

When `backup.enabled=true`, pgtuskmaster also *owns* Postgres `archive_command` / `restore_command` wiring at startup by injecting settings that invoke the `pgtuskmaster wal ...` helper (which runs pgBackRest `archive-push` / `archive-get` under the hood).

- The helper’s stdout/stderr is emitted as part of PostgreSQL logs (because Postgres executes `archive_command` / `restore_command`).
- pgtuskmaster does not yet emit a first-class structured “archive invocation” event stream of its own; rely on the Postgres logs + ingest diagnostics for now.

## Postgres log ingest health

The Postgres ingest worker tails configured inputs and emits internal diagnostic records when ingestion or cleanup encounters errors (instead of failing silently).

What to look for in logs:

- internal log records with `event.domain=postgres_ingest` (and usually `origin=postgres_ingest::...`)
- event identifiers like:
  - `postgres_ingest.iteration` (debug breadcrumbs)
  - `postgres_ingest.step_once_failed` (warn/error)
  - `postgres_ingest.recovered` (recovery marker)
- stable tags in the message payload: `stage=... kind=... path=...`
- `suppressed=N` when repeated identical failures are rate-limited
