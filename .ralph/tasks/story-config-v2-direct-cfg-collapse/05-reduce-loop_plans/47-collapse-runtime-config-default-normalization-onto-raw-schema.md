## Plan: Collapse Runtime Config Default Normalization Onto Raw Schema

### Why this reduction target

`src/config_v2/parser/private_schema.rs` already owns the default-bearing runtime input graph, but `src/config_v2/parser/load_config.rs` still repeats that same policy in a second place:

- `load_config.rs` defines another copy of the runtime default constants for listen host/port, connect timeout, HA/process timeouts, postgres log polling, and log-cleanup limits.
- `map_runtime_document(...)` then runs those same fields through `normalized_or_default(...)` and `nonzero_or_default(...)` before building `RuntimeConfigV2`.
- `private_schema.rs` already owns the raw runtime fields and their default semantics, and the test runtime document builder still seeds sentinel zero values such as `connect_timeout_s: 0` only because the mapper, not the schema, currently owns zero-means-default behavior.

That is a wrong boundary. Input-default normalization belongs at the TOML/input layer, not in a one-shot mapper that immediately converts the same fields into runtime durations and ports.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` defines the raw runtime fields and default constants for `listen_port`, `loop_interval_ms`, `lease_ttl_ms`, `pg_rewind_ms`, `bootstrap_ms`, `fencing_ms`, `poll_interval_ms`, `max_files`, `max_age_seconds`, and `protect_recent_seconds`.
- `src/config_v2/parser/load_config.rs` repeats those constants and re-applies them via `nonzero_or_default(...)` at the runtime mapping boundary.
- `src/config_v2/parser/load_config.rs` also repeats the default listen host with `normalized_or_default(...)`.
- `src/config_v2/parser/private_schema.rs::runtime_test_document(...)` currently writes `connect_timeout_s: 0`, proving the builder still depends on mapper-owned default normalization instead of the raw schema owning it directly.
- `src/config_v2/parser/load_config.rs::load_runtime_timing_values(...)` reads raw runtime timing fields directly, so this slice must keep those timing reads aligned with the same canonical normalization path.

### Execution plan

1. Make the raw runtime schema own default normalization.
   - Add parser-local field deserializers or tiny wrapper helpers in `src/config_v2/parser/private_schema.rs` for the runtime fields that currently rely on mapper-side `0 => default` handling.
   - Move blank-or-missing listen-host normalization there as well if it reduces code instead of creating more glue.
   - Keep the TOML surface unchanged: missing values and explicit zero values must still resolve exactly as they do today.

2. Delete mapper-local default duplication from `load_config.rs`.
   - Remove the duplicate default constants from `src/config_v2/parser/load_config.rs`.
   - Replace `nonzero_or_default(...)` and any now-redundant `normalized_or_default(...)` calls with direct mapping from already-normalized raw values into `RuntimeConfigV2`.
   - Keep validation wording unchanged for truly invalid values such as advertise ports that remain zero after normalization is not applicable.

3. Reuse the normalized raw values in test-support readers/builders.
   - Update `load_runtime_timing_values(...)` to read the same normalized defaults instead of depending on a parallel fallback path.
   - Simplify `runtime_test_document(...)` and any adjacent test builders so they no longer seed sentinel values just to trigger mapper-owned defaults.
   - Reuse existing raw schema owners; do not create a second “normalized document” DTO layer.

4. Rebuild tests around the surviving owner.
   - Keep or add focused tests proving that explicit zero runtime timing/logging fields still resolve to the documented defaults.
   - Preserve the existing runtime/operator parser tests and their config spellings unless a minimal assertion rewrite is required by deleted helper functions.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+8038 -11066 diff: -3028` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not introduce a replacement runtime config DTO layer or a generic parser framework.
- Keep config file semantics stable: this slice is about moving normalization to the right owner, not changing accepted field names or defaults.
- Prefer a few local serde/default helpers over a broad abstraction if that keeps total code lower.
- If parser-local normalization for these fields expands into more adapter code than the repeated constants and mapper glue it deletes, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
