# Configuration Reference

The `config` subsystem loads, normalizes, and validates the runtime configuration document consumed by the rest of the process.

## Module layout

| Module | Surface |
| --- | --- |
| `config::schema` | Deserialized configuration types for the accepted runtime shape and the rejected legacy shape |
| `config::parser` | File loading, version dispatch, v2 normalization, and validation |
| `config::defaults` | Default values used while normalizing partially specified inputs |

## Runtime configuration shape

`RuntimeConfig` contains these top-level sections:

| Section | Contents |
| --- | --- |
| `cluster` | Cluster name and local `member_id` |
| `postgres` | Data directory, listen settings, local and rewind identities, TLS, roles, `pg_hba`, `pg_ident`, and extra GUCs |
| `dcs` | Endpoint list, scope, and optional bootstrap init payload |
| `ha` | HA loop interval and lease TTL |
| `process` | Timeouts and binary paths for managed PostgreSQL operations |
| `logging` | Log level, subprocess capture, PostgreSQL log ingestion, and sink configuration |
| `api` | Listen address plus TLS and token-auth settings |
| `debug` | Debug API enablement flag |

## Input shapes

The loader accepts a versioned TOML document. `config_version = "v2"` is the supported input shape. `config_version = "v1"` is explicitly rejected and returns a validation error after a diagnostic probe of the legacy layout.

`InlineOrPath` is used for values that may be provided inline or by path. `SecretSource` wraps that surface for secret-bearing values and redacts inline content from `Debug` output.

The v2 input schema is normalized into `RuntimeConfig`. Optional v2 fields are filled from defaults where the parser defines them, including logging, debug, process, and API defaults.

## Loading and normalization

`load_runtime_config(path)` performs three steps:

1. Read the TOML file from disk.
2. Parse the `config_version` envelope and reject missing or unsupported versions.
3. Parse `RuntimeConfigV2Input`, normalize it into `RuntimeConfig`, and run `validate_runtime_config`.

Normalization resolves the explicit secure v2 input into the concrete runtime shape. This includes materializing required role and connection identity blocks, building the `RuntimeConfig` tree, and validating `postgres.extra_gucs` entries during normalization.

## Validation surface

`validate_runtime_config` enforces these invariants:

| Area | Invariants |
| --- | --- |
| Required values | Non-empty strings and non-empty paths for required runtime fields |
| Binary paths | Every managed PostgreSQL binary path is present and absolute |
| Timeouts | Process and log polling timeouts stay within `1..=86_400_000` milliseconds |
| Role and connection identity | Local connection user matches the configured superuser; rewind connection user matches the configured rewinder role |
| PostgreSQL auth and TLS | Role auth currently accepts only `password`; TLS-required SSL modes are rejected when PostgreSQL TLS is disabled |
| PostgreSQL files | `pg_hba`, `pg_ident`, log paths, and optional TLS identity/client CA inputs are non-empty when configured |
| Logging path ownership | The file sink cannot point at tailed PostgreSQL log inputs or live inside the tailed log directory |
| DCS and HA | DCS endpoints must be non-empty, scope must be non-empty, `ha.loop_interval_ms` must be non-zero, and `ha.lease_ttl_ms` must be greater than `ha.loop_interval_ms` |
| API security | Token-auth values must be non-empty when present, and role-token auth requires at least one token |
| DCS init payload | `dcs.init.payload_json` must be non-empty, valid JSON, and decodable as a `RuntimeConfig` JSON document |

## Error surface

`ConfigError` distinguishes three failure classes:

| Variant | Meaning |
| --- | --- |
| `Io` | Reading the configuration file failed |
| `Parse` | TOML deserialization failed |
| `Validation` | The decoded configuration violates a field-level or cross-field invariant |
