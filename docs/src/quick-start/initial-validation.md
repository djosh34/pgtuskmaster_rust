# Initial Validation

After first startup, validate observable behavior before you treat the setup as operationally ready.

## Validation checklist

- API reachability: `GET /ha/state` responds consistently.
- Trust visibility: reported trust level aligns with current etcd health.
- PostgreSQL role visibility: current phase and role are understandable.
- DCS coherence: scope keys exist and reflect expected membership/leader intent.
- Logs clarity: startup and steady-state transitions appear with useful context.

## What "good" looks like

Good initial validation means that state transitions are explainable. If trust degrades, the node reports conservative behavior. If trust is healthy, normal role progression is visible.

## Common first-run issues

- Missing binaries in `process.binaries`
- Backup enabled without `process.binaries.pgbackrest` and `[backup.pgbackrest]` stanza/repo
- Unreadable secret files
- Incorrect `pg_hba` for replication paths
- etcd endpoint mismatch or scope mismatch
- Directory permissions that prevent PostgreSQL startup

After this checklist, continue with **Operator Guide** for production profile selection and full field-level configuration reasoning.
