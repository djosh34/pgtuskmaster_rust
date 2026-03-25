## Plan: Collapse Replica Conninfo Rendering Onto `PgConnInfo`

### Why this is the next verified reduction target

`src/postgres_managed.rs` still lowers the same replica connection state twice:

- once into `primary_conninfo` inside `render_managed_postgres_conf(...)`
- once into the managed standby passfile inside `materialize_managed_standby_passfile(...)`

That is a real boundary smell because the actual owner of the connection fields is already `PgConnInfo` in `src/pginfo/conninfo.rs`, but `postgres_managed.rs` still re-derives host/port extraction and escaping policy locally.

This is a good next slice because it stays inside two closely related owners, reuses the existing `PgConnInfo` and `PgClientTls` shapes, and should delete code from one of the largest remaining source files without reopening broader process or config behavior.

### Verified overlap in the current code

1. `PgConnInfo` already owns canonical conninfo rendering.
   - `src/pginfo/conninfo.rs`
     - `impl fmt::Display for PgConnInfo`
     - `render_conninfo_value(...)`
   - `Display` already knows how to derive `host`, `port`, optional `hostaddr`, TLS fields, and quoted conninfo values from the real owner.

2. `postgres_managed.rs` still rebuilds endpoint lowering for the same owner.
   - `src/postgres_managed.rs`
     - `render_libpq_passfile_entry(...)`
     - `passfile_target_fields(...)`
     - `escape_libpq_passfile_field(...)`
   - `render_libpq_passfile_entry(...)` immediately pulls `host` and `port` back out of `conninfo.route.endpoint()`, which duplicates the same route ownership already handled in `PgConnInfo::fmt(...)`.

3. Replica-only validation is split from replica-only rendering.
   - `replica_primary_conninfo(...)` decides whether a replica source exists.
   - `materialize_managed_standby_passfile(...)` and `render_managed_postgres_conf(...)` each call that owner and then rebuild different string projections from the same `PgConnInfo`.
   - The replica branch therefore still fans out into two single-purpose string renderers plus one selector.

4. The existing tests already assert the real behavior boundary.
   - `src/postgres_managed.rs`
     - `render_managed_postgres_conf_quotes_and_escapes_string_values`
     - `materialize_managed_postgres_config_writes_managed_standby_passfile`
     - the real replay/integration coverage around managed replica startup
   - `src/pginfo/conninfo.rs`
     - render and parse round-trip tests for the canonical conninfo owner
   - That means execution can mostly retarget existing tests instead of introducing new fixture layers.

### Execution plan

1. Move replica connection string ownership onto `PgConnInfo`.
   - Add the smallest owner methods needed on `PgConnInfo` itself for the remaining managed-postgres projections.
   - Prefer methods that render:
     - the standby passfile line
     - the conninfo string with an appended managed `passfile=...` entry
   - Do not introduce a new replica DTO, wrapper enum, or renderer trait.

2. Delete the local lowering helpers from `postgres_managed.rs`.
   - Remove `render_libpq_passfile_entry(...)`.
   - Remove `passfile_target_fields(...)`.
   - Remove `escape_libpq_passfile_field(...)`.
   - Replace their call sites with direct `PgConnInfo` owner methods.

3. Collapse replica-only selection so the managed owner stops asking twice.
   - Keep `replica_primary_conninfo(...)` only if it still deletes more branching than it costs.
   - If the selector becomes a one-line wrapper after step 2, inline it into the two replica-only call sites and delete it.

4. Keep escaping rules exact.
   - Preserve the current passfile newline rejection.
   - Preserve `:` and `\` escaping in the passfile output.
   - Preserve the current `primary_conninfo = '... passfile=...'` quoting behavior in `postgresql.conf`.
   - Reuse `render_conninfo_value(...)` where possible instead of rebuilding quoting logic.

5. Reuse the existing tests at the real boundary.
   - Update `pginfo/conninfo.rs` tests to cover any new owner methods directly.
   - Keep the managed-postgres tests focused on emitted config/passfile contents rather than helper names.
   - Delete test-only helpers if they only existed for removed local renderers.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34070`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves beyond `+10300 -14305 diff: -4005`
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new structs, enums, traits, or renderer layers for this slice.
- `PgConnInfo` remains the owner of replica connection data; do not move string policy into another managed-postgres courier.
- If reusing `PgConnInfo` would force awkward placeholder arguments for non-replica cases, keep the owner methods replica-focused rather than widening them into a generic rendering abstraction.
- If the selector helper still keeps call sites clearly smaller after the rendering collapse, keep it; otherwise inline it and delete the wrapper.

### Expected yield

- Delete the passfile-specific route/escape helper stack from `src/postgres_managed.rs`.
- Move the remaining replica connection rendering policy onto the existing `PgConnInfo` owner.
- Reduce one more chunk of local stringly glue in `src/postgres_managed.rs`, which is still one of the largest files in the tree.

NOW EXECUTE
