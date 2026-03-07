# Troubleshooting by Symptom

This chapter is organized by what operators usually see first: failed requests, a surprising phase, or a node that is not progressing.

## API unreachable or intermittently failing

Likely causes:

- the node process is not running or failed at startup
- the client is pointed at the wrong `api.listen_addr`
- auth or TLS expectations do not match the server configuration

First checks:

- runtime process status
- configured `api.listen_addr`
- API security settings versus client expectations
- recent runtime and API events such as `runtime.startup.entered`, `runtime.startup.mode_selected`, `api.step_once_failed`, `api.tls_handshake_failed`, and `api.tls_client_cert_missing`

## Node reports fail-safe unexpectedly

Likely causes:

- etcd bootstrap, connect, or watch-session failures
- scope mismatch across members
- inconsistent membership or leader view

First checks:

- etcd transport stability and timeouts
- `[dcs].scope` consistency on all nodes
- leader and member records in the current scope
- DCS trust and health transitions such as `dcs.store.health_transition`, `dcs.trust.transition`, `dcs.watch.drain_failed`, and `dcs.watch.refresh_failed`

## Switchover request accepted but no transition

Likely causes:

- safety preconditions are not met yet
- trust is not at full quorum
- the target node is not healthy or not eligible

First checks:

- `/ha/state` phase and trust on the relevant nodes
- DCS visibility of the switchover record
- PostgreSQL readiness on the current and target nodes
- HA and process correlation through `ha.phase.transition`, `ha.role.transition`, `ha.action.intent`, `ha.action.dispatch`, `ha.action.result`, `process.job.started`, `process.job.exited`, and `process.job.timeout`

## Rewind or bootstrap loops

Likely causes:

- rewind identity or password is wrong
- replication authentication rules are incomplete
- the advertised leader endpoint in DCS is stale or unreachable

First checks:

- `postgres.rewind_conn_identity`
- `postgres.roles.rewinder`
- `pg_hba` replication rules
- current leader and member records in DCS plus network reachability to the advertised PostgreSQL endpoint

## PostgreSQL started, but runtime behavior does not match the managed config

Likely causes:

- PostgreSQL was started outside pgtuskmaster with a different `config_file`
- managed config materialization drifted or failed before startup
- an operator or external automation edited `PGDATA` directly

First checks:

- `SHOW config_file;` and confirm it points to `PGDATA/pgtm.postgresql.conf`
- `SHOW hba_file;` and confirm it points to `PGDATA/pgtm.pg_hba.conf`
- `SHOW ident_file;` and confirm it points to `PGDATA/pgtm.pg_ident.conf`
- `SHOW data_directory;`
- recent startup and process events for materialization failures

If those `SHOW` results point somewhere else, treat that as startup drift or out-of-band interference rather than as normal PostgreSQL behavior under pgtuskmaster.

## Leader flaps or repeated role churn

Likely causes:

- overly aggressive timing parameters
- unstable etcd connectivity
- unstable PostgreSQL readiness signals

First checks:

- `[ha].loop_interval_ms` and `[ha].lease_ttl_ms`
- etcd connectivity to configured endpoints
- local PostgreSQL logs and readiness probes

## When to go deeper

Use this page for the first ten minutes of diagnosis. If the symptom does not resolve into an obvious config, trust, or process problem, move to:

- [System Lifecycle](../lifecycle/index.md)
- [Decision Model](../assurance/decision-model.md)
- [DCS Data Model and Write Paths](../assurance/dcs-data-model.md)
