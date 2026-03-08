# pgtuskmaster

Runs a pgtuskmaster high-availability PostgreSQL node.

## Synopsis

`pgtuskmaster --config <PATH>`

## Options

`--config <PATH>`

:   Path to runtime configuration file (TOML). Required. The program exits with code 2 if this option is omitted.

`-h, --help`

:   Display help information.

## Description

`pgtuskmaster` is a long-running daemon process that manages a PostgreSQL instance in a high-availability cluster. It loads runtime configuration from the specified file, validates it, then starts a set of concurrent workers that manage database lifecycle, monitor cluster state, and expose APIs.

// todo: This startup sequence is too detailed for the supplied source set. Keep only the directly supported facts: config load, config validation, logging bootstrap, startup planning/execution, and worker startup.

The daemon performs these high-level steps on startup:

1. Load and validate configuration
2. Bootstrap logging subsystem
3. Plan and execute startup actions
4. Run worker loops

All operational behavior is controlled through the runtime configuration file rather than command-line flags.

## Exit codes

`0`

:   The runtime returned success.

`1`

:   Runtime error. Either Tokio runtime construction failed, or a worker encountered a fatal error.

`2`

:   Missing required `--config` option.

## Related references

- [pgtuskmasterctl CLI](pgtuskmasterctl-cli.md) - Administrative CLI for the API
- [Runtime Configuration](runtime-configuration.md) - Configuration file schema and options
