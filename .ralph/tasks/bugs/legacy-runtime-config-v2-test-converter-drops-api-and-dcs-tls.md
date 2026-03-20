## Bug: Legacy RuntimeConfigV2 Test Converter Drops API And DCS TLS <status>completed</status> <passes>true</passes>

<description>
`src/dev_support/runtime_config_v2.rs` currently converts legacy `RuntimeConfig` into `RuntimeConfigV2` for test and harness code, but it silently degrades TLS settings instead of preserving or rejecting them.

Today the converter maps legacy `api.transport = https` to `ApiTransport::Http`, and it maps legacy `dcs.client.tls = enabled` to `None`. That means a test or harness that asks for HTTPS or DCS TLS can accidentally run with weaker transport than requested while still appearing to succeed.

Explore the existing config-v2 TLS shapes and the dev-support callers first, then make the converter faithful. If a legacy TLS input cannot be represented in `RuntimeConfigV2`, return a clear error instead of silently downgrading it.
</description>

<acceptance_criteria>
- [x] `src/dev_support/runtime_config_v2.rs` either preserves legacy API/DCS TLS in `RuntimeConfigV2` or returns an explicit error for unsupported legacy TLS inputs
- [x] No test or harness path silently downgrades HTTPS or DCS TLS to plaintext/disabled transport
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- The bug is a wrong config-ingestion boundary inside [src/dev_support/runtime_config_v2.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config_v2.rs): the legacy-to-v2 adapter already owns the one-time conversion from test-only `RuntimeConfig` into validated `RuntimeConfigV2`, but it currently discards TLS intent by mapping `ApiTransportConfig::Https` to `ApiTransport::Http` and `DcsTlsConfig::Enabled` to `None`.
- The shared v2 type graph is already sufficient for the supported cases. [src/config_v2/types.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/types.rs) has `ApiTransport::Https { tls, client_ca, client_cert_required, allowed_client_common_names }` and `DcsConfig.tls: Option<TlsConfig>`, and [src/config_v2/parser/load_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_config.rs) already defines the truthful validation rules for those shapes.
- That means this task does not need a new enum, mirror struct, or another helper layer. The right reduction is to reuse the existing `TlsConfig`/`ApiTransport` shapes and make the legacy adapter either populate them faithfully or return the same class of validation error that config-v2 already returns for unsupported legacy inputs.
- The unsupported legacy DCS cases are already clear from the v2 parser and should stay explicit here too: `dcs.client.tls.server_name` is not supported by config-v2, and enabled DCS TLS without a client identity is not representable because current v2 DCS TLS requires certificate/key material.
- Inline test credentials should continue to be materialized through the existing `materialize_path_or_inline` boundary rather than introducing parallel TLS-file writers. That keeps one helper for path-or-inline conversion and avoids duplicating filesystem behavior.

### Execution plan
1. Keep the current type graph and reduce the bad adapter boundary in [src/dev_support/runtime_config_v2.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config_v2.rs) instead of adding new config structs or enums.
2. Replace the silent downgrade branches with faithful mapping:
   - map legacy `api.transport = http` to `ApiTransport::Http`;
   - map legacy `api.transport = https` to `ApiTransport::Https`, preserving identity, optional client CA, required-vs-optional client certificate mode, and allowed client common names by reusing the existing v2 fields;
   - map legacy `dcs.client.tls = disabled` to `None`;
   - map legacy `dcs.client.tls = enabled` to `Some(TlsConfig)` when the legacy input is representable in v2.
3. Mirror the existing config-v2 validation boundary for unrepresentable legacy DCS TLS instead of weakening it:
   - return a clear error if `dcs.client.tls.server_name` is set;
   - return a clear error if DCS TLS is enabled without a client identity;
   - do not silently strip CA or identity inputs that the v2 type can carry.
4. Keep code reduction as the constraint while implementing:
   - prefer one direct `match` per legacy TLS edge over new wrapper functions;
   - reuse `TlsConfig`, `ApiTransport`, `materialize_path_or_inline`, and existing secret/path resolvers;
   - do not reintroduce `crate::config::RuntimeConfig` anywhere downstream once conversion is complete.
5. Add focused regression coverage around the adapter boundary:
   - a converter test proving legacy API HTTPS survives as v2 HTTPS, including client-auth details when configured;
   - a converter test proving representable legacy DCS TLS survives as `cfg.dcs.tls`;
   - negative tests proving unsupported `dcs.client.tls.server_name` and missing DCS client identity now return explicit errors instead of downgrading to plaintext.
6. Run the required validation gates in repo order: `make check`, `make lint`, `make test`, and `make test-long`. If execution shows the current v2 TLS types are still insufficient, switch this task back to `TO BE VERIFIED`, explain the exact representability gap in this file, and stop immediately.
7. Only after every required gate passes:
   - tick the acceptance boxes that are truly complete,
   - set `<passes>true</passes>`,
   - run `/bin/bash .ralph/task_switch.sh`,
   - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
   - push with `git push`,
   - stop immediately.

NOW EXECUTE
