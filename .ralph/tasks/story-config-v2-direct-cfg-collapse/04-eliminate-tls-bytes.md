## Task: Eliminate TLS bytes <status>completed</status> <passes>true</passes>

<description>
Someone previously had this crazy idea to store bytes within the config struct. This causes a crazy amount of issues.
I don't want that at all. I don't want tls bytes to be inside the config struct, nor i ever want them to be written somewhere else.
Instead all components in code use the same tls struct and all use only path based approaches.
Do not store bytes of pem/cert/key whatever in any way inside the code.
Just give the full path to postgres or just give the path to rustls.

Also no more writing the key/cert/pem files to anywhere. Any cert file writing of that kind is forbidden.
</description>

<acceptance_criteria>
- [x] All versions of storing pem/cert or whatever bytes and/or copying/writing them into new files must be fully and verifiably be gone from code
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly 
</acceptance_criteria>

<boundary_review>
Smell applied from `improve-code-boundaries`: wrong config-ingestion boundary.

Verified repository facts before execution:
- The validated runtime/operator TLS shapes are already path-based: `src/pginfo/conninfo.rs` exposes `PgClientTls`, and `src/config_v2/types.rs` exposes `TlsConfig` plus `OperatorClientTlsConfig`.
- TLS byte-capable shapes still survive in three wrong places:
  - parser-private TLS schema still uses `PathOrInline` / `SecretSource` for TLS-only fields, then rejects non-path values later with `must be path-backed`
  - `src/cli/client.rs` still defines `CliTlsConfig` with `ca_cert_pem`, `client_cert_pem`, and `client_key_pem`
  - `src/postgres_managed.rs` still copies operator-supplied cert/key/CA files into managed files under `PGDATA`
- `src/postgres_managed.rs` currently materializes `pgtm.server.crt`, `pgtm.server.key`, and `pgtm.ca.crt`, and tests still pin that copy behavior.
- Local dependency audit shows `reqwest 0.12.28` and `etcd-client 0.14.1` only expose PEM-buffer TLS constructors. That means the correct production boundary is still path-only config/runtime state, with any unavoidable PEM reads kept as immediate library-edge parsing only. Those reads must not become stored config/runtime fields or copied output files.

Verified design correction needed before execution:
- Do not introduce a new public TLS DTO tree. Collapse onto the existing path-only shapes instead.
- TLS fields in the raw parser schema must stop accepting inline/string/env-capable shapes. If a TLS field is path-only by design, the parser-private type should encode that directly instead of validating it later.
- The CLI must stop carrying a second TLS representation. Reuse the existing path-only operator/runtime shapes and only derive `PgClientTls` when rendering PostgreSQL conninfo.
- Managed PostgreSQL must render the original configured TLS file paths directly. Copying cert/key/CA files into new managed files is forbidden by this task and must be removed.
- If execution reveals a caller that truly needs a missing shared path-only TLS shape, switch this task back to `TO BE VERIFIED`, explain the exact missing shape here, and stop immediately.
</boundary_review>

<plan>
1. Tighten the TLS schema boundary first:
   - replace TLS-only uses of `PathOrInline` / `SecretSource` in `src/config_v2/parser/private_schema.rs` with one path-only parser representation, or direct path-only fields where simpler
   - remove loader-side `must be path-backed` branches for TLS fields in `src/config_v2/parser/load_config.rs` and `src/config_v2/parser/load_operator_config.rs`
   - update parser tests/fixtures so TLS inline/string/env inputs fail because the schema no longer accepts them, not because later mapping rejects them
2. Collapse the CLI onto existing path-only TLS types:
   - delete PEM byte fields and byte-loading helpers from `src/cli/client.rs`
   - use `config_v2::types::OperatorClientTlsConfig` as the operator-side path carrier and derive `PgClientTls` only at the PostgreSQL connection rendering boundary
   - simplify `src/cli/connect.rs` so `--tls` only toggles emission of already-path-backed TLS and does not compensate for a bytes-vs-path split
3. Remove managed TLS file copying from PostgreSQL startup:
   - delete `MaterializedTlsFiles` and the cert/key/CA copying logic from `src/postgres_managed.rs`
   - keep `ManagedPostgresTlsConfig` path-based, but point it directly at `cfg.postgres.tls` file paths
   - remove copied-path fields from `ManagedPostgresConfig` if they no longer carry independent information, and update the dependent tests/callers
4. Keep remaining TLS bytes at one local library edge only:
   - for `reqwest` and `etcd-client`, keep any required PEM reads local to the client-builder callsites because those crates do not expose path APIs in the pinned versions
   - do not return PEM bytes from helpers, stash them in structs, or write them back out as files
5. Rewrite tests to prove the new boundary:
   - invert postgres-managed tests to assert direct configured TLS paths are rendered and that no `pgtm.server.*` / `pgtm.ca.crt` copies are created
   - update CLI/config/parser tests to assert path-only TLS resolution with no PEM byte fields or non-path-backed fallback branches
   - verify deletions with focused searches for `ca_cert_pem`, `client_cert_pem`, `client_key_pem`, `MaterializedTlsFiles`, TLS-specific `must be path-backed` branches, and managed TLS copy file names
6. Run the required gates in repo order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. If execution reveals a consumer that still requires a second TLS DTO or regenerated TLS files, switch this task back to `TO BE VERIFIED`, document the exact caller and missing shared path-only shape here, and stop immediately.
</plan>
