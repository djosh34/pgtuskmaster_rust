# pgtuskmasterctl Reference

CLI surface, HTTP mapping, output rendering, token selection, and exit behavior for the PGTuskMaster admin tool.

## Binary

| Item | Value |
|---|---|
| Source | `src/bin/pgtuskmasterctl.rs` |
| Runtime | `#[tokio::main(flavor = "current_thread")]` |
| Entry point | `main()` parses `Cli::parse()` and calls `pgtuskmaster_rust::cli::run(cli).await` |
| Success output | `println!` of the returned string |
| Error output | `eprintln!` of the formatted error |
| Exit code | `0` on success, `err.exit_code()` on failure |

## Global Options

| Option | Environment | Default | Values |
|---|---|---|---|
| `--base-url` | none | `http://127.0.0.1:8080` | `String` |
| `--read-token` | `PGTUSKMASTER_READ_TOKEN` | none | `Option<String>` |
| `--admin-token` | `PGTUSKMASTER_ADMIN_TOKEN` | none | `Option<String>` |
| `--timeout-ms` | none | `5000` | `u64` |
| `--output` | none | `json` | `json`, `text` |

Tokens are trimmed; blank strings become `None`.

## Commands

| Command | Arguments | Client method | HTTP mapping |
|---|---|---|---|
| `ha state` | none | `get_ha_state()` | `GET /ha/state` |
| `ha switchover clear` | none | `delete_switchover()` | `DELETE /ha/switchover` |
| `ha switchover request` | `--requested-by <STRING>` | `post_switchover(requested_by)` | `POST /switchover` |

## Client Construction

`cli::run` calls `CliApiClient::new(base_url, timeout_ms, read_token, admin_token)`, dispatches the selected command, then passes the result to `output::render_output(...)`.

| Step | Behavior |
|---|---|
| Base URL | Trimmed before URL parsing |
| Timeout | `Duration::from_millis(timeout_ms)` |
| HTTP pool | `pool_max_idle_per_host(0)` |
| Token normalization | Trimmed; blanks become `None` |
| URL parse or client build failure | `CliError::RequestBuild(...)` |

## HTTP Details

| Command | Method | Endpoint | Role | Expected status | Request body |
|---|---|---|---|---|---|
| `ha state` | `GET` | `/ha/state` | read | `200 OK` | none |
| `ha switchover clear` | `DELETE` | `/ha/switchover` | admin | `202 Accepted` | none |
| `ha switchover request` | `POST` | `/switchover` | admin | `202 Accepted` | `{"requested_by":"..."}` |

### Token Selection

| Request role | Token used |
|---|---|
| read | `read_token` if present, else `admin_token` |
| admin | `admin_token` only |

## Output Formats

### JSON

`--output json` uses `serde_json::to_string_pretty` over an untagged enum containing `AcceptedResponse` or `HaStateResponse`.

### Text

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

## Errors and Exit Codes

| Failure | `CliError` variant | Exit code |
|---|---|---|
| URL parse or join failure | `RequestBuild(...)` | `3` |
| HTTP client build failure | `RequestBuild(...)` | `3` |
| HTTP send failure | `Transport(...)` | `3` |
| Unexpected HTTP status | `ApiStatus { status, body }` | `4` |
| Response body read failure | `body = "<failed to read response body: ...>"` | `4` |
| JSON decode failure | `Decode(...)` | `5` |
| JSON output encode failure | `Output(...)` | `5` |
| Clap usage failure before `cli::run` | n/a | `2` |

The `body` field in `ApiStatus` falls back to `<failed to read response body: ...>` if reading the response body fails.

## Verified Behaviors

| Test file | What it verifies |
|---|---|
| `tests/cli_binary.rs` | `--help` includes `ha`; invalid subcommand usage exits `2`; `ha state` against a refused connection exits `3` with stderr containing `transport error` |
| `src/cli/args.rs` | default parsing of `ha state`; environment-variable token loading; `ha switchover request` requires `--requested-by` |
| `src/cli/client.rs` | read-token selection with admin fallback; `DELETE /ha/switchover`; API status mapping; malformed JSON decode mapping; refused-connection transport mapping |
| `src/cli/output.rs` | text rendering for HA state lines; JSON rendering for accepted payloads |
| `src/cli/mod.rs` | stable exit-code mapping |
