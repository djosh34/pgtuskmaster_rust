# CLI stays outside the control loop

`pgtuskmaster` and `pgtuskmasterctl` are separate binaries with distinct responsibilities. The node process runs a control loop inside a Tokio runtime, initialized from a configuration file. The CLI tool acts as an external client that exercises that running node through its HTTP API.

## Daemon responsibility

The `pgtuskmaster` binary loads configuration and delegates to `runtime::run_node_from_config_path`. This spawns a long-lived Tokio runtime that hosts the node's control loop. All state, decisions, and internal steering live inside this process.

## Client responsibility

The `pgtuskmasterctl` binary parses its command-line interface using clap and delegates to `cli::run`. It prints output on success and maps errors to exit codes. It does not embed or reach into the node runtime; it constructs an HTTP client and calls the API.

The `CliApiClient` builds an HTTP client against a base URL, normalizes read and admin tokens, and exposes three operations:
- `get_ha_state` for observation
- `delete_switchover` for cancellation
- `post_switchover` for initiation

Read operations use a read token when configured and fall back to the admin token if no read token exists. Admin operations always require the admin token.

## Why HTTP is the boundary

All control flows through supported HTTP paths. This is not incidental; it is enforced. The policy test explicitly allows observation and switchover requests through these documented paths and forbids any post-start internal steering.

This design removes the temptation to treat the CLI as a local control backdoor. It cannot bypass the API, cannot directly manipulate runtime state, and cannot circumvent token checks. Every operation an operator or automation tool can perform must succeed or fail on the same terms that any external client would face.

## Implications for operations

Because the CLI is just another HTTP client, automation that calls `pgtuskmasterctl` can be replaced with direct HTTP calls without loss of capability. Operational discipline benefits from a single surface for permissions, auditing, and rate limiting. The boundary is sharp: runtime internals stay inside the node; control actions travel over the network stack, even on localhost.
