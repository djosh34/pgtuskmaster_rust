## Plan: Export Generic Runtime Test Document Builder Before Collapsing Fixture Renderers

### Why plan 38 failed

The host-observer half of the slice is still sound, but the HA runtime half cannot be implemented by wrapping `render_runtime_test_config_toml(...)` with appended TOML sections:

- `render_runtime_test_config_toml(...)` trims `ha`, `process`, `logging`, `api`, `pgtm`, and `debug`, but it does **not** trim `postgres.roles.mandatory.*`.
- The HA cucumber fixtures must override the replicator and rewinder usernames for `CustomRoles`, and they must replace the default string secrets with file-backed secrets for all mandatory roles.
- Reopening those already-rendered role tables from the caller would duplicate TOML tables/keys and fail parsing.

So the boundary reduction needs one extra generic surface first: caller-owned fixture code must be able to start from the reusable runtime baseline *before* it is serialized into immutable TOML text.

### Revised reduction target

Keep the parser schema module responsible only for generic runtime/operator test document construction. Move the HA fixture-specific mutations and the observer-specific appended sections into their only callers.

### Execution plan

1. Expose the reusable runtime test document value.
   - Change `build_runtime_test_document_value(...)` to a test-support export that callers can reuse.
   - Re-export that generic helper from `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs`.
   - Do not expose private schema structs; export only `toml::Value`-based helpers that keep the parser module generic.

2. Move HA fixture ownership onto `tests/ha/support/givens/mod.rs`.
   - Replace the import of `render_ha_member_runtime_config_toml(...)` with the exported generic runtime document builder plus local serialization.
   - In `givens/mod.rs`, mutate the returned `toml::Value` with the existing HA-only values: member listen host, rewind TLS, server TLS identity/client auth, mandatory role usernames and file secrets, access file paths, extra GUCs, process overrides, logging, API auth/TLS, `pgtm` observer config, and debug flag.
   - Serialize the final document locally with `toml::to_string`, validate it with the existing runtime parser, and keep the current test expectations unchanged.

3. Move host-observer operator config ownership onto `tests/ha/support/observer/pgtm.rs`.
   - Replace `render_host_observer_operator_config_toml(...)` with a local wrapper around `render_operator_test_config_toml(...)`.
   - Keep auth/TLS sections caller-owned there because the generic operator renderer already leaves those sections empty.

4. Delete the dead specialized parser helpers.
   - Remove `build_ha_member_runtime_document_value(...)`, `render_ha_member_runtime_config_toml(...)`, and `render_host_observer_operator_config_toml(...)`.
   - Remove any now-dead helper that existed only for those surfaces.
   - Stop re-exporting the deleted names.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7247 -10007 diff: -2760` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not keep the fixture-specific runtime mutator in `private_schema.rs`; the whole point is to move HA-specific knowledge to the HA harness.
- Do not add a new config DTO layer or a second generic runtime renderer with fixture-specific knobs.
- If the exported `toml::Value` builder still forces too much table surgery in the caller, stop and design a smaller generic helper around the exact reusable boundary before writing more code.

NOW EXECUTE
