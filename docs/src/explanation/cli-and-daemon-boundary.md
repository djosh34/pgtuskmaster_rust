# Why the CLI stays outside the control loop

`pgtuskmaster` and `pgtuskmasterctl` are deliberately separate binaries with different responsibilities. The daemon runs the node and its control loop. The CLI is an external client of the daemon's supported HTTP surface.

The [pgtuskmaster reference](../reference/pgtuskmaster.md), [pgtuskmasterctl reference](../reference/pgtuskmasterctl.md), and [HTTP API reference](../reference/http-api.md) describe the pieces. This page explains why the boundary is kept sharp.

## The daemon owns runtime control

The `pgtuskmaster` binary accepts `--config` and delegates into `runtime::run_node_from_config_path` inside a Tokio runtime. That long-lived process owns state publication, HA decisions, and worker coordination.

Those responsibilities stay inside the daemon because they are part of the runtime's internal control loop, not part of the CLI contract.

## The CLI is a supported external client

`pgtuskmasterctl` parses its CLI, delegates to `cli::run`, prints output on success, and maps failures to exit codes. Its `CliApiClient` constructs an HTTP client against a base URL, normalizes read and admin tokens, and exposes a small set of supported calls: read HA state, post switchover, and clear switchover.

Read requests use a read token when configured and fall back to the admin token when no read token exists. Admin operations use the admin token.

## Why the boundary matters

The post-start policy explicitly allows observation and supported switchover requests while forbidding internal steering after startup. That policy would be much weaker if the CLI were allowed to reach behind the daemon boundary through local hooks or internal shortcuts.

Keeping the CLI outside the control loop means every supported operation goes through the same API contract and permission model that any other client would face. The CLI is convenient, but it is not a privileged bypass.

## The tradeoff

This design narrows the CLI on purpose. Operators cannot use it to hot-patch runtime internals or steer HA through undocumented backdoors. The benefit is operational discipline: one supported control surface, one token model, and a clearer line between daemon internals and client behavior.
