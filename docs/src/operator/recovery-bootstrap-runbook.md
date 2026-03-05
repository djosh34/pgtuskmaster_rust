# Recovery Bootstrap Runbook

This runbook describes the intended operator workflow for bringing up a new cluster from a pgBackRest backup using `backup.bootstrap`.

## Preconditions

- `backup.enabled = true`
- `backup.bootstrap.enabled = true`
- `process.binaries.pgbackrest` points to an executable `pgbackrest`
- `backup.pgbackrest.stanza` and `backup.pgbackrest.repo` are set
- pgBackRest repository configuration is provided via per-operation options (for example `--repo1-path=...`)

## What pgtuskmaster does during restore bootstrap

When `postgres.data_dir` is `Missing|Empty` and the cluster is uninitialized (no DCS init lock), pgtuskmaster selects a restore bootstrap path:

1. Runs `pgbackrest restore` (captured as a PgTool subprocess job).
2. Performs a deterministic takeover of backup-era artifacts in `PGDATA`:
   - removes/quarantines `postgresql.conf` and `postgresql.auto.conf`
   - removes/quarantines `pg_hba.conf` and `pg_ident.conf`
   - removes/quarantines any existing `recovery.signal` / `standby.signal`
   - removes/quarantines stale `pgtm.*` managed artifacts
3. Starts PostgreSQL using a pgtuskmaster-owned config file via `-c config_file=PGDATA/pgtm.postgresql.conf`.

## Expected log sequence

Use the runtime logs plus PgTool subprocess output to confirm progress:

- Runtime startup markers: `startup phase=restore`, then `startup phase=takeover`, then `startup phase=start`
- PgTool output records for `job_kind=pgbackrest_restore` and `job_kind=start_postgres`

## Common failure cases and next actions

### pgBackRest restore fails immediately

Check:

- `process.binaries.pgbackrest` path/executability
- `backup.pgbackrest.stanza` / `backup.pgbackrest.repo`
- `[backup.pgbackrest.options].restore` contains the repo configuration you expect (example: `--repo1-path=...`)

### PostgreSQL starts but recovery stalls

Check:

- Postgres logs for repeated WAL restore attempts and error signatures
- PgTool output records for `job_kind=pgbackrest_restore` and `job_kind=start_postgres` (stderr content is captured)

### Postgres start fails with unexpected settings

Check:

- that takeover ran (look for quarantine directory `PGDATA/pgtm.quarantine.*` when takeover policy is `quarantine`)
- that `PGDATA/postgresql.auto.conf` was removed/quarantined

## Safety notes

- `backup.bootstrap` is intentionally explicit. Do not enable it unless you intend restore bootstrap to be a valid initialization path for the node.
- The takeover step is designed to prevent unsafe “backup-era config resurrection” (especially `postgresql.auto.conf`).
