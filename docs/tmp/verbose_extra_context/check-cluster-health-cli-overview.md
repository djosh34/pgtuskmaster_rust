# Verbose CLI overview for cluster health checks

This note exists to give K2 exhaustive source-backed context from the exact files it requested.

## CLI surface from `src/cli/args.rs`

- The binary name is `pgtuskmasterctl`.
- Global flags:
  - `--base-url <BASE_URL>` defaults to `http://127.0.0.1:8080`.
  - `--read-token <READ_TOKEN>` reads from `PGTUSKMASTER_READ_TOKEN` when present.
  - `--admin-token <ADMIN_TOKEN>` reads from `PGTUSKMASTER_ADMIN_TOKEN` when present.
  - `--timeout-ms <TIMEOUT_MS>` defaults to `5000`.
  - `--output <OUTPUT>` defaults to `json` and accepts only `json` or `text`.
- Top-level subcommands:
  - `ha`
- `ha` subcommands:
  - `state`
  - `switchover`
- `ha switchover` subcommands:
  - `clear`
  - `request --requested-by <REQUESTED_BY>`

## Health-checking implications

- There is exactly one read-oriented HA command in the CLI surface exposed by `src/cli/args.rs`: `ha state`.
- `switchover clear` and `switchover request` are write operations. They should not be described as health-check commands even though they are HA-related.
- Any how-to for cluster health should stay precise about that distinction:
  - inspection is via `ha state`
  - state-changing operations are via `ha switchover ...`
- The CLI can emit either JSON or text for `ha state`, because output formatting is global and not command-specific.

## Output format details from `src/cli/output.rs`

- JSON output is a pretty-printed serialization of the API payload.
- For `ha state`, JSON output serializes `HaStateResponse` directly.
- For accepted write calls, JSON output serializes `{ "accepted": true|false }`.
- Text output for `ha state` is a newline-delimited list of key-value pairs in this exact order:
  - `cluster_name=...`
  - `scope=...`
  - `self_member_id=...`
  - `leader=...` where missing leaders render as `<none>`
  - `switchover_requested_by=...` where missing requests render as `<none>`
  - `member_count=...`
  - `dcs_trust=...`
  - `ha_phase=...`
  - `ha_tick=...`
  - `ha_decision=...`
  - `snapshot_sequence=...`

## `ha_decision` text encoding details

- `no_change`
- `wait_for_postgres(start_requested=..., leader_member_id=...)`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader(leader_member_id=...)`
- `become_primary(promote=...)`
- `step_down(reason=..., release_leader_lease=..., clear_switchover=..., fence=...)`
- `recover_replica(strategy=...)`
- `fence_node`
- `release_leader_lease(reason=...)`
- `enter_fail_safe(release_leader_lease=...)`

## Runtime evidence collected for the CLI

### `cargo run --bin pgtuskmasterctl -- --help`

```text
HA admin CLI for PGTuskMaster API

Usage: pgtuskmasterctl [OPTIONS] <COMMAND>

Commands:
  ha
  help  Print this message or the help of the given subcommand(s)

Options:
      --base-url <BASE_URL>        [default: http://127.0.0.1:8080]
      --read-token <READ_TOKEN>    [env: PGTUSKMASTER_READ_TOKEN=]
      --admin-token <ADMIN_TOKEN>  [env: PGTUSKMASTER_ADMIN_TOKEN=]
      --timeout-ms <TIMEOUT_MS>    [default: 5000]
      --output <OUTPUT>            [default: json] [possible values: json, text]
  -h, --help                       Print help
```

### `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 ha --help`

```text
Usage: pgtuskmasterctl ha <COMMAND>

Commands:
  state
  switchover
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
