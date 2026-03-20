## Bug: Config v2 operator config rejects expected_transport needed by HA harness <status>not_started</status> <passes>false</passes>

<description>
`make test-long` currently fails during HA harness bootstrap because `src/config_v2/parser/load_operator_config.rs` rejects `pgtm.api.expected_transport` as unsupported, even though the config_v2 private schema still accepts that field and the HA observer support code still materializes it.

Explore the operator config_v2 type graph first, confirm whether `expected_transport` should survive on `OperatorConfigV2` or be collapsed into an existing shared shape, and then fix the parser/type boundary without reintroducing legacy operator config adapters.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
