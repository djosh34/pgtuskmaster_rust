# Current Tasks Summary

Generated: Mon Mar 23 11:05:16 PM CET 2026

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

# Task `.ralph/tasks/bugs/legacy-config-cutover-still-blocks-compile.md`

```
## Bug: Legacy config cutover still blocks compile <status>not_started</status> <passes>false</passes>

<description>
Disabling the exports in `src/config/mod.rs` shows that production code still imports legacy `crate::config` types and helpers instead of using `config_v2`.
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

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/03-delete-old-config.md`

```
## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>not_started</status> <passes>false</passes>

<description>
Migrate the final code to src/config_v2, while not making ANYTHING new public inside src/config_v2/parser
All validation, in ALL code (must verify this), but only be done once, and that must be done only inside src/config_v2/parser.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/04-eliminate-tls-bytes.md`

```
## Task: Eliminate TLS bytes <status>not_started</status> <passes>false</passes>

<description>
Someone previously had this crazy idea to store bytes within the config struct. This causes a crazy amount of issues.
I don't want that at all. I don't want tls bytes to be inside the config struct, nor i ever want them to be written somewhere else.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/05-reduce-loop.md`

```
## Task: Reduce Code Loop <status>not_started</status> <passes>false</passes>

Your only goal is to reduce code and clean up the codebase.

use just-reduce-code skill
```

