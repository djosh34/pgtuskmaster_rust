# Extra Context: Exact pgtuskmasterctl Command To Check The Current Leader

K2 asked for the exact `pgtuskmasterctl` command to check which node is the current leader. This note answers that with source-backed detail from the CLI parser, client, and output renderer.

## There is no dedicated leader subcommand

The current CLI parser in `src/cli/args.rs` exposes:

- `pgtuskmasterctl ha state`
- `pgtuskmasterctl ha switchover clear`
- `pgtuskmasterctl ha switchover request --requested-by <member>`

There is no `ha leader get`, `ha leader show`, or `ha leader set` command in the current CLI shape.

This matters because `tests/cli_binary.rs` intentionally invokes `pgtuskmasterctl ha leader set` and asserts that it exits with clap usage code `2`. That test is strong evidence that older leader-specific command wording should not be used in fresh docs.

## How leader information is retrieved

`src/cli/mod.rs` routes `ha state` to `CliApiClient::get_ha_state()`.

`src/cli/client.rs` shows that `get_ha_state()` performs:

- HTTP method: `GET`
- path: `/ha/state`

So the CLI does not ask a separate leader endpoint. It reads the full HA state payload and extracts the `leader` field from that response.

`src/api/mod.rs` defines the returned shape. `HaStateResponse` includes:

- `self_member_id`
- `leader`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- `snapshot_sequence`

The current leader is therefore whatever value appears in the `leader` field.

## Exact command to run against node-a in the example cluster

Using the fixed API port from `.env.docker.example`, the most explicit text-output command is:

```bash
pgtuskmasterctl --base-url http://127.0.0.1:18081 --output text ha state
```

That is the best exact command for the tutorial because:

- it names the binary exactly as defined by the clap parser
- it targets node-a's published API port from the checked-in example env file
- it asks for text output, which makes the `leader=` line easy to read in a tutorial

The rendered text output includes a line in the form:

```text
leader=node-a
```

or:

```text
leader=node-b
```

or:

```text
leader=<none>
```

That format is backed by `src/cli/output.rs`, which prints the `leader` field as a standalone `leader=...` line in text mode.

## JSON alternative

If the tutorial wants machine-readable output instead, the equivalent command is:

```bash
pgtuskmasterctl --base-url http://127.0.0.1:18081 ha state
```

JSON is the default output mode, so omitting `--output` still works. In that case, the current leader is the value of the JSON `leader` field.

## Authentication note for this docker cluster

All three cluster runtime configs set:

- API TLS mode: disabled
- API auth type: disabled

That means the example docker cluster does not require `--read-token` or `--admin-token` for the `ha state` read command.

## Evidence sources behind this note

- `src/cli/args.rs`
- `src/cli/mod.rs`
- `src/cli/client.rs`
- `src/cli/output.rs`
- `src/api/mod.rs`
- `.env.docker.example`
- `tests/cli_binary.rs`
- `docker/configs/cluster/node-a/runtime.toml`
- `docker/configs/cluster/node-b/runtime.toml`
- `docker/configs/cluster/node-c/runtime.toml`
