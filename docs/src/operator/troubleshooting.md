# Troubleshooting by Symptom

This chapter is organized by what operators typically see first: errors, symptoms, and log signatures.

## API unreachable or intermittently failing

Likely causes:
- node process not running or failed at startup
- listen address mismatch
- auth/tls mismatch between client and API policy

First checks:
- runtime process status
- configured `api.listen_addr`
- API security settings versus client expectations

## Node reports fail-safe unexpectedly

Likely causes:
- etcd bootstrap/connect/watch session setup failures or timeouts
- scope mismatch across members
- inconsistent membership/leader view

First checks:
- etcd transport/connect stability and timeouts
- `[dcs].scope` consistency on all nodes
- leader/member records in current scope

## Switchover request accepted but no transition

Likely causes:
- safety preconditions not met
- trust not at full quorum
- target node not eligible or not healthy

First checks:
- `/ha/state` phase and trust on relevant nodes
- DCS switchover intent visibility
- PostgreSQL readiness on current and target nodes

## Rewind/bootstrap loops

Likely causes:
- rewind identity/auth misconfigured or database privileges insufficient
- replication auth rules incomplete
- source host/port for rewind is invalid

First checks:
- `postgres.rewind_conn_identity`
- `postgres.roles.rewinder`
- `pg_hba` replication rules
- connectivity to `rewind_source_host:rewind_source_port`

## Restore/recovery bootstrap failures

Likely causes:
- `backup.bootstrap.enabled = true` but pgBackRest is not fully configured (missing `process.binaries.pgbackrest`, missing `backup.pgbackrest.stanza/repo`, or missing repo configuration in pgBackRest options)
- backup-era config artifacts interfering with a managed start (should be quarantined/deleted by takeover; if not, check takeover logs)
- missing/incorrect `logging.postgres.archive_command_log_file` (required when backup is enabled; restore/archive events are written there)

First checks:
- config validation errors on startup (they include stable field paths)
- `logging.postgres.archive_command_log_file` contents:
  - look for JSON lines with `backup.event_kind = archive_get|archive_push`
  - correlate by `backup.invocation_id` and `backup.status_code`
- PgTool subprocess logs (`job_kind=start_postgres|pgbackrest_restore`) for stderr output

## Leader flaps or repeated role churn

Likely causes:
- overly aggressive timing parameters
- unstable etcd connectivity (watch invalidation and reconnect snapshots)
- unstable PostgreSQL readiness signals

First checks:
- `[ha].loop_interval_ms` and `[ha].lease_ttl_ms`
- etcd connectivity to configured endpoints (or per-node proxy endpoints, if used)
- local PostgreSQL logs and readiness probes

## Why this matters

Symptom-first troubleshooting reduces time-to-diagnosis during incidents. Operators should not need to rebuild the full architecture model before taking the first safe diagnostic steps.

## Tradeoffs

Symptom-first guidance can hide subsystem boundaries if overused. This page therefore includes subsystem cross-links to lifecycle and architecture chapters for deeper cause analysis.

## Cross-links for deeper analysis

- [System Lifecycle](../lifecycle/index.md)
- [Architecture Assurance / Decision Model](../assurance/decision-model.md)
- [Architecture Assurance / DCS Data Model and Write Paths](../assurance/dcs-data-model.md)
