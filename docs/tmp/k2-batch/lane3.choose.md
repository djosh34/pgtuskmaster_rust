Target docs path: docs/src/reference/cli.md
Diataxis type: reference
Why this is the next doc:
- Completes the core documentation set after existing tutorial and how-to
- CLI is the primary operator interface and requires authoritative description
- Existing docs demonstrate usage but lack comprehensive command/flag inventory
- Reference material serves practitioners applying craft (action/cognition quadrant)
- Respects machinery structure by deriving docs directly from source definitions

Exact additional information needed:
- file: src/cli/args.rs
  why: Contains clap command/flag definitions with help text; needed to enumerate all subcommands, arguments, options, and their types/descriptions without invention
- file: src/cli/client.rs
  why: Shows API client implementation mapping CLI commands to HTTP endpoints; needed to document exact API paths, methods, and request parameters
- file: src/cli/output.rs
  why: Implements JSON and text formatters; needed to document output schemas, field names, and value semantics for each command
- file: src/bin/pgtuskmasterctl.rs
  why: Main entry point and environment variable handling; needed to document env var names, sources, and precedence rules
- file: tests/cli_binary.rs
  why: Contains integration test invocations and expected outputs; provides concrete usage examples and sample outputs for documentation
- file: docker/configs/cluster/node-a/runtime.toml
  why: Shows example configuration that defines API address and auth settings; needed to document default base-url behavior and authentication token requirements

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmasterctl -- --help
  why: Captures auto-generated top-level help text to verify command structure and global flags
- command: cargo run --bin pgtuskmasterctl -- ha --help
  why: Shows subcommand help to document nested command hierarchy and HA-specific options
- command: PGTUSKMASTERCTL_BASE_URL=http://127.0.0.1:18081 cargo run --bin pgtuskmasterctl -- ha state 2>&1 || true
  why: Demonstrates environment variable usage and captures actual error output format when API is unreachable
