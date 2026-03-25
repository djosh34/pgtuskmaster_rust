## Plan: Flatten Operator Client TLS Input Surface

### Why this is the next verified reduction target

The operator config still exposes two mirrored TLS input branches even though the validated owner is already a single shared field:

- `src/config_v2/types.rs`
  - `OperatorConfigV2` already stores exactly one `client_tls: Option<PgClientTls>`.
- `src/config_v2/parser/private_schema.rs`
  - `OperatorDocument` still splits that same concept across `api.tls` and `postgres.tls`.
- `src/config_v2/parser/load_config.rs`
  - `merge_matching(...)` and `merge_operator_client_tls(...)` only exist to reconcile those two mirrored inputs back into the one runtime owner.

That is adapter-only structure. The CLI consumes one shared TLS bundle for both HTTPS API access and PostgreSQL `--tls` DSN emission, so keeping two config branches only preserves duplication in parser code, docs, examples, and fixtures.

### Current overlap verified in code

1. The validated shape already has one owner.
   - `src/config_v2/types.rs`
     - `OperatorConfigV2.client_tls` is the only surviving operator TLS field after parsing.

2. The parser still carries duplicated input structure solely to merge it away.
   - `src/config_v2/parser/private_schema.rs`
     - `OperatorApiConfig` contains `tls: ClientTlsInput`.
     - `OperatorPostgresConfig` contains another `tls: ClientTlsInput`.
     - `OperatorDocument` keeps both owners alive.
   - `src/config_v2/parser/load_config.rs`
     - `merge_matching(...)` compares API and postgres TLS values field-by-field.
     - `merge_operator_client_tls(...)` rebuilds one `PgClientTls` from those two mirrored branches.

3. The duplicated input leaks into user-facing and test-facing surfaces.
   - `docs/examples/docker-cluster-node-{a,b,c}.toml`
     - carry identical `[api.tls]` and `[postgres.tls]` sections.
   - `docs/src/reference/runtime-configuration.md`
     - documents both `pgtm.api.tls` and `pgtm.postgres.tls` even though runtime keeps only one merged value.
   - `tests/ha/support/observer/pgtm.rs`
     - renders both blocks with the same values.
   - `src/config_v2/parser/load_config.rs`
     - parser tests assert the merge path instead of one direct owner.

### Execution plan

1. Collapse operator input onto the existing single owner.
   - In `src/config_v2/parser/private_schema.rs`, delete `OperatorPostgresConfig`.
   - Replace the split operator TLS inputs with one shared `client_tls: ClientTlsInput`.
   - Keep `OperatorConfigV2.client_tls` as the only validated operator TLS field; do not add a replacement wrapper.

2. Rewrite operator parsing to resolve one TLS block directly.
   - In `src/config_v2/parser/load_config.rs`, delete `merge_matching(...)` and `merge_operator_client_tls(...)`.
   - Resolve only:
     - `pgtm.client_tls.ca_cert`
     - `pgtm.client_tls.identity.cert`
     - `pgtm.client_tls.identity.key`
   - Build `OperatorConfigV2.client_tls` directly from that one input.

3. Remove the mirrored config surface everywhere it appears.
   - Update standalone operator config rendering to use `[client_tls]` instead of separate `[api.tls]` and `[postgres.tls]`.
   - Update embedded runtime `pgtm` rendering/docs/examples to use `[pgtm.client_tls]`.
   - Rewrite HA observer/operator fixtures and parser tests to provide one TLS block instead of two identical ones.

4. Keep behavior, delete compatibility.
   - Preserve current semantics:
     - HTTPS operator API calls still use `client_tls` when present.
     - `pgtm primary --tls` and `pgtm replicas --tls` still emit the same TLS material.
     - path-only TLS identity parsing remains enforced.
   - Do not support both old and new shapes; remove `pgtm.api.tls` and `pgtm.postgres.tls` outright.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff stays net-negative and improves beyond the current `+10045 -13859 diff: -3814` baseline.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse `OperatorConfigV2.client_tls` and `PgClientTls`; do not invent `UnifiedOperatorTlsConfig` or a second client TLS DTO.
- Keep operator auth, base URL, advertised URL, expected transport, and resolve-to behavior unchanged.
- Keep failure fields concrete and current by reporting only the new `pgtm.client_tls.*` paths.
- If deleting the split surface unexpectedly increases code because too many tests depend on hand-built section overlays, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete parser-only merge helpers whose sole job is reconciling duplicate operator TLS inputs.
- Delete `OperatorPostgresConfig` and the second operator TLS branch from the raw schema.
- Shorten docs, examples, and fixtures by removing repeated TLS sections.
- Leave one operator client-TLS concept end to end: input `client_tls`, validated `client_tls`, and CLI consumption of `client_tls`.

NOW EXECUTE
