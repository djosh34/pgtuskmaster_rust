## Task: Remove Managed Conf Parse-Back And Re-Derive Start Intent <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove the current pattern where pgtuskmaster reparses its own managed PostgreSQL startup artifacts from `PGDATA` back into typed startup intent. Replace it with a stricter architecture where typed Rust models are the only authoritative internal model, startup intent is re-derived from DCS plus runtime config plus minimal local physical facts, and managed PostgreSQL files are treated as render outputs only.

This task exists because the current codebase drifted into a narrower but still undesirable design: `pgtm.postgresql.conf` and managed signal files became the durable artifact set for resume/rejoin, and startup helper code now parses `primary_conninfo` back into `PgConnInfo`. Repository history shows that this was a deliberate implementation choice for resume/rejoin continuity, but not an inherent PostgreSQL requirement and not evidence that a persisted typed state file is required. The higher-order goal is to restore a coherent controller architecture: pgtuskmaster decides typed startup state fresh, rewrites managed files from that state, and never reparses its own rendered conninfo.

**Scope:**
- Rework startup planning and HA start-intent derivation so they no longer reconstruct `ManagedPostgresStartIntent` from `pgtm.postgresql.conf`.
- Audit every production use of `read_existing_replica_start_intent`, `parse_managed_primary_conninfo`, and `parse_pg_conninfo`.
- Replace parse-back usage with minimal local-fact inspection helpers that answer only the filesystem questions that truly survive restart.
- Prove whether any persisted pgtuskmaster-owned state artifact inside `PGDATA` is still required after the redesign. Do not assume a JSON/YAML sidecar. Only add a tiny persisted marker if code and tests demonstrate that DCS + config + physical local facts are insufficient.
- Update tests and docs so they assert the new architecture instead of the current round-trip helper behavior.
- Remove obsolete parse/render round-trip helpers and tests if they only exist to support the old parse-back design.

**Context from research:**
- The current HA decision stack is already mostly in the right architectural shape:
  - `src/ha/decide.rs` is functional and computes `HaDecision` from `DecisionFacts`
  - `src/ha/lower.rs` lowers `HaDecision` into an effect plan
  - `src/ha/apply.rs` dispatches those effects
- In particular, `decide.rs` does not parse managed config and does not carry conninfo strings around. For startup it emits `HaDecision::WaitForPostgres { start_requested, leader_member_id }`, and the lower/apply layers turn that into `HaAction::StartPostgres`.
- The remaining architectural leak is below the functional decision layer:
  - `src/ha/process_dispatch.rs` `dispatch_process_action(...)` handles `HaAction::StartPostgres`
  - `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)` correctly derives replica intent from DCS when `leader_member_id` is present
  - but if no leader-derived source is available, that same function falls back to `existing_replica_start_intent(...)`
  - `existing_replica_start_intent(...)` calls `src/postgres_managed.rs` `read_existing_replica_start_intent(...)`
  - that helper parses `primary_conninfo` from `pgtm.postgresql.conf` via `src/postgres_managed_conf.rs` `parse_managed_primary_conninfo(...)`
  - which in turn calls `src/pginfo/conninfo.rs` `parse_pg_conninfo(...)`
- That means the current design problem is not in `decide.rs` itself. It is at the boundary where a clean functional HA decision gets converted into concrete startup files and a process job.
- Current parse-back chain:
  - `src/postgres_managed.rs` `read_existing_replica_start_intent(...)`
  - `src/postgres_managed_conf.rs` `parse_managed_primary_conninfo(...)`
  - `src/pginfo/conninfo.rs` `parse_pg_conninfo(...)`
- Current runtime startup path only needs local managed replica state mostly as a consistency/error signal before DCS-derived intent wins:
  - `src/runtime/node.rs` `select_resume_start_intent(...)`
- Current HA dispatch is DCS-first but still falls back to parsed on-disk replica intent:
  - `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)`
- Current managed config writer already regenerates the whole startup surface from typed input:
  - `src/postgres_managed.rs` `materialize_managed_postgres_config(...)`
- Existing local physical facts already available and likely legitimate to keep:
  - `src/postgres_managed.rs` `existing_recovery_signal(...)`
  - `PGDATA` presence / emptiness / initialized-cluster checks in runtime startup planning
- Repository history that must inform the redesign:
  - `.ralph/tasks/story-authoritative-managed-postgres-config/01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer.md`
    - records the explicit question of whether `ResumeExisting` needed a small persisted marker
  - `.ralph/tasks/story-authoritative-managed-postgres-config/03-task-take-full-ownership-of-replica-recovery-signal-and-auto-conf-state.md`
    - resolves the prior design in favor of rereading managed artifacts from disk rather than adding a new sidecar
  - `.ralph/tasks/bugs/postgres-primary-conninfo-password-auth-missing.md`
    - documents the later requirement that startup without a DCS leader could reread previously materialized managed replica state
