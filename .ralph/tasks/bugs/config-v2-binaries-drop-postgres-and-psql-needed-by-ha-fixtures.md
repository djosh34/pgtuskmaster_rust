## Bug: Config V2 Binaries Drop Postgres And Psql Needed By HA Fixtures <status>not_started</status> <passes>false</passes>

<description>
`src/config_v2/parser/load_config.rs` currently rejects `process.binaries.overrides.postgres` and `process.binaries.overrides.psql` as unsupported, and `config_v2::types::BinariesConfig` has no place to carry them. The ultra-long HA fixtures still depend on those binary overrides, so seed-primary bootstrap dies during config parsing before the cluster can start.

This was detected by `make test-long`, where multiple HA scenarios failed at harness bootstrap with `config error: invalid config field process.binaries.overrides.postgres: is not supported by config_v2`. Explore the current process/tool/runtime usage first, then extend the existing v2 binary shape directly instead of rebuilding an old process config mirror.
</description>

<acceptance_criteria>
- [ ] `src/config_v2::types::BinariesConfig` carries every process binary path still legitimately needed by runtime code and HA fixtures
- [ ] `src/config_v2/parser/load_config.rs` no longer rejects `process.binaries.overrides.postgres` / `psql` when those binaries are still part of the supported runtime surface
- [ ] No helper rebuilds old `ProcessConfig` or introduces a new mirror such as `ProcessConfigV2`
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
