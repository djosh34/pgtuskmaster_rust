---
## Task: Expand runtime config schema for explicit secure node startup <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Redesign the runtime config model so every required secure startup setting is explicitly represented (TLS, HTTP, PostgreSQL hosting, roles/auth, pg_hba/pg_ident, and DCS init config).

**Scope:**
- Expand `src/config/mod.rs` and `src/config/schema.rs` with strongly typed fields for:
- PostgreSQL TLS server identity and client auth material.
- HTTP server TLS identity and client-facing auth wiring.
- PostgreSQL hosting/listen/datadir/socket/replication/bootstrap-relevant fields (including any currently inferred fields).
- Role list structure with required role kinds (`superuser`, `replicator`, `rewinder`), each carrying username plus enum auth (`tls` or `password`).
- `pg_hba` and `pg_ident` file-content/path fields as explicit config.
- DCS init config payload field(s) required for bootstrapping cluster defaults.
- Remove implicit fallback semantics from model shape by making inference impossible at type level.

**Context from research:**
- Current config relies on inferred defaults in multiple sections (`defaults.rs`, parser fallback, runtime assumptions like postgres user).
- Roles are not currently modeled as explicit typed list with role-specific auth semantics.
- TLS is currently represented by simple enable/disable and lacks explicit cert/key surfaces for both Postgres and HTTP in one canonical config contract.

**Expected outcome:**
- The config schema can fully describe a safe node startup without hidden runtime assumptions.
- All required secure runtime inputs are represented by explicit typed fields and enums.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `src/config/mod.rs` includes complete strongly typed runtime config structures for HTTP TLS, Postgres TLS, role list/auth enum, pg_hba/pg_ident, and DCS init config
- [ ] `src/config/schema.rs` includes matching partial/serde schema types and compatibility strategy for parsing
- [ ] Required roles (`superuser`, `replicator`, `rewinder`) are represented explicitly and unambiguously in the schema
- [ ] Role auth is enum-typed (`tls` | `password`) with explicit fields for each mode
- [ ] No type-level path remains for implicitly inferred default postgres identity fields
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
