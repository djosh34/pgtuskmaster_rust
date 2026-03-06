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
- recent runtime/API events:
  - `runtime.startup.entered` / `runtime.startup.mode_selected` to confirm startup completed
  - `api.step_once_failed` (warn/error) for request-loop failures
  - `api.tls_handshake_failed` / `api.tls_client_cert_missing` for TLS/mTLS policy mismatches

## Node reports fail-safe unexpectedly

Likely causes:
- etcd bootstrap/connect/watch session setup failures or timeouts
- scope mismatch across members
- inconsistent membership/leader view

First checks:
- etcd transport/connect stability and timeouts
- `[dcs].scope` consistency on all nodes
- leader/member records in current scope
- DCS trust/health transitions:
  - `dcs.store.health_transition` (recovered/failed)
  - `dcs.trust.transition` (fullquorum/failsafe/nottrusted)
  - `dcs.watch.drain_failed` / `dcs.watch.refresh_failed` / `dcs.watch.apply_had_errors`

## Switchover request accepted but no transition

Likely causes:
- safety preconditions not met
- trust not at full quorum
- target node not eligible or not healthy

First checks:
- `/ha/state` phase and trust on relevant nodes
- DCS switchover intent visibility
- PostgreSQL readiness on current and target nodes
- HA + process correlation:
  - `ha.phase.transition` and `ha.role.transition` to see where progression stops
  - `ha.action.intent` / `ha.action.dispatch` / `ha.action.result` for per-action outcomes
  - `process.job.started` / `process.job.exited|process.job.timeout` for side-effect execution

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
