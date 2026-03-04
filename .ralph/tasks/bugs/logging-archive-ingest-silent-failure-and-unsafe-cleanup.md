---
## Bug: Archive/restore ingest silently fails and cleanup/path ownership can destroy active observability signals <status>not_started</status> <passes>false</passes>

<description>
Archive/restore observability has several correctness failures in the current logging pipeline:

1) `postgres_ingest::run()` suppresses errors from wrapper creation and ingest steps (`ensure_archive_wrapper`, `step_once`, and `cleanup_log_dir`), so failures are silent and operators lose telemetry without actionable diagnostics.
2) `cleanup_log_dir()` only protects `pg_ctl_log_file` and can delete currently active files in `logging.postgres.log_dir` (e.g. active `postgres.json`, `postgres.stderr.log`, and potentially archive JSON if colocated), causing dropped logs.
3) No path ownership validation prevents sink/source overlap (for example `logging.sinks.file.path` overlapping any tailed postgres/archive file), which can create recursive self-ingestion loops and log amplification.

Please investigate the full flow first, then implement a fix that is explicit about ownership boundaries and failure surfacing.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
