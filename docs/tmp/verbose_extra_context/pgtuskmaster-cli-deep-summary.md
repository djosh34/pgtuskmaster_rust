# Pgtuskmaster CLI Deep Summary

This file gathers only source-backed context for `docs/src/reference/pgtuskmaster-cli.md`.

## Binary shape

- The daemon binary entry point is `src/bin/pgtuskmaster.rs`.
- The clap command name is `pgtuskmaster`.
- The clap about text is `Run a pgtuskmaster node`.
- Runtime help confirms the current public surface:
- `Usage: pgtuskmaster [OPTIONS]`
- `--config <PATH>`
- `-h, --help`
- No subcommands are defined in the binary entry point.

## Required config path behavior

- The clap field is `config: Option<PathBuf>`.
- Even though clap models the flag as optional, the program treats it as required at runtime.
- If `--config` is omitted, `run_node()` prints `missing required \`--config <PATH>\`` to stderr and exits with code `2`.
- The binary does not attempt any default config-file discovery.

## Runtime startup path

- `main()` parses CLI args and passes them to `run_node(cli)`.
- `run_node(cli)` constructs a Tokio multi-thread runtime with `worker_threads(4)` and `enable_all()`.
- If Tokio runtime construction fails, the program prints `failed to build tokio runtime: ...` and exits with code `1`.
- When `--config` is present, the binary calls `pgtuskmaster_rust::runtime::run_node_from_config_path(config.as_path())`.

## Config loading and validation

- `src/runtime/node.rs` defines `run_node_from_config_path(path)`.
- That function loads the config by calling `load_runtime_config(path)` and then forwards to `run_node_from_config(cfg)`.
- `run_node_from_config(cfg)` calls `validate_runtime_config(&cfg)` before bootstrapping logging and workers.
- This means the daemon binary performs both config parsing and config validation before starting worker loops.
- `src/config/mod.rs` re-exports the relevant config APIs: `load_runtime_config`, `validate_runtime_config`, and `ConfigError`.

## What the binary does after config validation

- After validation, runtime startup bootstraps logging.
- The runtime emits a startup event with cluster and logging metadata.
- Startup then plans initial actions, executes the startup plan, and finally runs the worker set.
- The binary reference should therefore describe `pgtuskmaster` as a long-running node process, not a one-shot utility command.

## Exit behavior

- Exit code `0` means `run_node_from_config_path()` returned success.
- Exit code `1` means Tokio runtime creation failed or the runtime returned an execution error.
- Exit code `2` is reserved for missing `--config`.

## Operator-facing implications

- Every invocation must pass an explicit runtime config path.
- The daemon CLI surface is intentionally small; operational behavior is configured through the runtime config file rather than through many flags or subcommands.
- The best companion references for this page are the runtime configuration reference and the `pgtuskmasterctl` CLI reference.
