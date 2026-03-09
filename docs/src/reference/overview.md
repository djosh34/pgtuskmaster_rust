# Reference

This chapter contains technical descriptions of PGTuskMaster machinery and how to operate it. Reference guides state, describe, and inform. They answer "What is...?" questions with dry, authoritative descriptions of interfaces, configurations, and data models.

- [HTTP API Reference](http-api.md) - Programmatic access to cluster state, high-availability operations, and diagnostics. Documents authentication, authorization, and API endpoints including switchover control, HA state queries, and fallback cluster operations.
- [HA Decisions](ha-decisions.md) - Catalog of HA decision variants exposed through `GET /ha/state`, including their relationship to HA phases.
- [Debug API](debug-api.md) - Read-only observability surface behind `pgtm status -v` and `pgtm debug verbose`, including the underlying `/debug/verbose`, `/debug/snapshot`, and `/debug/ui` endpoints.
- [DCS State Model](dcs-state-model.md) - DCS-backed state structures, trust evaluation rules, key layout patterns, and cache update mechanisms.
- [pgtuskmaster CLI](pgtuskmaster-cli.md) - Daemon binary synopsis, options, exit codes, and runtime behavior.
- [pgtm CLI](pgtm-cli.md) - HA admin client command reference, including cluster status, connection helpers, and the CLI debug surfaces.
- [Runtime Configuration](runtime-configuration.md) - TOML configuration schema, validation rules, and defaulted sections.
