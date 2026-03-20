## Bug: CLI client drops file-backed API TLS material <status>completed</status> <passes>true</passes>

<description>
The CLI API client accepts `CliTlsConfig` with CA/client certificate/client key paths, but `src/cli/client.rs` only consumes the inline PEM byte fields and silently ignores the file-backed fields populated by operator config loading.

This was detected while debugging `make test-long`: the HA harness materialized correct HTTPS observer configs and the node API was listening, but `pgtm status` still failed with `transport error: error sending request for url (...)` because the CLI never loaded the configured CA or client identity from disk.

Explore and research the codebase first, then fix.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- `src/cli/client.rs` already loads API TLS material from either inline PEM bytes or configured filesystem paths through `read_optional_tls_bytes()`. Focused `cargo nextest` coverage confirmed the HTTPS client succeeds with CA/client certificate/client key paths.
- `src/cli/config.rs` already preserves operator-config TLS paths into `CliTlsConfig`, and focused `cargo nextest` coverage confirmed those path fields survive config loading for both API and Postgres client use.
- That means the task description is stale at the exact point where it says `src/cli/client.rs` only consumes inline PEM fields. The named symptom may already be fixed on this branch.
- The remaining real boundary smell is that one `CliTlsConfig` still serves two downstream consumers with different guarantees:
  - `apply_tls_config()` can consume inline bytes or paths for the reqwest API client;
  - `build_connection_tls()` in `src/cli/connect.rs` can only render Postgres conninfo from path-backed material and still branches on source form late.
- If the full repo gates still fail, the likely remaining bug is not "reqwest never reads files" anymore; it is a stale caller path or shared TLS shape that still forces late inline-vs-path branching after config ingestion.

### Execution plan
1. Start with verification, not refactor churn: run `make check`, `make lint`, `make test`, and `make test-long` in repo order.
2. If every gate passes, treat this bug as already fixed by the current tree. Tick the acceptance boxes, set `<passes>true</passes>`, switch the task, commit, and push without inventing extra changes.
3. If a gate still fails with this bug's symptom, keep the fix focused on the TLS boundary rather than layering more helpers around `CliTlsConfig`.
4. Flatten the shared TLS shape at `resolve_operator_context()`:
   - keep the API client path on the current reqwest-facing TLS builder path; and
   - convert the Postgres connection path directly into the existing path-only `PgClientTls` shape instead of reusing `CliTlsConfig`.
5. Remove late source-form branching from `build_connection_tls()` by making the operator-context output already match the downstream Postgres conninfo type.
6. Add or adjust only the narrow regressions needed by execution:
   - path-backed API TLS still works end-to-end for the HTTPS client; and
   - operator-context Postgres TLS conversion produces the exact path-backed conninfo inputs needed by `pgtm primary` / `pgtm replicas`.
7. If execution shows the current type split is still wrong in a deeper way than this diagnosis covers, switch this task back to `TO BE VERIFIED`, explain the exact remaining design gap here, and stop immediately.

### Constraints for execution
- Prefer reusing existing types over adding another TLS wrapper. `PgClientTls` is already the correct path-only downstream shape for rendered conninfo.
- Reduce code instead of growing helpers. If `build_connection_tls()` becomes a pure passthrough or disappears after the boundary rewrite, delete it.
- Do not reintroduce raw config-source distinctions downstream once ingestion has already resolved them.
- Do not run `cargo test`; use the required `make` targets, and use focused `cargo nextest` only for local repros if a gate actually fails.

NOW EXECUTE
