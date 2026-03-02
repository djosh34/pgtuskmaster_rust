---
## Task: Implement typed postgres config and conninfo parser renderer <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
**Goal:** Replace raw decisive postgres strings with typed config and strict conninfo parsing.

**Scope:**
- Implement typed `PgConfig`, `PgConnInfo`, and supporting value types in `src/pginfo/state.rs` and/or dedicated config domain module.
- Implement `parse_pg_conninfo` and `render_pg_conninfo`.
- Add strict validation and roundtrip tests.

**Context from research:**
- Plan requires no raw string decisions for critical HA fields.

**Expected outcome:**
- HA-relevant postgres config values are type-safe and validated.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] `PgConfig` and `PgConnInfo` contain typed fields from plan.
- [ ] Parser rejects invalid syntax, missing required keys, and unsupported ssl modes.
- [ ] Roundtrip tests ensure `parse(render(x)) == x` for canonical forms.
- [ ] Run targeted parser tests.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] On failure, create `$add-bug` tasks with failing input samples.
</acceptance_criteria>
