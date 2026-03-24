## Plan: Collapse In-Crate Test Config String Overlays Onto Private Schema Owner

### Why this reduction target

The broad "make the typed schema public" direction already failed twice. The remaining higher-yield seam is narrower and stays inside the crate:

- `src/config_v2/parser/private_schema.rs` already owns the real typed test document builders through `RuntimeDocument`, `OperatorDocument`, `build_runtime_test_document(...)`, and `build_operator_test_document_value(...)`.
- In-crate unit tests still leave that owner and rebuild config knowledge as raw TOML snippets passed through `render_runtime_test_config_toml(..., extra_sections)` and `render_operator_test_config_toml(..., extra_sections)`.
- Those snippets duplicate the same auth/TLS/path/binary sections across:
  - `src/cli/config.rs`
  - `src/config_v2/parser/load_config.rs`
  - `src/process/cluster.rs`
  - `src/pginfo/worker.rs`
- External integration-style callers in `tests/cli_binary.rs` and `tests/ha/support/observer/pgtm.rs` still need a text-facing surface, but the in-crate callers do not.

This is the better slice because it deletes duplicated schema text from large unit-test modules without widening raw schema visibility to external callers.

### Current overlap already verified

- `rg` shows the remaining renderer callsites are concentrated in four in-crate test modules plus two external integration callers.
- `src/cli/config.rs` and `src/config_v2/parser/load_config.rs` both still construct operator auth/TLS sections with `toml_path_source(...)` and `toml_string_secret(...)`.
- `src/process/cluster.rs` and `src/pginfo/worker.rs` still bolt runtime overrides on top of the baseline renderer through large `extra_sections` TOML fragments.
- `private_schema.rs` already has the typed owners needed to avoid this:
  - `build_runtime_test_document(...)` returns `RuntimeDocument`
  - `build_operator_test_document_value(...)` already assembles the operator-side typed document before serializing
- The failed plan-50/51 path proved the mistake was widening those types outward, not keeping mutation inside the owner.

### Execution plan

1. Add crate-only typed mutation helpers at the schema owner.
   - In `src/config_v2/parser/private_schema.rs`, add cfg-gated helpers that:
     - start from the existing runtime/operator typed builders
     - apply a caller-provided mutation closure inside the module that owns `RuntimeDocument` / `OperatorDocument`
     - serialize once at the end
   - Keep these helpers crate-only for in-crate tests; do not re-export raw schema structs or make them public for integration callers.

2. Collapse the in-crate runtime-config test callsites onto the typed helpers.
   - Rewrite the unit-test config builders in `src/process/cluster.rs` and `src/pginfo/worker.rs` so they mutate the typed runtime document directly instead of appending `extra_sections` TOML.
   - Reuse the existing typed fields for rewind TLS, binary overrides, logging, API transport/auth, and debug settings instead of reconstructing TOML syntax.

3. Collapse the in-crate operator-config test callsites onto the typed helpers.
   - Rewrite the operator config fixtures in `src/cli/config.rs` and `src/config_v2/parser/load_config.rs` to set auth/TLS on the typed operator document builder rather than format raw `[api.auth]`, `[api.tls]`, and `[postgres.tls]` tables.
   - Keep intentionally invalid parser fixtures as the smallest remaining raw TOML snippets only where the typed schema should not represent that invalid state.

4. Remove the now-redundant in-crate text glue.
   - Delete the repeated format strings and the associated `toml_path_source(...)` / `toml_string_secret(...)` imports from the rewritten in-crate modules.
   - Only remove shared renderer helpers from `private_schema.rs` if no external caller still needs them; do not widen this slice just to delete `join_rendered_sections(...)`.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current `+8107 -11150 diff: -3043` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not expose `RuntimeDocument`, `OperatorDocument`, or the raw auth/TLS/path structs outside the schema owner.
- Do not change the external integration-test support API in this slice; `tests/cli_binary.rs` and `tests/ha/support/observer/pgtm.rs` can stay on the string-facing helpers for now.
- Do not invent a second patch DTO hierarchy. The whole point is to reuse the existing typed schema and mutate it only inside `private_schema.rs`.
- If the closure-based helper surface starts growing more code than the deleted unit-test TOML snippets, switch this plan back to `TO BE VERIFIED` with the measured failed-yield note before executing further.

### Failed yield note

- Attempted this slice by adding crate-only editor wrappers around the private schema owner and rewriting the targeted in-crate tests onto them.
- Measured result after formatting and compile cleanup: `+8265 -11158 diff: -2893`.
- Previous validated baseline before this slice: `+8107 -11150 diff: -3043`.
- This means the wrapper/edit-surface added roughly 150 more lines than it deleted, so the plan did not achieve the required reduction and should not continue in its current form.

TO BE VERIFIED
