# Why pgtuskmaster materializes managed PostgreSQL files

The boundary between operator-owned configuration and runtime-owned files is the central design choice that makes pgtuskmaster safe to run inside an already-provisioned PostgreSQL data directory. This page explains why the project materializes specific files under the data directory instead of assuming the operator will keep them untouched.

## Operator-owned versus runtime-owned split

pgtuskmaster treats the PostgreSQL data directory as a shared workspace with clear ownership zones. The operator supplies security-sensitive material such as users, roles, authentication rules, TLS certificates, and `pg_hba.conf` or `pg_ident` mappings. The runtime, in turn, must be able to enforce a working PostgreSQL configuration that matches the intended high-availability state: primary, replica, or recovery. Without the ability to write files that PostgreSQL reads at startup, the runtime could not reliably transition between those states or guarantee that critical settings such as `primary_conninfo` or `hot_standby` match the current cluster topology.

The split is enforced by convention and code: the `defaults` module is restricted to safe defaults only and is forbidden from synthesizing security-sensitive configuration. All materialization logic lives in `postgres_managed.rs` and `postgres_managed_conf.rs`, where `materialize_managed_postgres_config` writes managed runtime artifacts and quarantines any `postgresql.auto.conf` that might conflict.

## “Managed” means controlled materialization, not reinvention

Materializing a managed file does not mean pgtuskmaster invents its content from scratch. Managed files are derived from operator-supplied configuration, runtime state, and safe defaults. For example:

- `pg_hba.conf` and `pg_ident.conf` are rendered from configured sources; the operator retains full control over auth rules.
- `postgresql.auto.conf` is quarantined because its presence can override the managed configuration in ways that break startup determinism.
- TLS files are copied into managed runtime paths only when TLS is enabled; the operator must supply production certificates, keys, and CA bundles.
- Standby passfiles and recovery signal files are materialized only when needed for replica or recovery intent.

The managed `postgresql.conf` header states explicitly that the file is managed by pgtuskmaster, removes backup-era archive and restore settings, and documents that production TLS material must be supplied by the operator. This header is not decorative: it signals to human operators and tools that the file should not be hand-edited because the runtime rewrites it on every configuration pass.

## Reserved GUC keys and deterministic startup

Extra GUC keys exist because certain PostgreSQL settings cannot be allowed to drift at runtime. The reserved set includes:

- `listen_addresses` – must match the service advertisement address.
- `hba_file`, `ident_file` – must point to the managed paths to guarantee auth rules are applied.
- `primary_conninfo`, replication slot settings – must match the current replication topology.
- Restore settings – must be removed from backup-era configurations to avoid accidental restore loops.
- TLS file settings – must be set to the managed runtime paths when TLS is enabled.

By reserving these keys, the runtime prevents accidental overrides from leftovers in `postgresql.auto.conf` or human edits that would cause startup to behave differently from the declared intent.

## TLS copying versus generation

pgtuskmaster never generates certificates. When TLS is enabled, the operator supplies the certificate, key, and CA bundle. The runtime copies those files into predictable locations under the data directory and updates `ssl_cert_file`, `ssl_key_file`, and `ssl_ca_file` to point to those copies. This indirection benefits HA behavior: the paths remain stable across state transitions, and PostgreSQL restart logic does not need to chase symbolic links or environment variables.

## Tradeoffs and failure modes

The managed boundary trades flexibility for determinism. Because the runtime rewrites the managed set on every pass, hand-editing those files is futile and can mask real configuration problems. Operators must use the configuration API rather than the file system to change managed behavior. Failure modes include:

- If the operator places required TLS material outside the operator-owned paths and the runtime cannot read it, materialization fails and PostgreSQL will not start.
- If `postgresql.auto.conf` is not quarantined, a leftover `primary_conninfo` or `restore_command` can cause a primary to follow a replica or enter an unintended restore loop.
- If auth files are not managed, a replica may start with stale `pg_hba.conf` and reject connections from the primary or monitoring agents.

## Determinism in startup and HA

`ManagedPostgresStartIntent` (Primary, Replica, Recovery) drives rendering of `primary_conninfo` and `hot_standby`. Because this intent is explicit and the managed files are materialized before PostgreSQL starts (or restarts), every startup follows the same code-path and reads the same file set. That determinism is essential for reliable failover and for operators who need to reason about cluster behavior from logs and configuration state alone.

## Related pages

- [Managed PostgreSQL runtime files](managed-postgres-runtime-files.md) – lists every file written under the data directory.
- [Managed PostgreSQL configuration](managed-postgres-configuration.md) – explains the configuration API that feeds the materializer.
- [Runtime configuration](runtime-config.md) – how operator intent becomes runtime parameters.
- [TLS reference](tls.md) – requirements and paths for production TLS material.
