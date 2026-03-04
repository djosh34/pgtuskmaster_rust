---
## Task: Build pgBackRest JSON observability with metrics, logs, and API visibility <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Ingest pgBackRest JSON output natively and expose backup/restore observability through structured logs, internal state snapshots, metrics/OTel signals, and API endpoints.

**Scope:**
- Parse pgBackRest JSON output for backup/info/check/restore operations and map it into typed Rust structures.
- Persist latest backup status/history and restore diagnostics in runtime state for operator/API/debug visibility.
- Expose key backup telemetry:
- last successful backup timestamp/label/type
- last failed backup timestamp/error summary
- per-run logs and exit status
- staleness indicators (no successful backup in expected interval)
- Integrate telemetry into current structured logging and OTel/metrics export path (or introduce one where missing).

**Context from research:**
- pgBackRest command output explicitly supports JSON (`--output=json`) and recommends this for machine integration: https://pgbackrest.org/command.html
- pgBackRest info/check/restore logs already include structured details that should be captured as first-class attributes, not raw line scraping: https://pgbackrest.org/user-guide.html
- Existing project already has structured log ingestion and debug snapshot infrastructure to extend (`src/logging/*`, `src/debug_api/*`).

**Expected outcome:**
- Operators can query API/debug endpoints and logs to answer: "when did backups last succeed?", "what failed?", and "what restore step is stuck?".
- Runtime can use typed backup state for decisioning without parsing fragile text.
- Backup observability becomes consistent across command output, API payloads, and telemetry streams.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add typed pgBackRest JSON models/parser module(s), e.g. `src/backup/pgbackrest_json.rs`, with:
- [ ] strict deserialization for expected command output shape
- [ ] tolerant handling for optional/forward-compatible fields
- [ ] no lossy parsing of essential backup identity/timestamps/status fields
- [ ] Add/extend backup runtime state module(s), e.g. `src/backup/state.rs`:
- [ ] track last success/last failure/current operation/history tail
- [ ] include operation id, start/end timestamps, duration, status, and normalized error
- [ ] Extend logging pipeline in `src/logging/mod.rs`, `src/logging/postgres_ingest.rs`, and related command execution call sites:
- [ ] emit structured backup/restore records with stable keys
- [ ] include command identity, stanza, backup label/type, and result
- [ ] keep stdout/stderr capture for failed commands for operator diagnosis
- [ ] Extend API/debug surfaces (`src/api/controller.rs`, `src/api/worker.rs`, `src/debug_api/snapshot.rs`, `src/debug_api/view.rs`) with backup observability endpoints/views
- [ ] Define metrics/OTel emission points for:
- [ ] backup success/failure counters
- [ ] backup duration histograms/timers
- [ ] last successful backup timestamp gauge
- [ ] restore attempt/success/failure counters
- [ ] Add unit/integration tests for:
- [ ] pgBackRest JSON parser happy-path and malformed/partial payloads
- [ ] state transitions on success/failure/retry
- [ ] API payload correctness and auth behavior for backup-status endpoints
- [ ] telemetry emission on success/failure
- [ ] Update docs with telemetry contract and troubleshooting fields in `docs/src/operator/`
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
