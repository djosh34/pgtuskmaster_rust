# Verbose CLI surface summary

This file exists to answer the "extra info" requests from the `choose-doc` outputs with source-derived facts only.

## Binary and entry point

- The operator CLI binary is `pgtuskmasterctl`.
- The binary entry point lives in `src/bin/pgtuskmasterctl.rs`.
- `main` parses `Cli` with clap, runs `pgtuskmaster_rust::cli::run(cli).await`, prints successful output to stdout, prints errors to stderr, and exits using the mapped CLI error exit code.

## Top-level CLI shape

The clap shape in `src/cli/args.rs` is:

- global flag: `--base-url <STRING>`
  - default: `http://127.0.0.1:8080`
- global flag: `--read-token <STRING>`
  - env fallback: `PGTUSKMASTER_READ_TOKEN`
- global flag: `--admin-token <STRING>`
  - env fallback: `PGTUSKMASTER_ADMIN_TOKEN`
- global flag: `--timeout-ms <U64>`
  - default: `5000`
- global flag: `--output <json|text>`
  - default: `json`
- top-level command group: `ha`

There are no other top-level command groups in `src/cli/args.rs` besides `ha`.

## HA subcommands

The `ha` command group contains:

- `ha state`
  - no subcommand-specific flags or positional arguments in `src/cli/args.rs`
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::get_ha_state()`
- `ha switchover clear`
  - no extra flags
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::delete_switchover()`
- `ha switchover request --requested-by <STRING>`
  - `--requested-by` is required
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::post_switchover(requested_by)`

There is no `leader set` subcommand in the current clap tree. The integration test `tests/cli_binary.rs` intentionally invokes `ha leader set` as an invalid command and expects clap to exit with code `2`.

## Authentication behavior

- `--read-token` and `--admin-token` are both real CLI flags.
- Read operations use `read_token` first and fall back to `admin_token` if `read_token` is absent.
- Admin operations require `admin_token`.
- Empty or whitespace-only token values are normalized to `None` by `normalize_token` in `src/cli/client.rs`.

Role-to-command behavior from `src/cli/client.rs`:

- `get_ha_state` performs `GET /ha/state` with read-role auth
- `delete_switchover` performs `DELETE /ha/switchover` with admin-role auth
- `post_switchover` performs `POST /switchover` with admin-role auth and JSON body `{ "requested_by": "..." }`

Important factual note:

- The POST endpoint path used by the CLI client is `/switchover`, not `/ha/switchover`.
- The API controller code writes switchover state under a DCS path `/{scope}/switchover`.
- The delete path exposed by the CLI client is `/ha/switchover`.
- Any doc draft that claims both POST and DELETE use the same `/ha/switchover` API path should be checked carefully against source.

## Output formats

The only output formats are `json` and `text`.

For accepted/acknowledgement responses:

- JSON renders the serialized `AcceptedResponse`
- Text renders exactly `accepted=<bool>`

For `ha state` responses, JSON renders the `HaStateResponse` payload from `src/api/mod.rs` with these top-level fields:

- `cluster_name`
- `scope`
- `self_member_id`
- `leader`
- `switchover_requested_by`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_tick`
- `ha_decision`
- `snapshot_sequence`

Text mode renders these lines:

- `cluster_name=...`
- `scope=...`
- `self_member_id=...`
- `leader=...` or `<none>`
- `switchover_requested_by=...` or `<none>`
- `member_count=...`
- `dcs_trust=...`
- `ha_phase=...`
- `ha_tick=...`
- `ha_decision=...`
- `snapshot_sequence=...`

## HA state enums and payload shape

`dcs_trust` string values:

- `full_quorum`
- `fail_safe`
- `not_trusted`

`ha_phase` string values:

- `init`
- `waiting_postgres_reachable`
- `waiting_dcs_trusted`
- `waiting_switchover_successor`
- `replica`
- `candidate_leader`
- `primary`
- `rewinding`
- `bootstrapping`
- `fencing`
- `fail_safe`

`ha_decision` is tagged JSON with `kind` in snake_case. Variants in `src/api/mod.rs` are:

- `no_change`
- `wait_for_postgres`
  - fields: `start_requested`, `leader_member_id`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader`
  - field: `leader_member_id`
- `become_primary`
  - field: `promote`
- `step_down`
  - fields: `reason`, `release_leader_lease`, `clear_switchover`, `fence`
- `recover_replica`
  - field: `strategy`
- `fence_node`
- `release_leader_lease`
  - field: `reason`
- `enter_fail_safe`
  - field: `release_leader_lease`

## Exit codes and observable behavior

From `src/cli/mod.rs` tests and the CLI error mapping:

- transport failures map to exit code `3`
- unexpected API status failures map to exit code `4`
- decode failures map to exit code `5`

From `tests/cli_binary.rs`:

- `pgtuskmasterctl --help` should succeed
- invalid clap usage should exit with code `2`
- unreachable `ha state` should exit with code `3` and mention `transport error`

