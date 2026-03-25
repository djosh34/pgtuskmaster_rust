## Plan: Collapse Managed Role Reconciliation Shadow Types

### Why this is the next reduction target

`src/postgres_roles.rs` still invents a file-local DTO layer that just recopies data already owned by `RuntimeConfigV2`:

- `DesiredManagedRoleSet`
- `ManagedRoleSpec`
- `MandatoryManagedRole`
- `ManagedRoleGrant`

That layer exists only to shuttle `cfg.postgres.{superuser,replicator,rewinder}` into SQL rendering. The connection bootstrap is already separate in `reconcile_managed_roles_v2`, so the remaining abstraction buys nothing and keeps both production code and tests larger than necessary.

This is a good next slice because it reuses the existing `config_v2::types::RoleConfig` and `Secret` instead of preserving one more private role taxonomy, and it should also let the tiny test module stop hand-constructing a full `RuntimeConfigV2`.

### Current overlap already verified

- `src/postgres_roles.rs` defines `DesiredManagedRoleSet` only to hold the same three mandatory roles that already live on `RuntimeConfigV2.postgres`.
- `ManagedRoleSpec { username, password, grants }` repeats `RoleConfig { username, password }` from `src/config_v2/types.rs`, with only one extra grant case for rewinder.
- `MandatoryManagedRole::attributes()` exists only to choose three static SQL attribute strings, so the enum is a wrapper around constants rather than domain state.
- `ManagedRoleGrant` has a single variant, `RewindFunctionExecute`, and is only used to decide whether rewinder grant SQL should be appended.
- The test in `src/postgres_roles.rs` currently builds an entire `RuntimeConfigV2` fixture inline instead of reusing `crate::config_v2::runtime_test_config()`.

### Execution plan

1. Delete the shadow managed-role DTO layer in `src/postgres_roles.rs`.
   - Remove `DesiredManagedRoleSet`, `ManagedRoleSpec`, `MandatoryManagedRole`, and `ManagedRoleGrant`.
   - Remove helpers that only exist to support those types:
     - `DesiredManagedRoleSet::all_roles`
     - `MandatoryManagedRole::attributes`
     - `v2_managed_role`
     - `render_managed_role_reconciliation_sql_for_set`
     - `render_role_grant_reconciliation_block`

2. Render reconciliation SQL directly from the config-owned roles.
   - Keep `render_managed_role_reconciliation_sql_v2(cfg)` as the public pure SQL boundary.
   - Replace the deleted DTO pipeline with direct rendering for the three mandatory roles:
     - superuser uses `cfg.postgres.superuser`
     - replicator uses `cfg.postgres.replicator`
     - rewinder uses `cfg.postgres.rewinder`
   - Change `render_protected_role_provision_block` to take the real inputs it needs directly, for example username/password/attributes, instead of a wrapper struct.
   - Append rewinder grant SQL directly from `cfg.postgres.rewinder.username` rather than through a single-variant grant enum.

3. Collapse the file-local test fixture onto existing config test support.
   - Replace the handwritten `sample_cfg()` body with `crate::config_v2::runtime_test_config()` plus only the role-field overrides needed by the assertions.
   - Keep the existing behavior checks for the three mandatory role attribute strings and the rewinder grant SQL.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the total diff to improve beyond the current validated baseline.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse `config_v2::types::RoleConfig` and `Secret`; do not introduce a replacement local role spec type.
- Keep `render_managed_role_reconciliation_sql_v2` pure; do not push tokio-postgres connection concerns back into SQL rendering.
- Keep the rewinder grant behavior unchanged; this slice deletes the grant enum boundary, not the underlying GRANT statements.
- If implementation reveals a second caller that truly needs an owned role-reconciliation shape, switch this plan back to `TO BE VERIFIED` instead of inventing another wrapper.

### Expected yield

- Delete four file-local ADTs from `src/postgres_roles.rs`.
- Delete multiple pass-through helpers that only exist to feed those ADTs.
- Shrink the production renderer so it follows the real owner (`RuntimeConfigV2`) directly.
- Remove the oversized handwritten test fixture in favor of existing config test support.

NOW EXECUTE
