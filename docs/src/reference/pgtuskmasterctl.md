# pgtuskmasterctl Reference

CLI surface, HTTP mapping, output rendering, token selection, and exit behavior for the PGTuskMaster admin tool.

## Binary

| Item | Value |
|---|---|
| Source location | `src/bin/pgtuskmasterctl.rs` |
| Runtime | `#[tokio::main(flavor = "current_thread")]` |
| Entry point | `main()` parses `Cli::parse()` and calls `pgtuskmaster_rust::cli::run(cli).await` |
| Stdout success | `println!` of the returned string |
| Stderr errors | `eprintln!` of the formatted error |
| Exit code | `0` on success, `err.exit_code()` on failure |

## Global Options

| Option | Environment | Default | Type |
|---|---|---|---|
| `--base-url` | none | `http://127.0.0.1:8080` | `String` |
| `--read-token` | `PGTUSKMASTER_READ_TOKEN` | none | `Option<String>` |
| `--admin-token` | `PGTUSKMASTER_ADMIN_TOKEN` | none | `Option<String>` |
| `--timeout-ms` | none | `5000` | `u64` |
| `--output` | none | `json` | `json`, `text` |

Tokens are trimmed. Blank token strings become `None`.

## Command Tree

| Command | Arguments | Client method |
|---|---|---|
| `ha state` | none | `client.get_ha_state()` |
| `ha switchover clear` | none | `client.delete_switchover()` |
| `ha switchover request` | `--requested-by <STRING>` | `client.post_switchover(requested_by)` |

## Client Construction

`cli::run` builds `CliApiClient::new(cli.base_url, cli.timeout_ms, cli.read_token, cli.admin_token)` and passes the result to `output::render_output(...)`.

| Step | Behavior |
|---|---|
| Base URL | `base_url.trim()` before URL parsing |
| URL parse failure | `CliError::RequestBuild("invalid --base-url value: ...")` |
| Tokens | Trimmed via `normalize_token`; blanks become `None` |
| Timeout | `Duration::from_millis(timeout_ms)` |
| HTTP pool | `pool_max_idle_per_host(0)` |

## HTTP Request Mapping

| Command | Method | Endpoint | Role | Expected status | Body |
|---|---|---|---|---|---|
| `ha state` | `GET` | `/ha/state` | read | `200 OK` | none |
| `ha switchover clear` | `DELETE` | `/ha/switchover` | admin | `202 Accepted` | none |
| `ha switchover request` | `POST` | `/switchover` | admin | `202 Accepted` | `{"requested_by":"..."}` |

## Token Selection

| Request role | Token used |
|---|---|
| read | `read_token` if present, else `admin_token` |
| admin | `admin_token` only |

## Output Formats

### JSON

`--output json` uses `serde_json::to_string_pretty(...)` over an untagged enum containing either `AcceptedResponse` or `HaStateResponse`.

### Text

| Payload | Rendering |
|---|---|
| `AcceptedResponse` | `accepted=<bool>` |
| `HaStateResponse` | newline-separated `key=value` lines for `cluster_name`, `scope`, `self_member_id`, `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, `ha_decision`, and `snapshot_sequence` |

Missing `leader` and `switchover_requested_by` render as `<none>`.

`ha_decision` text encodings:

```text
no_change
wait_for_postgres(start_requested=..., leader_member_id=...)
wait_for_dcs_trust
attempt_leadership
follow_leader(leader_member_id=...)
become_primary(promote=...)
step_down(reason=..., release_leader_lease=..., clear_switchover=..., fence=...)
recover_replica(strategy=...)
fence_node
release_leader_lease(reason=...)
enter_fail_safe(release_leader_lease=...)
```

## Error Mapping

| Failure point | `CliError` variant |
|---|---|
| URL parse or join failure | `CliError::RequestBuild(...)` |
| HTTP client build failure | `CliError::RequestBuild(...)` |
| HTTP send failure | `CliError::Transport(...)` |
| Unexpected HTTP status | `CliError::ApiStatus { status, body }` |
| Response-body read failure while building `ApiStatus` | `body = "<failed to read response body: ...>"` |
| JSON decode failure | `CliError::Decode(...)` |
| JSON output encode failure | `CliError::Output(...)` |

## Exit Codes

| Code | Source |
|---|---|
| `2` | clap usage failure before `cli::run` |
| `3` | `CliError::Transport`, `CliError::RequestBuild` |
| `4` | `CliError::ApiStatus` |
| `5` | `CliError::Decode`, `CliError::Output` |

## Verified Behaviors

| Test file | What it verifies |
|---|---|
| `tests/cli_binary.rs` | `--help` includes `ha`; invalid subcommand usage exits `2`; `ha state` against a refused connection exits `3` with stderr containing `transport error` |
| `src/cli/args.rs` | default parsing of `ha state`; environment-variable token loading; `ha switchover request` requires `--requested-by` |
| `src/cli/client.rs` | read-token selection with admin fallback; `DELETE /ha/switchover` path; API status mapping; malformed JSON decode mapping; refused-connection transport mapping |
| `src/cli/output.rs` | text rendering for HA state lines; JSON rendering for accepted payloads |
| `src/cli/mod.rs` | stable exit-code mapping |
