## Plan: Collapse Runtime Test Preset Ladder Onto `runtime_test_document`

### Why this is the next verified reduction target

Previous config-reduction slices already removed the public TOML-renderer corridor and several one-off wrapper helpers above `config_v2`, but `src/config_v2/parser/load_config.rs` still keeps a second internal preset ladder on top of one real owner:

- `runtime_test_document(...)` already owns the shared runtime test document defaults.
- `render_runtime_test_config_document_toml(...)` already owns the generic TOML rendering path for that document.
- Several higher helpers still just hard-code small bundles of identity, HBA, and extra-section overrides on top of those owners:
  - `render_runtime_test_config_toml(...)`
  - `load_runtime_test_config_with_hba_and_sections(...)`
  - `load_runtime_test_config_with_sections(...)`
  - `runtime_test_config(...)`
  - `runtime_test_config_with_data_dir(...)`
  - `managed_postgres_test_config(...)`
  - `trace_logging_test_config(...)`

That is still a boundary smell. The real owner is the document/default bundle, but `load_config.rs` keeps re-expressing the same defaults through a stack of preset couriers and then many callers immediately mutate the returned config again.

### Verified overlap in the current code

1. `runtime_test_document(...)` already carries the shared typed defaults.
   - `src/config_v2/parser/load_config.rs`
     - `runtime_test_document(...)`
     - `render_runtime_test_config_document_toml(...)`
   - The real degrees of freedom are already explicit there: identity, paths, DCS endpoints, role credentials, access contents, and extra sections.

2. The remaining preset helpers are thin wrappers over that same owner.
   - `src/config_v2/parser/load_config.rs`
     - `render_runtime_test_config_toml(...)` just injects fixed credentials and access contents, then calls `render_runtime_test_config_document_toml(...)`.
     - `load_runtime_test_config_with_hba_and_sections(...)` just calls the same document renderer with fixed cluster/member defaults plus binary-path sections, then reparses it.
     - `load_runtime_test_config_with_sections(...)`, `runtime_test_config(...)`, `runtime_test_config_with_data_dir(...)`, `managed_postgres_test_config(...)`, and `trace_logging_test_config(...)` each only hard-code one more small preset on top.

3. Callers often mutate these presets immediately, which proves the wrappers are not the real owners.
   - `src/logging/postgres_ingest.rs:1478`
     - `trace_logging_test_config()` is called and then `cfg.logging.postgres.log_dir` and `cfg.postgres.log_file` are overwritten immediately.
   - `src/process/worker.rs:830`
     - `managed_postgres_test_config(...)` is called and then `socket_dir` and `log_file` are overwritten immediately.
   - `src/postgres_managed.rs:676`
     - `managed_postgres_test_config(...)` is called and then `listen_host`, `listen_port`, `socket_dir`, and `extra_gucs` are overwritten immediately.
   - Those wrappers are therefore just intermediate couriers between callers and the actual `RuntimeConfigV2` fields they want.

4. The public cfg-gated surface still exports these remaining presets.
   - `src/config_v2/mod.rs`
   - `src/config_v2/parser/mod.rs`
   - Even after the earlier renderer cleanup, the module surface still exposes multiple specialized preset entrypoints that differ only by a few hard-coded fields.

### Execution plan

1. Keep one real runtime test-document owner.
   - Keep `runtime_test_document(...)` as the source of shared defaults.
   - Keep `render_runtime_test_config_document_toml(...)` as the generic rendered-document path for callers that truly need TOML text.
   - Do not add a new builder struct, patch enum, or fixture DTO.

2. Collapse the preset corridor in `load_config.rs`.
   - Delete the thin wrappers that only hard-code defaults already representable by the generic document owner.
   - Target the remaining preset stack first:
     - `render_runtime_test_config_toml(...)`
     - `load_runtime_test_config_with_sections(...)`
     - `runtime_test_config(...)`
     - `managed_postgres_test_config(...)`
     - `trace_logging_test_config(...)`
   - Keep at most one typed loader helper if it still clearly deletes more code than inlining it at every call site.
   - If `load_runtime_test_config_with_hba_and_sections(...)` still buys real reduction after the preset collapse, keep it; otherwise inline it into the one surviving typed helper and delete it too.

3. Retarget callers to the remaining real owners.
   - Switch TOML-only callers such as `tests/cli_binary.rs` to `render_runtime_test_config_document_toml(...)`.
   - Switch typed-config callers in:
     - `src/postgres_managed.rs`
     - `src/process/worker.rs`
     - `src/process/cluster.rs`
     - `src/api/worker.rs`
     - `src/ha/worker.rs`
     - `src/pginfo/worker.rs`
     - `src/runtime/node.rs`
     - `src/postgres_roles.rs`
     - `src/dev_support/api.rs`
     - `src/logging/postgres_ingest.rs`
     - `src/logging/core/runtime.rs`
   - Prefer direct use of the surviving generic helper plus final call-site field rewrites instead of recreating named presets.

4. Narrow the re-export surface after the preset collapse.
   - Remove dead cfg-gated re-exports from:
     - `src/config_v2/mod.rs`
     - `src/config_v2/parser/mod.rs`
   - Keep only the generic test-support entrypoints still needed after step 3.

5. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34068`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves beyond `+10352 -14359 diff: -4007`
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new structs, enums, traits, or builder layers for runtime test config presets.
- Keep `runtime_test_document(...)` as the owner of shared runtime test defaults; do not spread those defaults into multiple helper stacks again.
- If one tiny typed helper clearly deletes more code than forcing every caller to spell out the same binary-path/HBA bundle, keep that one helper and delete the rest.
- If call-site changes reveal that a deleted preset was actually hiding materially different semantics rather than fixed defaults, switch this plan back to `TO BE VERIFIED` instead of forcing a lossy collapse.

### Expected yield

- Delete the remaining runtime test preset ladder from `src/config_v2/parser/load_config.rs`.
- Shrink cfg-gated re-exports in `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs`.
- Remove one more layer of `RuntimeConfigV2` courier code where callers already mutate the returned configs directly.

NOW EXECUTE
