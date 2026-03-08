Target docs path: docs/src/reference/cli-pgtuskmasterctl.md
Diataxis type: reference
Why this is the next doc:
- Zero reference docs exist; reference is critical for "information-oriented" needs
- Users already invoke pgtuskmasterctl in tutorial and how-to without a command/flag reference
- Provides dry, authoritative machinery description that how-to guides deliberately omit
- Unblocks users transitioning from learning to independent work

Exact additional information needed:
- file: src/cli/args.rs
  why: contains clap-derived structs that define every CLI subcommand, flag, argument, and help text
- file: src/bin/pgtuskmasterctl.rs
  why: shows command dispatch and any runtime help/usage generation logic
- file: src/cli/mod.rs
  why: reveals module organization and any additional CLI entry points or helpers

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmasterctl -- --help
  why: captures actual top-level command listing and description
- command: cargo run --bin pgtuskmasterctl -- ha state --help
  why: captures subcommand-specific options and flags used in existing docs
