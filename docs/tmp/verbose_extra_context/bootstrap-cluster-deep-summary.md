# Bootstrap cluster deep summary

This support note is only raw factual context for `docs/src/how-to/bootstrap-cluster.md`.
Do not invent steps beyond what the cited source files support.

Bootstrap-relevant configuration surface:

- `RuntimeConfig` includes cluster identity, PostgreSQL settings, DCS settings, HA timings, API, debug, process, and binary paths. The core bootstrap identifiers are `cluster.name`, `cluster.member_id`, `dcs.endpoints`, `dcs.scope`, and the optional `dcs.init` payload. The schema is in `src/config/schema.rs`.
- `dcs.init` is modeled as `DcsInitConfig { payload_json, write_on_bootstrap }`. The schema allows carrying a JSON payload that is written during bootstrap.
- The checked-in Docker runtime example at `docker/configs/cluster/node-a/runtime.toml` shows a normal node configuration with:
  - `cluster.name = "docker-cluster"`
  - `member_id = "node-a"`
  - `dcs.endpoints = ["http://etcd:2379"]`
  - `dcs.scope = "docker-cluster"`
  - `ha.loop_interval_ms = 1000`
  - `ha.lease_ttl_ms = 10000`
  - explicit process binary paths
- Important caveat: that checked-in Docker runtime example does not include a `[dcs.init]` section, so it is not by itself the full zero-state bootstrap recipe.

DCS layout and trust facts relevant to zero-state bootstrap:

- `src/dcs/store.rs` defines DCS key helpers for member records, leader lease, switchover, and config/init writes.
- Relevant keys include:
  - `/{scope}/member/{member_id}`
  - `/{scope}/leader`
  - `/{scope}/switchover`
  - cache/state also tracks `init_lock`
- Leader acquisition is done with create-if-absent semantics, so only one node can establish `/{scope}/leader`.
- `src/dcs/state.rs` evaluates trust this way:
  - `NotTrusted` if etcd is unhealthy
  - `FailSafe` if the local member record is missing or stale
  - `FailSafe` if the leader record is missing from member cache or stale
  - `FailSafe` in multi-member clusters when fewer than two fresh member records remain
  - `FullQuorum` otherwise
- Freshness is bounded by `cache.config.ha.lease_ttl_ms`.

Exact source-backed bootstrap sequence from zero state:

1. Start with an empty DCS scope.
   - The HA startup harness connects to a fresh etcd cluster and later asserts that `/{scope}/init` and `/{scope}/config` appear after bootstrap.
2. Supply `dcs.init` before first-node bootstrap.
   - In the HA E2E startup path, node runtime config is built with `init: Some(DcsInitConfig { payload_json: dcs_init_payload_json, write_on_bootstrap: true })`.
   - The payload written into DCS is a serialized runtime config snapshot.
   - The nested stored payload intentionally sets `"dcs": { ..., "init": null }`, so the written config does not recursively contain the bootstrap-init stanza.
3. Start the first node only.
   - HA begins in `Init`.
   - It transitions to `WaitingPostgresReachable`.
   - After PostgreSQL is reachable or a start attempt completed, it transitions toward `WaitingDcsTrusted`.
   - If there is no follow target and local PostgreSQL is already primary, the node proceeds into leadership acquisition.
4. Wait for the first node to become bootstrap primary.
   - The startup harness explicitly waits only for node index `0` to report bootstrap-primary status before continuing.
5. Verify DCS bootstrap materialization.
   - The harness asserts `/{scope}/init` exists.
   - It asserts `/{scope}/config` exists.
   - It asserts `/{scope}/config` matches `dcs.init.payload_json`.
6. Provision replication and rewind roles on the elected primary before later nodes join.
   - The harness creates or alters the replicator role with `LOGIN REPLICATION`.
   - It creates or alters the rewinder role with `LOGIN SUPERUSER`.
   - This happens after first-node bootstrap and before later nodes are started.

Subsequent node joining patterns:

- When a later node reaches `WaitingDcsTrusted` and another active leader exists, it transitions to replica-follow behavior instead of self-electing.
- Even without an explicit leader lease, a node can follow another healthy primary member record it sees in DCS rather than attempting leadership itself.
- If replica recovery is required after rewind failure or fencing and another leader/primary is known, HA chooses `RecoverReplica` with `BaseBackup`.
- If divergence exists and a leader is known, HA prefers `pg_rewind` first and uses base backup as the fallback after rewind failure.
- The observer in `tests/ha/support/observer.rs` treats more than one primary as split-brain and fails immediately, so docs may safely state that the bootstrap and join flow is expected to maintain at most one primary at a time.

Safe docs caveats to preserve:

- The Docker runtime example is illustrative but incomplete for zero-state bootstrap because it lacks `dcs.init`.
- The strongest source-backed bootstrap description comes from the HA E2E harness and the HA decision code, not from a single checked-in operator example.
