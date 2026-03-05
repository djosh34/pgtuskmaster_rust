# Cluster Restore Takeover Runbook

This runbook describes the operator workflow for forcing a restore takeover on a single executor node and converging the cluster back under normal HA control.

The workflow is **intent-driven**:

1. An operator creates a restore request via the Node API.
2. The executor node observes the request via DCS and runs the restore sequence.
3. Non-executor nodes suppress promotions and fence themselves when needed to avoid split brain.

## Preconditions

- The Node API is reachable on at least one node (the request can be sent to any node).
- API auth is configured if required by your deployment:
  - `POST /restore` and `DELETE /ha/restore` require **Admin** auth.
  - `GET /ha/restore` is a **Read** endpoint.
- `process.binaries.pgbackrest` is configured and points to an executable `pgbackrest` (absolute path).
- `backup.pgbackrest.stanza` and `backup.pgbackrest.repo` are set.
- `[backup.pgbackrest.options].restore` contains your repository configuration (example: `--repo1-path=...`).
- Restore target behavior:
  - If you are restoring into an existing non-empty `postgres.data_dir` on the executor, include `--delta` and `--force=y` in `[backup.pgbackrest.options].restore` (or ensure the executor data directory is empty before the restore starts).

## Trigger the restore takeover

Pick the executor node (the one that will own the restored data directory) and record its `member_id`.

Send:

- `POST /restore`
  - Body:
    - `requested_by`: operator identity (string)
    - `executor_member_id`: node `member_id` that must execute restore
    - `reason`: optional string

The response includes a server-generated `restore_id`. A second concurrent request returns `409 Conflict`.

## Monitor progress

Poll:

- `GET /ha/restore`

Key fields:

- `status.phase`: current restore lifecycle phase
- `status.last_error`: error string (when present)
- `derived.heartbeat_stale`: true if the executor stopped heartbeating (used for orphan detection)

Phases are intended to be operator-meaningful:

- `requested`
- `fencing_primaries`
- `restoring`
- `takeover_managed_config`
- `starting_postgres`
- `waiting_primary`
- terminal: `completed`, `failed`, `cancelled`, `orphaned`

## Completion

When `status.phase=completed`:

1. Verify cluster health using `GET /ha/state`.
2. Clear the restore intent records:
   - `DELETE /ha/restore`

Clearing the records ensures future restore requests are accepted and prevents long-lived “completed restore” records from becoming operationally confusing.

## Failure / rollback

When `status.phase=failed`:

1. Read `status.last_error` and correlate with executor logs:
   - process job logs for `pgbackrest_restore`
   - managed takeover logs (takeover step)
   - postgres start logs
2. Decide whether to:
   - correct configuration/inputs and retry (after `DELETE /ha/restore`), or
   - abandon the restore attempt and return to a known-good HA plan.

Note: `DELETE /ha/restore` clears DCS intent records but does **not** forcibly terminate an in-flight `pgbackrest restore` subprocess if it is already running on the executor.

## Safety notes

- Restore takeover is single-flight by design: only one restore request can exist at a time.
- While a blocking restore request exists, non-executor nodes suppress promotions and fence themselves to reduce split-brain risk.
- Switchover intent is cleared while restore is blocking to avoid post-restore demotion loops.
