# Verbose context for docs/src/reference/dcs-state-model.md

Top-level DCS concepts from `src/dcs/state.rs`:
- `DcsTrust` variants:
  - `FullQuorum`
  - `FailSafe`
  - `NotTrusted`
- `MemberRole` variants:
  - `Unknown`
  - `Primary`
  - `Replica`
- `MemberRecord` fields:
  - `member_id: MemberId`
  - `postgres_host: String`
  - `postgres_port: u16`
  - `role: MemberRole`
  - `sql: SqlStatus`
  - `readiness: Readiness`
  - `timeline: Option<TimelineId>`
  - `write_lsn: Option<WalLsn>`
  - `replay_lsn: Option<WalLsn>`
  - `updated_at: UnixMillis`
  - `pg_version: Version`
- `LeaderRecord` fields:
  - `member_id: MemberId`
- `SwitchoverRequest` fields:
  - `requested_by: MemberId`
- `InitLockRecord` fields:
  - `holder: MemberId`
- `DcsCache` fields:
  - `members: BTreeMap<MemberId, MemberRecord>`
  - `leader: Option<LeaderRecord>`
  - `switchover: Option<SwitchoverRequest>`
  - `config: RuntimeConfig`
  - `init_lock: Option<InitLockRecord>`
- `DcsState` fields:
  - `worker: WorkerStatus`
  - `trust: DcsTrust`
  - `cache: DcsCache`
  - `last_refresh_at: Option<UnixMillis>`

Required versus optional fields:
- In Rust struct terms, `member_id`, `postgres_host`, `postgres_port`, `role`, `sql`, `readiness`, `updated_at`, and `pg_version` are required fields of `MemberRecord`.
- `timeline`, `write_lsn`, and `replay_lsn` are optional.
- `leader`, `switchover`, and `init_lock` are optional at the cache level.
- `last_refresh_at` is optional at the state level.
- The `config` embedded in `DcsCache` is a full runtime config, not an optional partial overlay.

How trust is evaluated:
- If the backing DCS store is unhealthy, trust becomes `NotTrusted`.
- If the local member is missing from the cache, trust becomes `FailSafe`.
- If the local member record is stale beyond `ha.lease_ttl_ms`, trust becomes `FailSafe`.
- If a leader record exists but the corresponding member record is missing or stale, trust becomes `FailSafe`.
- If there is more than one member in the cache and fewer than two fresh members remain, trust becomes `FailSafe`.
- Otherwise trust is `FullQuorum`.

Freshness definition:
- Freshness is based on `now - updated_at <= cache.config.ha.lease_ttl_ms`.
- That means the DCS cache embeds enough config to interpret leases and member staleness.

How member records are built from local PostgreSQL state:
- Unknown PostgreSQL state produces `role = Unknown`.
- Primary PostgreSQL state produces `role = Primary` and sets `write_lsn`.
- Replica PostgreSQL state produces `role = Replica` and sets `replay_lsn`.
- In all cases the worker writes the local member id, listen host/port, SQL health, readiness, optional timeline, current timestamp, and the publishing PostgreSQL-state version.

Key layout from `src/dcs/keys.rs`:
- Leader key: `/{scope}/leader`
- Switchover key: `/{scope}/switchover`
- Config key: `/{scope}/config`
- Init lock key: `/{scope}/init`
- Member key: `/{scope}/member/{member_id}`

What the runtime config contributes:
- `docker/configs/cluster/node-a/runtime.toml` sets:
  - `dcs.endpoints`
  - `dcs.scope`
  - `ha.lease_ttl_ms`
- Those values matter because:
  - endpoints identify the coordination backend
  - scope namespaces all DCS keys
  - lease TTL controls freshness and therefore trust state

Good reference-page boundaries:
- This page should document state shape and key layout precisely.
- It should avoid drifting into "how to debug" or "how to recover from failures"; those belong in how-to or explanation pages.
- It can cross-link to debug API docs because the debug snapshot exposes DCS-derived state, but the source of truth for field definitions is the Rust state model.
