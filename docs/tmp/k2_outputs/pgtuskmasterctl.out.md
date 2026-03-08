# pgtuskmasterctl

CLI surface, HTTP mapping, token selection, output rendering, and exit behavior.

## Binary entrypoint

| Item | Value |
|---|---|
| Source | `src/bin/pgtuskmasterctl.rs` |
| Runtime | `#[tokio::main(flavor = "current_thread")]` |
| Entry point | `main()` parses `Cli::parse()` and awaits `pgtuskmaster_rust::cli::run(cli)` |
| Success output | `println!` of the returned string |
| Error output | `eprintln!` of the formatted error |
| Exit code | `0` on success, otherwise `err.exit_code()` |

## Global options

| Option | Type | Default | Environment | Description |
|---|---|---|---|---|
| `--base-url` | `String` | `http://127.0.0.1:8080` | none | HTTP base URL for API requests |
| `--read-token` | `Option<String>` | none | `PGTUSKMASTER_READ_TOKEN` | Token for read operations |
| `--admin-token` | `Option<String>` | none | `PGTUSKMASTER_ADMIN_TOKEN` | Token for admin operations |
| `--timeout-ms` | `u64` | `5000` | none | Request timeout in milliseconds |
| `--output` | `json` or `text` | `json` | none | Output format |

Token strings are trimmed. Blank token strings become `None`.

## Command tree

```
Cli
└─ Command::Ha(HaArgs)
   └─ HaCommand
      ├─ State
      └─ Switchover(SwitchoverArgs)
         └─ SwitchoverCommand
            ├─ Clear
            └─ Request(RequestSwitchoverArgs)
               └─ requested_by: String (required)
```

`--requested-by` is required for `ha switchover request`.

## Client construction

`cli::run` constructs `CliApiClient` and dispatches commands.

| Parameter | Processing |
|---|---|
| Base URL | Trimmed before URL parsing |
| Timeout | `Duration::from_millis(timeout_ms)` |
| HTTP pool | `pool_max_idle_per_host(0)` |
| Tokens | Trimmed; blanks become `None` |
| Construction failure | `CliError::RequestBuild` |

## HTTP mapping

| Command | Method | Path | Token role | Expected status | Body |
|---|---|---|---|---|---|
| `ha state` | `GET` | `/ha/state` | read | `200 OK` | none |
| `ha switchover clear` | `DELETE` | `/ha/switchover` | admin | `202 Accepted` | none |
| `ha switchover request` | `POST` | `/switchover` | admin | `202 Accepted` | `{"requested_by":"..."}` |

Read requests use `read_token` when present, otherwise fall back to `admin_token`. Admin requests use `admin_token` only.

Unexpected status returns `CliError::ApiStatus { status, body }`. Body read failure becomes `<failed to read response body: ...>`.

## Output rendering

`--output json` serializes `CommandOutput` as an untagged enum containing `HaStateResponse` or `AcceptedResponse` using `serde_json::to_string_pretty`.

`--output text` renders:

| Payload | Format |
|---|---|
| `AcceptedResponse` | `accepted=<bool>` |
| `HaStateResponse` | newline-separated `key=value` lines for `cluster_name`, `scope`, `self_member_id`, `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, `ha_decision`, `snapshot_sequence` |

Missing `leader` and `switchover_requested_by` render as `<none>`.

`ha_decision` text forms:

```
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

## Exit behavior

| Condition | Exit code |
|---|---|
| Success | `0` |
| Clap usage failure | `2` |
| `CliError::RequestBuild` | `3` |
| `CliError::Transport` | `3` |
| `CliError::ApiStatus` | `4` |
| `CliError::Decode` | `5` |
| `CliError::Output` | `5` |
