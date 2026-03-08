Target docs path: docs/src/reference/runtime-configuration.md
Diataxis type: reference
Why this is the next doc:
- The project has a runtime configuration schema in src/config/schema.rs but no authoritative reference document
- Users deploying clusters need complete configuration documentation beyond the minimal examples in docker/configs/
- The tutorial references runtime.toml files but doesn't explain the available options, validation rules, or defaults
- This fills a critical reference gap for application of skill (deploying/configuring)

Exact additional information needed:
- file: src/config/schema.rs
  why: Contains the Rust struct definitions for all runtime configuration options, their types, and structure
- file: src/config/defaults.rs
  why: Defines default values where applicable and helps identify which fields are optional vs required
- file: docker/configs/cluster/node-a/runtime.toml
  why: Real-world example showing all configuration sections used in a production-like three-node setup
- file: docker/configs/single/node-a/runtime.toml
  why: Minimal configuration example showing required fields only
- file: src/config/parser.rs
  why: Reveals validation logic, error messages, and parsing rules that must be documented as constraints
- file: docs/draft/docs/src/reference/runtime-configuration.md
  why: Previous draft attempt may contain useful structure or content to build upon

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmaster -- --help
  why: Would show runtime configuration file path expectations and CLI flags that relate to config loading
