# Troubleshooting by Symptom

This chapter is organized by what operators usually see first: failed requests, a surprising phase, or a node that is not progressing. Use it for the first pass of diagnosis, then step into the lifecycle and assurance chapters when the symptom clearly maps to a deeper phase or policy question.

## API unreachable or intermittently failing

Start with the simplest split: is the API listener down, or is the API refusing the request on purpose?

Likely causes:

- the node process never finished startup or crashed
- the client is pointed at the wrong `api.listen_addr` or host port
- auth or TLS expectations do not match the server configuration
- the route exists, but the underlying snapshot or DCS-backed action is unavailable

Check in this order:

1. confirm the process and container are still running
2. confirm you are targeting the configured API address and port
3. separate `401`/`403` from `503`
4. inspect recent runtime and API events such as `runtime.startup.entered`, `runtime.startup.mode_selected`, `api.step_once_failed`, `api.tls_handshake_failed`, and `api.tls_client_cert_missing`

Interpretation:

- connection refused usually means bind/startup failure
- `401` or `403` means the security contract is working, even if your client setup is wrong
- `503 snapshot unavailable` means the API worker is alive but cannot serve the required underlying state yet

## Node reports fail-safe unexpectedly

Fail-safe is not an empty error label. It means the runtime no longer considers the shared coordination picture trustworthy enough for normal promotion behavior.

Likely causes:

- etcd bootstrap, connect, or watch-session failures
- scope mismatch across members
- stale or incomplete membership/leader visibility
- mixed faults where PostgreSQL still looks reachable but DCS trust has degraded

Check in this order:

1. read `/ha/state` on more than one node if possible
2. inspect DCS trust-related events such as `dcs.store.health_transition`, `dcs.trust.transition`, `dcs.watch.drain_failed`, and `dcs.watch.refresh_failed`
3. confirm `[dcs].scope` is identical across members
4. inspect leader and member records in the current scope

What usually rules causes in or out:

- one node in fail-safe while peers still show coherent full-quorum state often points at local connectivity or config drift
- all nodes moving into fail-safe together points at a shared coordination outage or partition
- a node that keeps answering `/ha/state` with `FailSafe` is behaving more usefully than a node that disappears entirely

## Switchover request accepted but no transition

The most common misread is assuming `202 Accepted` means the switchover already happened. It means the intent write succeeded. The lifecycle still has to decide whether executing that intent is safe.

Likely causes:

- trust is not at full quorum
- the current primary is not ready to step down safely
- no usable successor is visible yet
- PostgreSQL readiness or process work is still catching up

Check in this order:

1. confirm `switchover_requested_by` is present in `/ha/state`
2. compare trust and phase across the relevant nodes
3. inspect DCS visibility of the switchover record
4. look for HA and process events that show whether the system is waiting, demoting, or blocked

Signals that help:

- `ha_phase = "WaitingSwitchoverSuccessor"` means the current primary already entered a controlled demotion path and is waiting for a safe follower outcome
- unchanged full-quorum state with no demotion activity may mean the request has been recorded but the preconditions are still not satisfied
- trust degradation during the switchover path can legitimately stall or redirect the transition

## Rewind or bootstrap loops

Repeated recovery work usually means the runtime found a real inconsistency and is failing to finish the repair path, not that it is randomly "flapping."

Likely causes:

- rewind identity or password is wrong
- replication authentication rules are incomplete
- the advertised leader endpoint is stale or unreachable
- the previous recovery job failed and the next safe option is now more conservative

Check in this order:

1. inspect the latest HA decision and phase
2. inspect process job outcomes for rewind, base backup, or bootstrap
3. verify `postgres.rewind_conn_identity`, `postgres.roles.rewinder`, and `postgres.roles.replicator`
4. confirm network reachability to the current leader's advertised PostgreSQL endpoint

Interpretation:

- rewind failure followed by bootstrap-oriented behavior can be the intended fallback path
- repeated bootstrap failure often points at auth, binary path, or filesystem prerequisites rather than at DCS policy
- a node that refuses to rejoin is often preserving safety until the recovery evidence becomes coherent

## PostgreSQL started, but runtime behavior does not match the managed config

This symptom usually means PostgreSQL was started outside the managed contract or the managed files were not the files PostgreSQL ended up using.

Likely causes:

- PostgreSQL was started outside `pgtuskmaster` with a different `config_file`
- managed config materialization drifted or failed before startup
- an operator or external automation edited `PGDATA` directly

Check:

- `SHOW config_file;` and confirm it points to `PGDATA/pgtm.postgresql.conf`
- `SHOW hba_file;` and confirm it points to `PGDATA/pgtm.pg_hba.conf`
- `SHOW ident_file;` and confirm it points to `PGDATA/pgtm.pg_ident.conf`
- `SHOW data_directory;`
- startup and process events for materialization failures

If those `SHOW` results point somewhere else, treat that as startup drift or out-of-band interference rather than as normal PostgreSQL behavior under `pgtuskmaster`.

## Leader flaps or repeated role churn

Role churn usually comes from unstable evidence, not from the phase names themselves.

Likely causes:

- overly aggressive timing parameters
- unstable etcd connectivity
- unstable PostgreSQL readiness signals
- a topology where nodes keep disagreeing about who is usable as leader

Check:

- `[ha].loop_interval_ms` and `[ha].lease_ttl_ms`
- etcd connectivity and watch stability
- local PostgreSQL health and readiness behavior
- repeated `ha.phase.transition`, `ha.action.*`, and DCS trust events

What to look for:

- if trust keeps dropping, the problem is often the coordination substrate rather than HA logic
- if PostgreSQL repeatedly becomes unreachable locally, recovery or fail-safe transitions may simply be the consequences of a lower-layer instability
- if leadership attempts recur without convergence, inspect leader-record visibility and scope agreement first

## When to go deeper

Use this page for the first ten minutes of diagnosis. Then move deliberately:

- [System Lifecycle](../lifecycle/index.md) when you need phase semantics
- [Decision Model](../assurance/decision-model.md) when you need to understand which evidence class is blocking progress
- [DCS Data Model and Write Paths](../assurance/dcs-data-model.md) when you need to understand who owns a stale or conflicting record

That progression keeps troubleshooting grounded in the same model the runtime itself uses.
