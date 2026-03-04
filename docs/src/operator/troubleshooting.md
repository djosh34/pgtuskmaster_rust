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
- etcd endpoint instability
- scope mismatch across members
- inconsistent membership/leader view

First checks:
- etcd endpoint health and latency
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
- rewind identity or privileges incorrect
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
- unstable network to etcd
- unstable PostgreSQL readiness signals

First checks:
- `[ha].loop_interval_ms` and `[ha].lease_ttl_ms`
- network path to etcd endpoints
- local PostgreSQL logs and readiness probes

## Why this matters

Symptom-first troubleshooting reduces time-to-diagnosis during incidents. Operators should not need to rebuild the full architecture model before taking the first safe diagnostic steps.

## Tradeoffs

Symptom-first guidance can hide subsystem boundaries if overused. This page therefore includes subsystem cross-links to lifecycle and architecture chapters for deeper cause analysis.

## Cross-links for deeper analysis

- [System Lifecycle](../lifecycle/index.md)
- [Architecture Assurance / Decision Model](../assurance/decision-model.md)
- [Architecture Assurance / DCS Data Model and Write Paths](../assurance/dcs-data-model.md)
