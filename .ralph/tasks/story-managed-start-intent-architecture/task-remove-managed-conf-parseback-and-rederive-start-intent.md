## Task: Remove Managed Conf Parse-Back And Re-Derive Start Intent <status>completed</status> <passes>true</passes>

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
- [x] Audit and document every production caller and test that currently depends on `read_existing_replica_start_intent`, `parse_managed_primary_conninfo`, and `parse_pg_conninfo`, distinguishing real behavioral requirements from tests that merely encode the current implementation.
- [x] `src/runtime/node.rs` is changed so resume/startup planning no longer reconstructs `ManagedPostgresStartIntent` from managed `.conf`; it may only inspect minimal local physical facts and must continue to honor DCS as authoritative for role/source selection.
- [x] `src/ha/process_dispatch.rs` is changed so `start_intent_from_dcs(...)` no longer falls back to reparsed managed `.conf` as a source of conninfo; any retained local-disk check must be limited to presence/sanity of managed replica artifacts.
- [x] `src/ha/decide.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs` are reviewed as part of this task and kept aligned with the target architecture: no new local-state parse-back may be introduced below a clean `WaitForPostgres` / `StartPostgres` effect pipeline.
- [x] `src/postgres_managed.rs` no longer exposes a helper that reads `primary_conninfo` out of `pgtm.postgresql.conf` and converts it back into typed startup intent unless a newly proven minimal persisted marker requires a much smaller read path.
- [x] `src/postgres_managed.rs` keeps or introduces only the local-fact helpers actually needed after restart, such as managed signal presence or conflicting managed-artifact detection, with explicit error handling and no hidden fallback behavior.
- [x] `src/postgres_managed_conf.rs` is simplified so parsing helpers for managed `primary_conninfo` are removed if they only supported the old parse-back flow; rendering helpers remain if still needed for managed file generation.
- [x] `src/pginfo/conninfo.rs` no longer contains production parsing logic for pgtuskmaster’s own rendered conninfo unless a separately justified external-ingest boundary still requires it; any retained parser must be justified by a real non-self-rendering boundary.
- [x] All affected tests are rewritten to assert the target architecture:
- [x] startup/resume tests prove fresh DCS-derived intent is used and managed files are regenerated rather than rehydrated from rendered conninfo text
- [x] local managed artifact residue causes explicit safe failure where required
- [x] if a small persisted marker is introduced, tests prove exactly why it is necessary and that no larger typed sidecar is needed
- [x] Historical/docs alignment is updated in the relevant docs and comments so the repository no longer claims both “typed source of truth” and “parse our own rendered managed conninfo.”
- [x] No new JSON/YAML sidecar is introduced unless the task first proves, with code-level necessity and tests, that DCS + runtime config + local physical facts are insufficient.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Phase 1: Audit the current parse-back boundary and lock the replacement scope

- [x] Re-audit the production call sites in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) and [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs) to confirm exactly what behavior they require from local disk after restart.
- [x] Re-audit the helper chain in [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs), [src/postgres_managed_conf.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs), and [src/pginfo/conninfo.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/conninfo.rs) and classify each use as either a real boundary or a self-rendered parse-back artifact.
- [x] Perform a repo-wide caller audit before deleting `parse_pg_conninfo(...)`. Current evidence says the parser is only used by the managed parse-back path, but execution must verify there is no remaining non-managed external-ingest boundary that still justifies keeping it.
- [x] Record the audit result directly in this task file as execution notes when implementation is done, distinguishing production requirements from tests that only preserve the old implementation shape.
- [x] Treat `docs/tmp/` generated prompt snapshots as derived artifacts, not primary docs sources, unless a real checked-in docs page or code comment needs to be updated.

### Phase 2: Replace typed intent reconstruction with minimal local physical facts

- [x] Remove the production helper that reconstructs `ManagedPostgresStartIntent` from `pgtm.postgresql.conf`; specifically, delete or replace `read_existing_replica_start_intent(...)` in [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs).
- [x] Introduce a narrower local-state helper in [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) that answers only the physical questions that genuinely survive restart:
  - whether managed recovery signal state exists
  - whether the managed recovery signal files are conflicting or invalid
  - optionally whether other managed artifacts required for consistency checking are present, but never their parsed conninfo contents
- [x] Prefer returning a small fact struct or enum that can be shared by runtime startup and HA dispatch, so both layers enforce the same residue/safety semantics instead of duplicating ad hoc signal checks.
- [x] Keep that helper small and fact-oriented. It must not read `primary_conninfo`, must not parse conninfo text, and must not synthesize upstream host/user/port state from managed files.
- [x] Preserve explicit error behavior for conflicting `standby.signal` and `recovery.signal`, because that is a valid local-disk safety check independent of parse-back.
- [x] Do not introduce a JSON/YAML sidecar or any new typed persistence artifact unless implementation proves an unavoidable gap that cannot be solved from DCS, runtime config, and local physical facts alone.

### Phase 3: Re-derive startup intent only from DCS plus runtime config

- [x] Update `select_resume_start_intent(...)` in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) to inspect only the new minimal local-fact helper instead of reconstructing replica conninfo from managed config.
- [x] Preserve the current authority ordering in runtime startup:
  - DCS leader or healthy foreign primary remains authoritative for building replica intent
  - local node leadership or primary membership still yields `ManagedPostgresStartIntent::primary()`
  - local managed replica residue without enough DCS authority remains an explicit safe failure rather than a silent fallback
