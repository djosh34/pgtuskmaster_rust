# Current Tasks Summary

Generated: Fri Mar 20 08:52:25 PM CET 2026

# Task `.ralph/tasks/bugs/api-worker-reload-certificates-tests-leak-managed-postgres-processes.md`

```
## Bug: Api Worker Reload Certificates Tests Leak Managed Postgres Processes <status>not_started</status> <passes>false</passes>

<description>
`make test` currently exits zero, but nextest reports five leaky tests in `api::worker::tests::reload_certificates_*`:
```

==============

# Task `.ralph/tasks/bugs/config-v2-dcs-https-endpoints-miss-required-tls-validation.md`

```
## Bug: Config V2 DCS HTTPS Endpoints Miss Required TLS Validation <status>not_started</status> <passes>false</passes>

<description>
`src/config_v2/parser/load_config.rs` currently accepts `dcs.endpoints` entries with the `https://` scheme even when `dcs.client.tls` is absent. The legacy parser rejected that configuration with a stable `dcs.client.tls` validation error, but the config-v2 path now lets startup proceed until it fails later in unrelated runtime preparation.
```

==============

# Task `.ralph/tasks/bugs/config-v2-drops-replica-source-tls-ca-needed-by-basebackup.md`

```
## Bug: Config v2 drops replica source TLS CA needed by basebackup <status>not_started</status> <passes>false</passes>

<description>
Ultra-long HA scenarios now progress past seed-primary bootstrap, but replica startup still fails because `pg_basebackup` is launched without the configured TLS root CA.
```

==============

# Task `.ralph/tasks/bugs/config-v2-ha-bootstrap-pgtm-https-status-cannot-reach-seed-primary.md`

```
## Bug: Config v2 HA bootstrap pgtm HTTPS status cannot reach seed primary <status>not_started</status> <passes>false</passes>

<description>
`make test-long` currently fails in all 16 ultra-long HA scenarios during seed-primary bootstrap. The harness waits for the bootstrap primary, but `pgtm status` exits with a transport error while requesting `https://node-b:<port>/state`.
```

==============

# Task `.ralph/tasks/bugs/config-v2-parser-rejects-snake-case-pg-sslmode.md`

```
## Bug: Config v2 parser rejects snake_case postgres sslmode values <status>not_started</status> <passes>false</passes>

<description>
`config_v2::parser::load_config::tests::load_runtime_config_preserves_shared_source_client_tls` failed during `make test` because TOML parsing rejected `postgres.rewind.transport.ssl_mode = "verify_full"` with `unsupported sslmode`.
```

==============

# Task `.ralph/tasks/bugs/incremental-nextest-builds-drop-objects-and-rlib-members.md`

```
## Bug: Incremental nextest builds drop objects and rlib members <status>not_started</status> <passes>false</passes>

<description>
`make test` and focused `cargo nextest run` invocations can fail during the build phase with linker/archive errors such as `ld: cannot find ... .rcgu.o` or `failed to build archive ... .rlib: failed to open object file`. The failures point at missing members under `target/aarch64-unknown-linux-gnu/debug/deps` while `CARGO_INCREMENTAL=1` is enabled.
```

==============

# Task `.ralph/tasks/bugs/legacy-runtime-config-v2-test-converter-drops-postgres-tls.md`

```
## Bug: Legacy RuntimeConfigV2 Test Converter Drops Postgres TLS <status>not_started</status> <passes>false</passes>

<description>
`src/dev_support/runtime_config_v2.rs` currently converts legacy `RuntimeConfig` into `RuntimeConfigV2` for test and harness code, but it silently drops `postgres.tls` by always setting the v2 field to `None`.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/01-task-create-private-toml-schema-and-initial-runtimeconfigv2-root-handoff.md`

```
## Task: Create Private TOML Schema And Initial `RuntimeConfigV2` Root Handoff <status>completed</status> <passes>false</passes>

<description>
**Goal:** Create the first concrete execution task for the config-v2 direct-cfg collapse story. The higher-order goal is to start the migration by making `src/config_v2/parser/private_schema.rs` the only TOML-parsable config shape, adding parse functions in the config-v2 loaders, and switching `src/runtime/node.rs` to root itself in `RuntimeConfigV2` only. This task intentionally does not finish the downstream migration. It must stop at the first compile-failing handoff once the remaining failures are only due to the old `src/config/` corridor that later tasks in this story will delete.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/10-task-rebuild-dev-support-and-tests-around-v2-config-only.md`

```
## Task: Rebuild `dev_support/` And Tests Around V2 Config Only <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove test/dev preservation of the old config tree by rebuilding helpers, builders, harnesses, and fixtures around `RuntimeConfigV2` and `OperatorConfigV2` only. The higher-order goal is to prevent tests from keeping `src/config/` alive after production code migrates.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/11-task-delete-src-config-and-prove-zero-config-dependencies-remain.md`

```
## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete `src/config/` entirely and prove that no code, tests, docs, or fixtures depend on it anymore. The higher-order goal is to close the story with a hard architectural proof instead of stopping at a partial migration.
```

