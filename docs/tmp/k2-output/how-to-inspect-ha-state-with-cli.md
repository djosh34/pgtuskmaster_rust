# How to inspect HA state with pgtuskmasterctl

This guide shows how to query the current High Availability snapshot from the command line and verify that the request succeeded.

## Prerequisites

- A running pgtuskmaster API endpoint reachable over HTTP.
- A read token or admin token for the endpoint (optional if authentication is not configured).

## Query the default endpoint

Run the `ha state` subcommand with no options to query `http://127.0.0.1:8080`:

```sh
pgtuskmasterctl ha state
```

The command returns JSON output and exits with status `0` on success (HTTP 200). On failure, it returns a non-zero exit code and prints a diagnostic message.

## Read a text-formatted snapshot

Use `--output text` to render the HA state as key-value lines:

```sh
pgtuskmasterctl ha state --output text
```

Example output (values may differ on your system):

```
cluster_name=mycluster
scope=prod
self_member_id=pg1
leader=<none>
switchover_requested_by=<none>
member_count=3
dcs_trust=true
ha_phase=initializing
ha_tick=42
ha_decision=no-action
snapshot_sequence=7
```

Fields that have no value appear as `<none>`.

## Troubleshooting

- **Exit code 3**: The request could not be sent or the connection failed. Verify network reachability to the base URL.

- **Exit code 4**: The API responded with a non-200 status. Check your tokens (`--read-token` or `--admin-token`) and ensure the server is healthy.

- **Exit code 5**: The response could not be decoded or displayed. Try again; persistent failures may indicate a server-side bug.

## Related pages

- [pgtuskmasterctl reference](../reference/pgtuskmasterctl.md)
- [HTTP API reference](../reference/http-api.md)
- [HA state machine reference](../reference/ha-state-machine.md)
- [HA decisions and actions](../explanation/ha-decisions-and-actions.md)
