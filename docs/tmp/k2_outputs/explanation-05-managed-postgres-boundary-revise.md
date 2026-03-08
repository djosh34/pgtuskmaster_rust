# Why pgtuskmaster materializes managed PostgreSQL files

pgtuskmaster draws a clear line between configuration the operator supplies and files it fully controls at runtime. This split exists to keep behavior predictable during startup and failover while leaving security-sensitive material under operator control.

## Operator-owned versus runtime-owned split

The operator owns:
- TLS certificate and key material. The managed PostgreSQL config header explicitly states that production TLS must be supplied by the operator. pgtuskmaster copies these files into place but never generates them.
- Initial secrets such as passwords, user definitions, roles, and authentication rules.

The runtime owns:
- `pg_hba.conf` and `pg_ident.conf`, which are regenerated from configured sources by `materialize_managed_postgres_config`.
- The managed `postgresql.conf`, which is written on every start.
- TLS files copied from operator sources when enabled.
- Standby passfiles created as needed.
- Recovery signal files created or removed as required.
- `postgresql.auto.conf`, which is quarantined to prevent accidental interference.

The `defaults` module enforces this boundary: it contains only safe defaults and intentionally refuses to synthesize users, roles, auth rules, TLS posture, `pg_hba`, or `pg_ident`.

## What "managed" means

"Managed" does not mean pgtuskmaster reinvents PostgreSQL configuration. It means the project materializes a deterministic, minimal set of files required for reliable startup and high availability. The managed `postgresql.conf` header declares the file is controlled by pgtuskmaster and strips legacy backup-era archive and restore settings that would conflict with current operation. Everything outside this narrow set remains untouched.

## Tradeoffs and failure modes

This boundary trades flexibility for determinism. Because `materialize_managed_postgres_config` reserves critical GUC keys, including listen addresses, `hba_file`, `ident_file`, `primary_conninfo`, slot settings, restore settings, and TLS file paths, manual edits to the managed files are overwritten on every run. Operators must supply configuration through the project's API rather than by editing files directly.

Failure modes include:
- Operator-provided TLS material missing or invalid: startup fails cleanly rather than generating insecure defaults.
- `postgresql.auto.conf` containing conflicting settings: it is quarantined to avoid non-deterministic merges.
- Manual edits to managed files: changes are lost on the next start, which can surprise operators who expect traditional PostgreSQL behavior.

## Deterministic startup and HA

`ManagedPostgresStartIntent` (Primary, Replica, Recovery) drives rendering of `hot_standby` and `primary_conninfo`. By coupling these settings to the start intent, pgtuskmaster ensures that each role starts with the correct recovery configuration and replication topology. This eliminates the guesswork of whether a previous `postgresql.auto.conf` or stray recovery file will alter behavior, making startup and failover reproducible across nodes.
