Target docs path: docs/src/reference/pgtuskmaster-cli.md
Diataxis type: reference
Why this is the next doc:
- The reference section documents machinery; we have pgtuskmasterctl and runtime config but not the primary daemon binary
- The daemon CLI is essential for running the system and complements the config reference
- It follows the pattern of documenting each major interface: daemon CLI, client CLI, and config schema

Exact additional information needed:
- file: src/bin/pgtuskmaster.rs
  why: To identify daemon CLI arguments, environment variable handling, and startup behavior
- file: src/cli/args.rs
  why: To capture the complete clap derive structure and argument definitions
- file: src/config/mod.rs
  why: To understand config loading precedence and CLI override mechanisms
- extra info: Does the daemon have subcommands beyond config loading and version info?

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmaster -- --help
  why: To capture exact help text, flag names, descriptions, and defaults
