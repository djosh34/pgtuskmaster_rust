## Bug: Config v2 operator config rejects expected_transport needed by HA harness <status>completed</status> <passes>true</passes>

<description>
`make test-long` currently fails during HA harness bootstrap because `src/config_v2/parser/load_operator_config.rs` rejects `pgtm.api.expected_transport` as unsupported, even though the config_v2 private schema still accepts that field and the HA observer support code still materializes it.

Explore the operator config_v2 type graph first, confirm whether `expected_transport` should survive on `OperatorConfigV2` or be collapsed into an existing shared shape, and then fix the parser/type boundary without reintroducing legacy operator config adapters.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Verified design
- `expected_transport` should remain on `OperatorConfigV2`. It is validated operator-client state that the CLI needs for URL-scheme enforcement, and it should not be collapsed into `ApiTransport` because the runtime API enum carries server-side TLS material that operator config does not own.
- The real boundary smell is duplication and stale task framing: the current v2 parser already preserves `expected_transport` for standalone operator documents, so execution should focus on proving the runtime-document (`[pgtm.api]`) ingestion path and reducing any leftover conversion/helper noise instead of reintroducing legacy config adapters.
- Keep the raw/private serde enum only at the TOML ingestion edge. The validated v2 boundary should continue to expose exactly one shared `expected_transport` field on `OperatorConfigV2`.

### Execution plan
1. Re-check the operator config ingestion boundary across [src/config_v2/parser/load_operator_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_operator_config.rs), [src/config_v2/parser/private_schema.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/private_schema.rs), [src/config_v2/types.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/types.rs), and [src/cli/config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/config.rs), and keep a single validated `expected_transport` field on `OperatorConfigV2`.
2. Reduce the conversion edge instead of adding another mirror:
   - inline or otherwise collapse the tiny `expected_transport` raw-to-validated mapping so `load_operator_config` performs the conversion once at ingestion;
   - do not introduce a new config-only transport enum and do not route this through legacy `crate::config::PgtmConfig`.
3. Add focused regression coverage for both accepted operator document shapes:
   - standalone operator config using `[api]`;
   - full runtime document using `[pgtm.api]`;
   - CLI/operator-context resolution rejecting a base URL whose scheme violates `expected_transport`.
4. If execution shows the runtime-document path already works, keep the smallest reduction plus coverage change that proves the boundary. If execution shows the validated operator type graph is still wrong, switch this task back to `TO BE VERIFIED`, explain the exact mismatch in this file, and stop immediately.
5. Run the required validation gates in repo order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
6. Only after every gate passes:
   - tick the acceptance boxes that are truly complete,
   - set `<passes>true</passes>`,
   - run `/bin/bash .ralph/task_switch.sh`,
   - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
   - push with `git push`,
   - stop immediately.

NOW EXECUTE
