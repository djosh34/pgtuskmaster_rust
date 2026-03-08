# pgtuskmaster

Runs a pgtuskmaster high-availability PostgreSQL node.

## Synopsis

`pgtuskmaster --config <PATH>`

## Options

`--config <PATH>`

:   Path to the runtime configuration file. The program exits with code `2` if this option is omitted.

`-h, --help`

:   Display help information.

## Description

`pgtuskmaster` is the daemon binary for a PGTuskMaster node. It parses the CLI arguments, requires an explicit runtime config path, builds a Tokio runtime, loads the config from disk, validates it, bootstraps logging, executes startup planning, and then runs the worker set for the node.

The daemon CLI surface is intentionally small. Runtime behavior is configured through the runtime config file rather than through many command-line flags or subcommands.

## Exit Codes

`0`

:   The runtime returned success.

`1`

:   Tokio runtime construction failed or the node runtime returned an execution error.

`2`

:   `--config <PATH>` was omitted.

## Related References

- [pgtuskmasterctl CLI](pgtuskmasterctl-cli.md)
- [Runtime Configuration](runtime-configuration.md)