- [x] Ensure runtime resume behavior is phrased and implemented as “rebuild authoritative intent now” rather than “rehydrate previously rendered intent from disk.”
- [x] Recheck whether any startup path still needs a persisted marker after this rewrite. The default assumption for execution is no new marker; only reverse that if code-level necessity is demonstrated during implementation.

### Phase 4: Align HA process dispatch with the same boundary

- [x] Update `start_intent_from_dcs(...)` in [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs) so `leader_member_id` is the only source for deriving replica conninfo.
- [x] Remove the fallback that currently calls `existing_replica_start_intent(...)` and returns a typed replica intent parsed from managed files.
- [x] Replace that fallback with behavior based only on minimal local facts:
  - if there is no leader-derived source and no conflicting managed replica residue, return primary start intent as before
  - if there is managed replica residue but no authoritative leader source, fail explicitly instead of silently trusting old rendered config
- [x] Review nearby code in [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs), [src/ha/lower.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/lower.rs), and [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs) to verify no new parse-back or typed on-disk reconstruction is introduced while making the dispatch change.

### Phase 5: Remove obsolete parsing code and replace tests with architectural tests

- [x] Delete `parse_managed_primary_conninfo(...)` and its supporting managed-primary-conninfo parse types from [src/postgres_managed_conf.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs) if they are left with no real boundary after the runtime and HA rewrites.
- [x] Delete `parse_pg_conninfo(...)` and its parser-only error/test machinery from [src/pginfo/conninfo.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/conninfo.rs) only if the Phase 1 repo-wide audit confirms it has no remaining non-self-rendered callers; otherwise narrow this task to removing only the managed parse-back dependency and leave any justified external-ingest parser in place with updated documentation.
- [x] Keep render helpers that are still needed for producing managed config and observed state text.
- [x] Replace parse-round-trip tests with behavior tests that prove the target architecture instead:
  - runtime resume prefers fresh DCS-derived replica intent even when stale local managed config exists
  - runtime resume rejects managed replica residue when DCS authority is unavailable
  - HA process dispatch does not rebuild replica intent from local managed `primary_conninfo`
  - minimal local-fact helpers still catch conflicting managed recovery signals and any other retained safety checks
- [x] Remove tests whose only purpose was validating the old parse-back helpers.

### Phase 6: Update documentation and comments to match the new architecture

- [x] Update the relevant checked-in docs and code comments so they consistently state that managed PostgreSQL files under `PGDATA` are render outputs, not typed persistence inputs.
- [x] If the user’s requested `update-docs` skill is still unavailable in this session, perform the docs changes directly in the repository and note that limitation in the execution notes.
- [x] Remove or tighten any stale wording that implies pgtuskmaster reparses its own rendered managed conninfo on resume or rejoin.

### Phase 7: Execute the full verification gate before completion

- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test-long`.
- [x] Run `make lint`.
- [x] If the repo-wide audit shows `parse_pg_conninfo(...)` must survive for an unrelated boundary, document that explicitly in the execution notes so the task closes the architectural leak without overstating parser removal.
- [x] Only after the code, tests, and docs are complete, tick the acceptance criteria and set `<passes>true</passes>`.
- [x] Append execution notes here summarizing the final architecture decision, whether a persisted marker was needed, which helpers were removed, and the exact verification results.
- [x] Then perform the Ralph completion sequence required by the user prompt:
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all staged changes, including `.ralph/` updates, with `task finished [task name]: [insert text]`
  - include test evidence and implementation challenges in the commit message
  - push with `git push`
  - update `AGENTS.md` only if there is a genuinely reusable learning

NOW EXECUTE


## Execution Notes

- Phase 1 audit result: the only production parse-back callers were `src/runtime/node.rs` `select_resume_start_intent(...)` and `src/ha/process_dispatch.rs` `start_intent_from_dcs(...)`. `src/postgres_managed_conf.rs` `parse_managed_primary_conninfo(...)` and `src/pginfo/conninfo.rs` `parse_pg_conninfo(...)` had no remaining non-managed external-ingest boundary, so both parser stacks were removed.
- Runtime architecture result: resume/startup now derives fresh `ManagedPostgresStartIntent` only from DCS plus runtime config, with `inspect_managed_recovery_state(...)` used solely as a minimal local physical-fact safety check for managed recovery residue. No conninfo is reconstructed from `pgtm.postgresql.conf`.
- HA architecture result: `start_intent_from_dcs(...)` now derives replica conninfo only when `leader_member_id` resolves to an authoritative DCS member. If managed replica residue exists locally but no leader-derived source is available, HA fails explicitly instead of trusting rendered managed config.
- Local persistence result: no new JSON/YAML sidecar or other persisted marker was required. DCS plus runtime config plus managed recovery-signal residue were sufficient.
- Helper/test cleanup result: removed `read_existing_replica_start_intent(...)`, `parse_managed_primary_conninfo(...)`, and `parse_pg_conninfo(...)`, along with parser-only tests. Replaced them with architectural tests around DCS authority and local residue failure semantics.
- Reviewed architecture boundaries in `src/ha/decide.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs`; no new parse-back or typed on-disk reconstruction was introduced there, so no code changes were needed in those files.
- Docs/comments result: no checked-in docs page outside this task file needed changes for this rewrite; `docs/tmp/` remained treated as generated artifacts.
- Verification:
  - `make check` passed
  - `make test` passed
  - `make test-long` passed, including ultra-long HA scenarios, Compose config validation, Docker single-node smoke, and Docker cluster smoke
  - `make lint` passed