- Historical evidence found during this investigation:
  - There is strong evidence for persisted managed startup artifacts in `PGDATA`.
  - There is not strong evidence that a full typed sidecar state file is required.
  - A small persisted managed marker was considered historically, but only as a possibility to verify.
  - JSON sidecars found in history were backup-era helper machinery that the project later removed, not a preferred startup-intent design.
- Tests that currently lock in the old helper behavior and may need replacement rather than preservation:
  - `src/postgres_managed.rs` tests around `read_existing_replica_start_intent`
  - `src/postgres_managed_conf.rs` round-trip parsing tests for managed `primary_conninfo`
- Tests that already align more closely with the desired architecture:
  - startup tests in `src/runtime/node.rs` where DCS authority wins and stale local artifacts cause errors rather than silently dictating role

**Expected outcome:**
- No production code reparses pgtuskmaster-rendered `primary_conninfo` into `PgConnInfo`.
- `src/ha/decide.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs` remain purely about facts, decisions, and effect dispatch; they must not gain new startup-intent parsing or disk-derived typed-state reconstruction.
- The lower startup layers must align with the functional HA design: `leader_member_id` from the decision layer remains the only path for deriving replica source conninfo, and absence of an authoritative leader source must no longer trigger managed `.conf` parse-back.
- Startup and HA derive fresh `ManagedPostgresStartIntent` only from DCS, runtime config, and strictly minimal local physical facts.
- Managed PostgreSQL files under `PGDATA` are always rewritten from fresh typed intent before Postgres starts.
- If a persisted marker is still required after the redesign, it is reduced to the smallest possible pgtuskmaster-owned artifact, justified explicitly in code and tests, and does not duplicate rendered conninfo/config state.
- If no persisted marker is required, no new JSON/YAML sidecar is introduced.
- Documentation and tests state clearly that managed `.conf` is a render target, not a typed persistence format.

</description>

<acceptance_criteria>
- [ ] Audit and document every production caller and test that currently depends on `read_existing_replica_start_intent`, `parse_managed_primary_conninfo`, and `parse_pg_conninfo`, distinguishing real behavioral requirements from tests that merely encode the current implementation.
- [ ] `src/runtime/node.rs` is changed so resume/startup planning no longer reconstructs `ManagedPostgresStartIntent` from managed `.conf`; it may only inspect minimal local physical facts and must continue to honor DCS as authoritative for role/source selection.
- [ ] `src/ha/process_dispatch.rs` is changed so `start_intent_from_dcs(...)` no longer falls back to reparsed managed `.conf` as a source of conninfo; any retained local-disk check must be limited to presence/sanity of managed replica artifacts.
- [ ] `src/ha/decide.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs` are reviewed as part of this task and kept aligned with the target architecture: no new local-state parse-back may be introduced below a clean `WaitForPostgres` / `StartPostgres` effect pipeline.
- [ ] `src/postgres_managed.rs` no longer exposes a helper that reads `primary_conninfo` out of `pgtm.postgresql.conf` and converts it back into typed startup intent unless a newly proven minimal persisted marker requires a much smaller read path.
- [ ] `src/postgres_managed.rs` keeps or introduces only the local-fact helpers actually needed after restart, such as managed signal presence or conflicting managed-artifact detection, with explicit error handling and no hidden fallback behavior.
- [ ] `src/postgres_managed_conf.rs` is simplified so parsing helpers for managed `primary_conninfo` are removed if they only supported the old parse-back flow; rendering helpers remain if still needed for managed file generation.
- [ ] `src/pginfo/conninfo.rs` no longer contains production parsing logic for pgtuskmaster’s own rendered conninfo unless a separately justified external-ingest boundary still requires it; any retained parser must be justified by a real non-self-rendering boundary.
- [ ] All affected tests are rewritten to assert the target architecture:
- [ ] startup/resume tests prove fresh DCS-derived intent is used and managed files are regenerated rather than rehydrated from rendered conninfo text
- [ ] local managed artifact residue causes explicit safe failure where required
- [ ] if a small persisted marker is introduced, tests prove exactly why it is necessary and that no larger typed sidecar is needed
- [ ] Historical/docs alignment is updated in the relevant docs and comments so the repository no longer claims both “typed source of truth” and “parse our own rendered managed conninfo.”
- [ ] No new JSON/YAML sidecar is introduced unless the task first proves, with code-level necessity and tests, that DCS + runtime config + local physical facts are insufficient.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
