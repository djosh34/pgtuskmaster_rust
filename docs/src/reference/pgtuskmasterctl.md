# pgtuskmasterctl Reference

CLI surface, HTTP mapping, output rendering, token selection, and exit behavior for the PGTuskMaster admin tool.

## Binary

| Item | Value |
|---|---|
| Source | `src/bin/pgtuskmasterctl.rs` |
| Runtime | `#[tokio::main(flavor = "current_thread")]` |
| Entry point | `main()` parses `Cli::parse()` and awaits `pgtuskmaster_rust::cli::run(cli)` |
| Success output | `println!` of the returned string |
| Error output | `eprintln!` of the formatted error |
| Exit code | `0` on success, otherwise `err.exit_code()` |

## Global Options

| Option | Type | Default | Environment | Description |
|---|---|---|---|---|
| `--base-url` | `String` | `http://127.0.0.1:8080` | none | HTTP base URL for API requests |
| `--read-token` | `Option<String>` | none | `PGTUSKMASTER_READ_TOKEN` | Token for read operations |
| `--admin-token` | `Option<String>` | none | `PGTUSKMASTER_ADMIN_TOKEN` | Token for admin operations |
| `--timeout-ms` | `u64` | `5000` | none | Request timeout in milliseconds |
| `--output` | `json` or `text` | `json` | none | Output format |

Configured token strings are trimmed. Blank token strings become `None`.

## Command Tree

```text
ha state
ha switchover clear
ha switchover request --requested-by <STRING>
```

The `--requested-by` argument is required for `ha switchover request`.

## Client Construction

`cli::run` calls `CliApiClient::new(base_url, timeout_ms, read_token, admin_token)`, dispatches the selected command, then passes the result to `output::render_output(...)`.

| Step | Behavior |
|---|---|
| Base URL | Trimmed before URL parsing |
| Timeout | `Duration::from_millis(timeout_ms)` |
| HTTP pool | `pool_max_idle_per_host(0)` |
| Token normalization | Trimmed; blanks become `None` |
| URL parse or client build failure | `CliError::RequestBuild(...)` |

## HTTP Mapping

| Command | Method | Path | Token role | Expected status | Request body |
|---|---|---|---|---|---|
| `ha state` | `GET` | `/ha/state` | read | `200 OK` | none |
| `ha switchover clear` | `DELETE` | `/ha/switchover` | admin | `202 Accepted` | none |
| `ha switchover request` | `POST` | `/switchover` | admin | `202 Accepted` | `{"requested_by":"..."}` |

Read requests use `read_token` when present, otherwise fall back to `admin_token`. Admin requests use `admin_token` only.

Unexpected HTTP status returns `CliError::ApiStatus { status, body }`. If reading the error response body fails, `body` becomes `<failed to read response body: ...>`.

## Output Rendering

`--output json` uses `serde_json::to_string_pretty` over an untagged enum containing `AcceptedResponse` or `HaStateResponse`.

`--output text` renders:

| Payload | Format |
|---|---|
| `AcceptedResponse` | `accepted=<bool>` |
| `HaStateResponse` | newline-separated `key=value` lines for `cluster_name`, `scope`, `self_member_id`, `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, `ha_decision`, and `snapshot_sequence` |

Missing `leader` and `switchover_requested_by` render as `<none>`.

`ha_decision` text forms:

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

## Exit Behavior

| Condition | Exit code |
|---|---|
| Success | `0` |
| Clap usage failure before `cli::run` | `2` |
| `CliError::RequestBuild` | `3` |
| `CliError::Transport` | `3` |
| `CliError::ApiStatus` | `4` |
| `CliError::Decode` | `5` |
| `CliError::Output` | `5` |

The command surface and exit mapping are exercised by `tests/cli_binary.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/cli/output.rs`, and `src/cli/mod.rs`.
