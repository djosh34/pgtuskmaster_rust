# Why pgtuskmaster materializes managed PostgreSQL files

pgtuskmaster draws a sharp line between operator-supplied inputs and runtime-owned PostgreSQL files. That boundary is what lets the system stay deterministic during startup and HA transitions without pretending to be a certificate authority or a generic PostgreSQL configuration editor.

The [managed PostgreSQL runtime files reference](../reference/managed-postgres.md), [managed PostgreSQL configuration reference](../reference/managed-postgres-conf.md), and [runtime configuration reference](../reference/runtime-config.md) describe the surfaces involved. This page explains why they are split that way.

## The operator owns sensitive inputs

The defaults layer is intentionally limited to safe defaults. It must not synthesize users, roles, authentication rules, TLS posture, `pg_hba`, or `pg_ident`.

That is a design statement, not an omission. Security-sensitive material belongs to operator intent and provisioning, not to runtime guesswork.

## The runtime owns the managed file set

At the same time, the runtime does not leave local PostgreSQL state to drift. `materialize_managed_postgres_config` writes managed HBA and ident files from configured sources, renders a managed `postgresql.conf`, materializes TLS files when enabled, writes standby passfiles when needed, creates or removes recovery signal files, and quarantines `postgresql.auto.conf`.

"Managed" therefore means controlled materialization of the runtime-owned file set. It does not mean pgtuskmaster invents every PostgreSQL setting from scratch, and it does not mean arbitrary local edits are treated as part of the desired state.

## Why some settings are reserved

The managed config layer reserves critical GUC keys such as listen addresses, HBA and ident file paths, recovery settings, slot settings, and TLS file paths. Those settings are part of the runtime contract. If they were left open to ad hoc override, startup intent and HA actions could no longer rely on the filesystem reflecting the runtime's view of the node.

The same logic explains why the managed config header explicitly states that production TLS material must be supplied by the operator while pgtuskmaster only copies managed runtime files into place.

## Why this helps HA and startup

`ManagedPostgresStartIntent` distinguishes primary, replica, and recovery paths. That intent directly controls the rendered recovery posture and related files. By tying managed files to start intent, the runtime avoids inheriting ambiguous leftover state from an older boot.

Quarantining `postgresql.auto.conf` fits the same philosophy. The runtime would rather make ownership explicit than silently merge with unknown local state and hope the result still matches HA expectations.

## The tradeoff

This boundary is stricter than traditional hand-managed PostgreSQL practice. Manual edits to managed files are not a durable control surface. The cost is reduced flexibility for local tinkering. The benefit is that startup and failover behavior remain anchored to one declared runtime model instead of a mixture of operator intent, historical residue, and accidental local edits.
