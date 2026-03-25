## Plan: Collapse Write-Convergence TLS And Connect-DSN Helpers Onto Core Owners

### Why this is the next verified reduction target

The HA write-convergence invariant still owns a private helper stack for two concerns that already have real owners elsewhere in the tree:

- `tests/ha/support/invariants/write_convergence.rs` rewrites `PgConnInfo` into a connectable DSN and decides whether TLS file material is present.
- The same file also reimplements PEM certificate/private-key loading even though `src/tls.rs` already owns PEM parsing for the runtime server path.

That means the HA support layer is carrying a fake boundary instead of depending on the core types that already define conninfo and PEM semantics.

### Current overlap verified in code

- `tests/ha/support/invariants/write_convergence.rs:373-393`:
  - `connect_member(...)` calls `connectable_conninfo(...)` and `conninfo_uses_tls_files(...)` before connecting.
- `tests/ha/support/invariants/write_convergence.rs:742-848` declares a second helper cluster just to support that connection flow:
  - `build_tls_connector(...)`
  - `required_tls_path(...)`
  - `conninfo_uses_tls_files(...)`
  - `connectable_conninfo(...)`
  - `load_cert_chain(...)`
  - `load_private_key(...)`
- `src/pginfo/conninfo.rs:62-79` already owns the `PgConnInfo` / `PgClientTls` shape, but the connect-ready variant logic is not owned there yet.
- `src/tls.rs:221-245` already owns PEM certificate and private-key parsing through:
  - `parse_pem_cert_chain(...)`
  - `parse_pem_private_key(...)`
- `src/lib.rs:18` keeps `tls` private today, while `Cargo.toml` keeps `tokio-postgres-rustls` as a dev-dependency only, so the reduction should reuse the shared PEM parsing owner without moving the test-only postgres connector type into the main library surface.

### Execution plan

1. Move connect-ready conninfo behavior onto the core conninfo types.
   - Add methods on the existing `PgClientTls` / `PgConnInfo` owners in `src/pginfo/conninfo.rs` instead of keeping free helpers in the HA invariant file.
   - The methods should cover:
     - whether TLS file material is present
     - creating the DSN/clone used for live postgres connections where `verify-ca` / `verify-full` are downgraded to `require` and file paths are stripped
   - Do not introduce a new wrapper struct for this.

2. Reuse the core PEM parsing owner from `src/tls.rs`.
   - Promote the existing PEM parsing helpers into a narrowly exposed shared API instead of duplicating file reads and PEM parsing in `write_convergence.rs`.
   - Keep the actual `MakeRustlsConnect` assembly inside `tests/ha/support/invariants/write_convergence.rs`, because that type is still test-only by dependency boundary.

3. Collapse the HA invariant helper stack.
   - Rewrite `connect_member(...)` in `tests/ha/support/invariants/write_convergence.rs` to call the new `PgConnInfo` / `PgClientTls` methods plus the shared PEM parsing functions.
   - Delete the local helper functions that become redundant:
     - `required_tls_path(...)`
     - `conninfo_uses_tls_files(...)`
     - `connectable_conninfo(...)`
     - `load_cert_chain(...)`
     - `load_private_key(...)`
   - If `build_tls_connector(...)` still remains, keep it as a thin adapter around the shared owners only.

4. Move tests to the real owners.
   - Relocate the conninfo-normalization assertions from `tests/ha/support/invariants/write_convergence.rs` into `src/pginfo/conninfo.rs` tests.
   - Add focused tests in `src/tls.rs` for the newly shared PEM-loading entry points if coverage is needed there.
   - Delete the HA-invariant tests that only existed for the removed local helper layer.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not move `tokio-postgres-rustls` into the main dependency surface unless the existing dev-dependency boundary proves impossible to preserve.
- Do not introduce a new DTO/helper wrapper around conninfo or TLS metadata.
- Keep TLS error messages concrete and do not swallow which file/key failed.
- Preserve the current write-convergence behavior for both non-TLS and TLS routing targets.

### Expected yield

- Delete the write-convergence file's duplicate conninfo/TLS helper layer.
- Keep connection-shape ownership on `PgConnInfo` / `PgClientTls`.
- Keep PEM parsing owned once in `src/tls.rs`.
- Shrink both the runtime and test support surface by removing private helper indirection from `tests/ha/support/invariants/write_convergence.rs`.

NOW EXECUTE
