Target docs path: `docs/src/reference/cli-commands.md`

Diataxis type: reference

Why this is the next doc:
- Existing tutorial and how-to already use `pgtuskmasterctl` commands but leave operators without complete command/flag inventory
- CLI is the primary operational interface; operators need authoritative command syntax, not just examples
- Reference quadrant demands machine-like description of the machinery; CLI args/structure are literal machinery
- How-to guide currently embeds partial command reference (`ha state`) which should link to a dedicated reference page
- Lacks dry, systematic listing of all subcommands, flags, and output schemas that reference requires

Exact additional information needed:
- file: `src/cli/args.rs`
  why: contains Cli, HaArgs, and all subcommand enums with their flags/descriptions
- file: `src/bin/pgtuskmasterctl.rs`
  why: shows top-level CLI structure and help text integration points
- file: `src/cli/output.rs`
  why: defines JsonState and TextState response structs and formatting logic
- file: `src/api/mod.rs` or api route definitions
  why: maps CLI subcommands to actual HTTP API endpoints and paths
- file: `src/runtime/node.rs` or similar containing runtime config schema
  why: CLI interacts with runtime; reference should show config context
- extra info: Confirm whether CLI supports authentication flags (`--read-token`, `--admin-token`) beyond base-url and output format
- extra info: Are there other top-level command groups beyond `ha` (e.g., debug, config, init)?

Optional runtime evidence to generate:
- command: `cargo run --bin pgtuskmasterctl -- --help`
  why: captures exact top-level usage, commands list, and global flags
- command: `cargo run --bin pgtuskmasterctl -- ha --help`
  why: captures HA subcommand tree and descriptions
- command: `cargo run --bin pgtuskmasterctl -- ha state --help`
  why: captures flags, options, and help text for the state command
- command: `cargo run --bin pgtuskmasterctl -- --output json ha state` (against a running node)
  why: provides actual JSON schema example to document field types and semantic meaning (vs guessing from code)
