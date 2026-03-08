Target docs path: `docs/src/reference/runtime-configuration.md`
Diataxis type: reference
Why this is the next doc:
- Only one reference page exists (implicitly via CLI usage), but no structured reference for runtime config schema
- The tutorial and how-to both depend on TOML configs (node-a/b/c) but no page describes the full schema
- Reference is the only Diátaxis quadrant not yet represented with a dedicated page
- Users need to know available config keys, types, defaults, and validation rules to actually run the system beyond the docker example

Exact additional information needed:
- file: `src/config/schema.rs`
  why: Defines the top-level Config struct and all nested sub-configs (DbConfig, DcsConfig, HaConfig, ApiConfig, TlsConfig, LoggingConfig); contains field names, types, and serde attributes that map to TOML keys
- file: `src/config/defaults.rs`
  why: Shows default values for each configuration key when not specified in TOML
- file: `docker/configs/cluster/node-a/runtime.toml`
  why: Concrete example of a complete runtime config used in the tutorial; demonstrates which keys are required vs optional in practice
- file: `src/config/parser.rs`
  why: Shows validation logic, required fields, and how config files are located and merged

Optional runtime evidence to generate:
- command: `cargo run --bin pgtuskmaster -- --help`
  why: Reveals CLI flags that override runtime config and how config file paths are specified
- command: `cargo run --bin pgtuskmasterctl -- --help`
  why: Lists available CLI commands and flags to confirm the reference scope for CLI is covered elsewhere
