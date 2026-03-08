Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft the page from scratch. The earlier attempt returned no content.

[Output path]
- docs/src/explanation/managed-postgres-boundary.md

[Page title]
- # Why pgtuskmaster materializes managed PostgreSQL files

[Audience]
- Readers who know PostgreSQL configuration basics and want to understand why this project writes managed files instead of assuming untouched operator files.

[User need]
- Understand the boundary between operator-supplied secure inputs and runtime-managed PostgreSQL files under the data directory.

[Diataxis guidance]
- Explanation page only.
- Focus on rationale, constraints, and consequences.

[Verified facts that are true]
- The defaults module is intentionally restricted to safe defaults only and must not synthesize security-sensitive material such as users, roles, auth, TLS posture, pg_hba, or pg_ident.
- materialize_managed_postgres_config writes managed HBA and ident files from configured sources, writes a managed postgresql config, materializes TLS files when enabled, writes standby passfiles when needed, creates or removes recovery signal files, and quarantines postgresql.auto.conf.
- The managed PostgreSQL config header states that the file is managed by pgtuskmaster, removes backup-era archive and restore settings, and says production TLS material must be supplied by the operator while pgtuskmaster only copies managed runtime files.
- ManagedPostgresStartIntent distinguishes Primary, Replica, and Recovery cases, which drive hot_standby and primary_conninfo rendering.
- Extra GUC keys reserve critical settings such as listen addresses, hba_file, ident_file, primary_conninfo, slot settings, restore settings, and TLS file settings.

[Relevant repo grounding]
- src/config/defaults.rs
- src/postgres_managed.rs
- src/postgres_managed_conf.rs

[Required structure]
- Explain the operator-owned versus runtime-owned split.
- Explain why "managed" here means controlled materialization, not total reinvention.
- Discuss tradeoffs and failure modes of this boundary.
- Mention how this helps startup and HA remain deterministic.

[Facts that must not be invented or changed]
- Do not claim pgtuskmaster generates certificates.
- Do not claim the project edits arbitrary PostgreSQL files outside the managed set.
