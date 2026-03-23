## Bug: Legacy config cutover still blocks compile <status>not_started</status> <passes>false</passes>

<description>
Disabling the exports in `src/config/mod.rs` shows that production code still imports legacy `crate::config` types and helpers instead of using `config_v2`.

`cargo check` currently fails with unresolved imports from:
- `src/cli/client.rs`
- `src/cli/config.rs`
- `src/cli/mod.rs`
- `src/logging/postgres_ingest.rs`
- `src/postgres_roles.rs`
- `src/tls.rs`

There are also remaining legacy-config dependencies in test support and migration helpers such as `src/dev_support/runtime_config.rs`, `src/dev_support/runtime_config_v2.rs`, and `src/dev_support/auth.rs`.

Explore and research the codebase first, then remove the remaining production dependency on `crate::config` so the runtime compiles against `config_v2` only.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
