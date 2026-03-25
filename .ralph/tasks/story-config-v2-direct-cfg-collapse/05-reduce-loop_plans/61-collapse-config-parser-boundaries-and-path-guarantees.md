## Plan: Collapse Config Parser Boundaries And Path Guarantees

### Why this is the next reduction target

The `config_v2` boundary still carries too much duplicated shape and too many post-parse guarantees:

- `src/config_v2/parser/private_schema.rs` mirrors a second runtime/operator config tree with its own `PostgresConfig`, `DcsConfig`, `LoggingConfig`, `ApiConfig`, token auth, TLS, and test-document builders.
- `src/config_v2/parser/load_config.rs` then hand-maps that raw tree into the canonical `RuntimeConfigV2` and `OperatorConfigV2`, while also owning shared path, secret, token, and URL normalization helpers.
- `src/runtime/node.rs` still hard-codes directory derivation from parsed config instead of consuming a canonical config-owned list of required runtime directories.
- `src/postgres_managed.rs` still re-absolutizes config paths and re-checks configured TLS file locations even though those are parser-owned guarantees.

That is a boundary problem, not domain complexity. The canonical owners already exist:

- `RuntimeConfigV2` and `OperatorConfigV2` for validated config,
- `PostgresConfig` and `LoggingConfig` for filesystem paths,
- parser-private TOML-only inputs for secrets, inline/path values, and transport-specific nesting.

This slice should collapse the parser onto those canonical owners and delete the downstream path re-validation that only exists because the parser does not finish the job yet.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` and `src/config_v2/types.rs` both define overlapping config concepts for postgres, DCS, logging, API, TLS, and operator auth; the raw tree exists mostly to be mapped immediately into the canonical tree.
- `src/config_v2/parser/load_config.rs` computes `data_dir`, `socket_dir`, `log_file`, `pg_hba_file`, `pg_ident_file`, `postgres_log_dir`, and `postgres_pg_ctl_log`, but `src/runtime/node.rs` still has bespoke startup logic for the same directory set.
- `src/postgres_managed.rs` re-absolutizes `cfg.postgres.pg_hba_file`, `cfg.postgres.pg_ident_file`, and `data_dir`-derived managed files even though those paths can be canonicalized once in the parser.
- `src/postgres_managed.rs` also re-validates configured TLS files through `resolve_existing_configured_file`, duplicating parser-time path guarantees instead of trusting validated config.
- `src/config_v2/parser/private_schema.rs` includes large TOML test-document builders that duplicate defaults and section wiring already encoded in the parser and test helpers.

### Execution plan

1. Make runtime filesystem paths canonical on the existing config types.
   - Normalize runtime filesystem paths to absolute paths during parsing, using the existing `RuntimeConfigV2`, `PostgresConfig`, and `LoggingConfig` fields instead of introducing a new path DTO.
   - Cover `data_dir`, `socket_dir`, `log_file`, `pg_hba_file`, `pg_ident_file`, logging sink file path, postgres log dir, postgres pg_ctl log, binary overrides, and parser-resolved TLS file paths.
   - Keep parser ownership of path validation and normalization; downstream code should only consume validated paths.

2. Collapse startup directory preparation onto config-owned facts.
   - Add a small iterator or accessor on an existing config type that yields the required runtime directories derived from canonical parsed paths.
   - Rewrite `src/runtime/node.rs` startup preparation as a tight loop over that existing config-owned path set plus the single Unix permission step for `data_dir`.
   - Preserve startup side effects in runtime startup; do not make `load_runtime_config` create directories.

3. Move conversion logic to the parser inputs that own the external TOML shape.
   - Push the large runtime/operator mapping logic out of `load_config.rs` and onto parser-private input impls such as `RuntimeDocument::into_runtime_config(...)` and `OperatorDocument::into_operator_config(...)`.
   - Keep `load_config.rs` as thin entry-point code for file I/O, TOML decode, and parser-private shared field helpers that truly need to be shared.
   - Merge overlapping token, TLS, URL, and path resolution paths where runtime and operator parsing currently repeat the same shape checks.

4. Delete downstream path revalidation in managed postgres.
   - Remove `absolutize_path` and stop converting already-canonical config paths at call sites.
   - Narrow `resolve_existing_configured_file` so it only performs existence/type checks that cannot happen earlier, or delete it entirely if parser-time canonical TLS path validation is sufficient.
   - Keep `postgres_managed` focused on materializing files and rendering config, not proving config correctness again.

5. Shrink parser test fixture overlap.
   - Reduce `private_schema.rs` test-document builder duplication by reusing the canonical parser conversion path and keeping only the minimal TOML-only helper surface that tests actually need.
   - Prefer a few focused helper constructors over a second full runtime/operator document assembly layer with duplicated defaults.
   - If fully deleting the builder layer expands test code, keep the public test helpers but remove duplicated section defaults and document assembly steps.

6. Update validation coverage.
   - Update parser tests to assert absolute/canonical path outputs, merged operator TLS/auth behavior, and the reduced mapper boundary.
   - Update `src/runtime/node.rs` tests to assert directory side effects from the config-owned path iterator rather than ad hoc per-field branches.
   - Update `src/postgres_managed.rs` tests so they expect canonical parser-delivered paths and no cwd-relative path repair.

7. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not introduce a replacement path wrapper if the existing config fields can carry the canonical paths directly.
- Keep filesystem side effects out of config loading; canonicalize and validate there, but create directories only during runtime startup.
- Reuse the existing config types and parser-private input types; do not add a second "validated config input" model.
- If deleting raw parser structs forces larger custom `Deserialize` implementations than the code it removes, keep the raw TOML-only shapes and still complete the path-boundary reductions in steps 1 to 4.
- Preserve current operator/runtime parsing semantics for token resolution, expected transport validation, and runtime documents that embed a `pgtm` operator block.

### Expected yield

- Shrink `src/config_v2/parser/load_config.rs` to a thin loader instead of a second config domain.
- Delete `postgres_managed::absolutize_path` and related pass-through path plumbing.
- Reduce or remove bespoke directory branching in `src/runtime/node.rs`.
- Delete duplicated parser/test fixture wiring that only exists to rebuild shapes already represented by the canonical config types.

NOW EXECUTE
