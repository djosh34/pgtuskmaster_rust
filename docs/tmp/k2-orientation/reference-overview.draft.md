# Reference

This chapter contains technical descriptions of PGTuskMaster machinery and how to operate it. Reference guides state, describe, and inform—they answer "What is...?" questions with dry, authoritative descriptions of interfaces, configurations, and data models.

- **[HTTP API Reference](reference/http-api.md)**: Programmatic access to cluster state, high-availability operations, and diagnostics. Documents authentication, authorization, and all API endpoints including switchover control, HA state queries, and fallback cluster operations.

- **[HA Decisions](reference/ha-decisions.md)**: Catalog of HA decision variants exposed through `GET /ha/state`. Describes each decision type (`no_change`, `wait_for_postgres`, `become_primary`, `step_down`, etc.) and their relationship to HA phases.

- **[Debug API](reference/debug-api.md)**: Read-only observability surface available when `debug.enabled` is true. Details `/debug/verbose`, `/debug/snapshot`, and `/debug/ui` endpoints, including response schemas and incremental polling mechanics.

- **[DCS State Model](reference/dcs-state-model.md)**: DCS-backed state structures, trust model evaluation rules, core type definitions (`MemberRecord`, `LeaderRecord`, `DcsCache`), key layout patterns, and cache update mechanisms.

- **[pgtuskmaster CLI](reference/pgtuskmaster-cli.md)**: Daemon binary synopsis, options (`--config`), exit codes, and behavior. Covers the minimal CLI surface and config-driven runtime execution.

- **[pgtuskmasterctl CLI](reference/pgtuskmasterctl-cli.md)**: HA admin client command reference. Documents global options, command hierarchy (`ha state`, `ha switchover`), authorization tokens, output formats, and exit codes with examples.

- **[Runtime Configuration](reference/runtime-configuration.md)**: Complete TOML schema specification with validation rules. Details all configuration sections: cluster identity, PostgreSQL instance settings, DCS endpoints, HA timing, process binaries, logging sinks, API parameters, and debug features.
