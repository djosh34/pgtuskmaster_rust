## Plan: Collapse Runtime `NodeIdentity` Mirrors

### Why this improvement

`RuntimeConfigV2` already owns `cluster_name`, `scope`, and `member_id`, but the runtime still stores a duplicated `NodeIdentity` across multiple worker roots and bootstraps:

- `src/runtime/node.rs`
- `src/dcs/mod.rs`
- `src/dcs/worker.rs`
- `src/process/startup.rs`
- `src/process/state.rs`
- `src/process/cluster.rs`
- `src/process/planner.rs`
- `src/ha/startup.rs`
- `src/ha/state.rs`
- `src/ha/worker.rs`
- `src/ha/process_dispatch.rs`
- `src/pginfo/startup.rs`
- `src/pginfo/state.rs`
- `src/pginfo/worker.rs`
- `src/api/startup.rs`
- `src/api/worker.rs`

This is exactly the cfg-reduce-loop smell: static config fields are copied into runtime state and then threaded around as if they were dynamic.

### Verified current overlap

- `RuntimeConfigV2` is already the shared static root.
- `NodeIdentity` is derivable entirely from `cfg.cluster_name`, `cfg.scope`, and `cfg.member_id`.
- Several hot-path consumers only need one field:
  - `process::planner::plan` only needs `member_id`
  - HA decision/dispatch only need `member_id` and `scope`
  - pginfo publish errors only need `member_id`
- The API response still legitimately exposes `NodeIdentity`, but that object can be synthesized at response/render time instead of stored in `ApiRuntimeCtx`.

### Execution steps

1. Remove the redundant bootstrap handoff.
   - Delete the `NodeIdentity` construction in [`src/runtime/node.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs).
   - Change these bootstraps to accept only `cfg` plus their true runtime dependencies:
     - [`src/dcs/mod.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/mod.rs)
     - [`src/process/startup.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/startup.rs)
     - [`src/ha/startup.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/startup.rs)
     - [`src/pginfo/startup.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/startup.rs)
     - [`src/api/startup.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/startup.rs)

2. Delete stored `identity` fields from runtime contexts.
   - Remove `identity: NodeIdentity` from:
     - [`src/dcs/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs)
     - [`src/process/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/state.rs)
     - [`src/ha/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/state.rs)
     - [`src/pginfo/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/state.rs)
     - [`src/api/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs)
   - Fix all constructor callsites to stop passing `NodeIdentity`.

3. Replace mirror reads with direct `cfg` reads in each corridor.
   - DCS:
     - use `cfg.scope` to build `DcsKeySpace`
     - use `cfg.member_id` directly for member-key writes, local leadership ownership, and local-member updates
   - HA:
     - pass `&ctx.cfg.member_id` into `decide`
     - use `ctx.cfg.member_id` / `ctx.cfg.scope` in [`src/ha/process_dispatch.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs)
     - use `ctx.cfg.member_id` when excluding self during observation
   - pginfo:
     - use `ctx.cfg.member_id` in publish/logging paths
   - API:
     - use `cfg` directly for auth, bind, and certificate reload as today
     - synthesize `NodeIdentity` only when building `NodeSnapshot` or controller arguments that still require it

4. Narrow helper signatures instead of preserving the old shape.
   - In [`src/process/planner.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/planner.rs), replace the `&NodeIdentity` parameter with the minimum needed input, expected to be `&MemberId`.
   - In [`src/process/cluster.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/cluster.rs), remove the stored `identity` field if it becomes unnecessary after planner narrowing.
   - In [`src/dcs/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), narrow internal helpers away from `&NodeIdentity` to `&MemberId` or direct `cfg` reads where possible.

5. Keep `NodeIdentity` only at real value boundaries.
   - Preserve the public API response shape in [`src/api/mod.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/mod.rs).
   - If a boundary still truly needs a `NodeIdentity` object, construct it locally from `cfg` at that boundary instead of storing it in runtime state.
   - Do not add a new config adapter, new cached identity field, or new wrapper method just to recreate the same duplication.

6. Update tests to follow the flattened boundary.
   - Remove `identity` setup from worker-context test builders that no longer need it.
   - Update process-planner tests to pass the narrowed member input instead of full `NodeIdentity`.
   - Update API tests to assert the rendered `NodeSnapshot.identity` still matches the config-derived fields.

7. Validate the reduction.
   - Run `make check`
   - Run `make lint`
   - Run `make test`
   - Run `make test-long`
   - Confirm tracked line count decreases with `bash .ralph/git_diff_lines_since.sh`

### Design guardrails

- Do not add a `cfg.identity` field or any other cached mirror on `RuntimeConfigV2`.
- Do not keep both `cfg` and `identity` in the same runtime context.
- Do not introduce a new helper DTO such as `RuntimeIdentity`, `NodeIdentityRef`, or `ProcessIdentity`.
- If a consumer turns out to need `cluster_name` or `scope` only transiently, pass that minimum field or synthesize `NodeIdentity` locally at the callsite.
- If execution reveals a boundary that cannot be collapsed onto `cfg` without changing shared types first, switch this plan tail back to `TO BE VERIFIED` and stop immediately.

NOW EXECUTE
