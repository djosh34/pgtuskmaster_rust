# Prerequisites

Before first run, confirm that all runtime dependencies and binaries are present and reachable from the configured paths.

## Required components

- PostgreSQL 16 binaries (`postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, `psql`)
- etcd endpoint(s) reachable by the node
- writable local directories for PostgreSQL data, logs, and socket paths
- runtime config file with `config_version = "v2"`

## Environment preparation

- Keep data directory ownership and permissions aligned with PostgreSQL requirements.
- Keep socket directory paths short to avoid Unix socket path-length issues.
- Ensure secrets referenced by file path are readable by the runtime account.
- Ensure API listen address and PostgreSQL listen settings are valid for your topology.

## Why this matters

Most startup failures are not logic errors. They are dependency and path problems. Verifying prerequisites first reduces noisy debugging and makes lifecycle behavior easier to interpret.
