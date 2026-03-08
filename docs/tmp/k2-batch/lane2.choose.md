Target docs path: `docs/src/reference/pgtuskmasterctl-cli.md`

Diataxis type: reference

Why this is the next doc:
- No reference documentation exists yet; this fills the information-oriented gap
- The existing how-to (`check-cluster-health.md`) already demonstrates two `pgtuskmasterctl` commands but lacks exhaustive coverage
- A complete CLI reference is essential for operators to discover all subcommands, flags, and output formats
- The codebase has a dedicated CLI module with structured argument parsing that can be mapped directly to reference sections
- This reference can be cross-linked from both the tutorial and how-to guides, strengthening the documentation map

Exact additional information needed:
- file: `src/cli/args.rs` - why: contains all Clap-derived CLI arguments, subcommands, and help text that define the exact surface of `pgtuskmasterctl`
- file: `src/bin/pgtuskmasterctl.rs` - why: shows top-level command structure and any additional CLI entry-point logic not captured in the args module
- file: `src/cli/output.rs` - why: defines output formatters (json, text) and field lists that must be documented
- file: `tests/cli_binary.rs` - why: provides real-world command invocations and expected outputs for verification

Optional runtime evidence to generate:
- command: `cargo run --bin pgtuskmasterctl -- --help`  
  why: captures the exact top-level usage, subcommands, and global flags as users will see them
- command: `cargo run --bin pgtuskmasterctl -- ha --help`  
  why: documents the `ha` subcommand group, which appears central to the tool
- command: `cargo run --bin pgtuskmasterctl -- ha state --help`  
  why: provides the definitive list of flags, options, and help text for the most-used command already shown in the how-to guide
