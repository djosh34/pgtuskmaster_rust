# HTTP API Reference

The API worker provides HTTP control and observation routes. Implementation spans `src/api/worker.rs`, `src/api/controller.rs`, and `src/api/fallback.rs`.

## Worker Loop And Transport

`ApiWorkerCtx` fields: `listener`, `poll_interval`, `scope`, `member_id`, `config_subscriber`, `dcs_store`, `debug_snapshot_subscriber`, `tls_mode_override`, `tls_acceptor`, `role_tokens`, `require_client_cert`, `log`.

`ApiWorkerCtx::new` copies `scope` from `cfg.dcs.scope`, copies `member_id` from `cfg.cluster.member_id`, sets `poll_interval` to `10 ms`, initializes `debug_snapshot_subscriber`, `tls_mode_override`, `tls_acceptor`, and `role_tokens` to `None`, and sets `require_client_cert` to `false`.

`api::worker::run` loops forever. It calls `step_once`, logs `api.step_once_failed` on error, returns the error only when fatal, then sleeps for the poll interval.

Fatal step errors contain one of the following messages:

| Message |
|---|
| `api accept failed` |
| `tls mode requires a configured tls acceptor` |
| `api local_addr failed` |

### Connection And Request Constants

| Constant | Value |
|---|---|
| `API_LOOP_POLL_INTERVAL` | `10 ms` |
| `API_ACCEPT_TIMEOUT` | `1 ms` |
| `API_REQUEST_READ_TIMEOUT` | `100 ms` |
| `API_TLS_CLIENT_HELLO_PEEK_TIMEOUT` | `10 ms` |
| `API_REQUEST_ID_MAX_LEN` | `128` |
| `HTTP_REQUEST_MAX_BYTES` | `1048576` |
| `HTTP_REQUEST_HEADER_LIMIT_BYTES` | `16384` |
| `HTTP_REQUEST_SCRATCH_BUFFER_BYTES` | `4096` |
| `HTTP_REQUEST_HEADER_CAPACITY` | `64` |

`step_once` accepts at most one connection attempt per loop iteration. An accept timeout returns `Ok(())`.

After accept, the worker logs `api.connection_accepted` with the peer address and effective TLS mode, negotiates plain or TLS transport, reads at most one HTTP request, authorizes it, routes it, writes one HTTP response, then logs `api.response_sent`.

Request-read timeout returns `Ok(())` without writing an HTTP response.

Parse failures before routing return `400 Bad Request` with the parser message body and log `api.request_parse_failed`.

## Security Configuration

### Authentication

Authentication configuration comes from `cfg.api.security.auth` unless overridden with `ApiWorkerCtx.configure_role_tokens(read_token, admin_token)`.

Auth config variants:

- `ApiAuthConfig::Disabled`
- `ApiAuthConfig::RoleTokens(ApiRoleTokensConfig { read_token, admin_token })`

Runtime token normalization trims configured runtime-config token strings and converts blank strings to `None`. `configure_role_tokens` uses `normalize_optional_token`, which rejects blank override strings with `WorkerError::Message("role token must not be empty when configured")`. If both resolved tokens are absent, requests are allowed without authorization.

Authorization-header lookup is case-insensitive. Bearer extraction trims the full header value, requires the exact prefix `Bearer `, trims the remainder, and rejects an empty remainder.

Route roles:

| Role | Routes |
|---|---|
| admin | `POST /switchover`, `POST /fallback/heartbeat`, `DELETE /ha/switchover` |
| read | all other supported routes |

Authorization outcomes:

| Condition | Result |
|---|---|
| both resolved tokens absent | request allowed without authorization |
| token protection configured and no bearer token extracted | `401 Unauthorized` with body `unauthorized` |
| bearer token matches `admin_token` | request allowed |
| bearer token matches `read_token` and request is a read route | request allowed |
| bearer token matches `read_token` and request is an admin route | `403 Forbidden` with body `forbidden` |
| any other token mismatch | `401 Unauthorized` with body `unauthorized` |

Auth decision logs include the following fields:

| Field | Description |
|---|---|
| `api.method` | HTTP method |
| `api.route_template` | matched route template |
| `api.auth.header_present` | boolean |
| `api.auth.result` | authorization result |
| `api.auth.required_role` | required role for route |
| `api.request_id` | optional truncated request ID |

`x-request-id` values are trimmed; empty values are ignored and non-empty values are truncated to `128` characters.

### TLS

TLS configuration comes from `cfg.api.security.tls` unless overridden with `ApiWorkerCtx.configure_tls(mode, server_config)`.

| Field | Type |
|---|---|
| `mode` | `ApiTlsMode` |
| `identity` | `Option<TlsServerIdentityConfig>` |
| `client_auth` | `Option<TlsClientAuthConfig>` |

`ApiTlsMode` values: `disabled`, `optional`, `required`.

`TlsServerIdentityConfig` fields:

| Field | Type |
|---|---|
| `cert_chain` | `InlineOrPath` |
| `private_key` | `InlineOrPath` |

`TlsClientAuthConfig` fields:

| Field | Type |
|---|---|
| `client_ca` | `InlineOrPath` |
| `require_client_cert` | `bool` |

`configure_tls` clears the acceptor in disabled mode and requires a server config for optional or required mode. Calling it without a server config in optional or required mode returns `WorkerError::Message("tls mode optional/required requires a server tls config")`.

Effective TLS mode uses the override when present, otherwise `cfg.api.security.tls.mode`.

Connection handling by effective TLS mode:

| Mode | Accept behavior |
|---|---|
| disabled | connection stays plain TCP |
| required | always start a TLS handshake |
| optional | peek one byte for up to `10 ms`; if the byte is not `0x16`, or the peek times out, or returns `WouldBlock`, stay plain TCP; otherwise start a TLS handshake |

Missing TLS acceptor in optional or required mode returns `WorkerError::Message("tls mode requires a configured tls acceptor")`.

TLS handshake failures log `api.tls_handshake_failed` and the connection is dropped without an HTTP response.

If `require_client_cert` is true and the accepted TLS stream has no peer certificate, the worker logs `api.tls_client_cert_missing` and drops the connection without an HTTP response.

## Route Table

| Method | Path | Role | Success status | Success payload | Notes |
|---|---|---|---|---|---|
| `POST` | `/switchover` | admin | `202 Accepted` | `AcceptedResponse` | writes serialized `SwitchoverRequest` JSON to `/{scope}/switchover` after trimming leading and trailing `/` from scope |
| `DELETE` | `/ha/switchover` | admin | `202 Accepted` | `AcceptedResponse` | clears switchover through `DcsHaWriter::clear_switchover` |
| `GET` | `/ha/state` | read | `200 OK` | `HaStateResponse` | returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/fallback/cluster` | read | `200 OK` | `FallbackClusterView` | `name` is copied from `cfg.cluster.name` |
| `POST` | `/fallback/heartbeat` | admin | `202 Accepted` | `AcceptedResponse` | request body is `FallbackHeartbeatInput` |
| `GET` | `/debug/snapshot` | read | `200 OK` | text from `format!("{:#?}", snapshot)` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/debug/verbose` | read | `200 OK` | JSON from `build_verbose_payload` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured; invalid `since` returns `400 Bad Request` with body `invalid since query parameter: <parse error>` |
| `GET` | `/debug/ui` | read | `200 OK` | HTML from `debug_ui_html()` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false |

Unknown routes return `404 Not Found` with body `not found`.

Removed `/ha/leader` routes also return `404 Not Found` and do not mutate DCS state. `tests/policy_e2e_api_only.rs` enforces observation through `GET /ha/state` plus admin switchover requests instead of direct controller or worker steering after startup.

## Request Inputs

### `SwitchoverRequestInput`

| Field | Type |
|---|---|
| `requested_by` | `MemberId` |

`#[serde(deny_unknown_fields)]`. Blank `requested_by` returns `400 Bad Request` with body `requested_by must be non-empty`.

Successful requests are serialized to DCS `SwitchoverRequest` JSON and written to `/{scope}/switchover` after trimming leading and trailing `/` from `scope`.

### `FallbackHeartbeatInput`

| Field | Type |
|---|---|
| `source` | `String` |

`#[serde(deny_unknown_fields)]`. Blank `source` returns `400 Bad Request` with body `source must be non-empty`.

### `GET /debug/verbose` Query

`parse_since_sequence` ignores unrelated query pairs, returns `None` when `since` is absent, and invalid integer values return `invalid since query parameter: <parse error>`.

## Response Payloads

### `AcceptedResponse`

| Field | Type |
|---|---|
| `accepted` | `bool` |

### `FallbackClusterView`

| Field | Type |
|---|---|
| `name` | `String` |

### `HaStateResponse`

| Field | Type |
|---|---|
| `cluster_name` | `String` |
| `scope` | `String` |
| `self_member_id` | `String` |
| `leader` | `Option<String>` |
| `switchover_requested_by` | `Option<String>` |
| `member_count` | `usize` |
| `dcs_trust` | `DcsTrustResponse` |
| `ha_phase` | `HaPhaseResponse` |
| `ha_tick` | `u64` |
| `ha_decision` | `HaDecisionResponse` |
| `snapshot_sequence` | `u64` |

`DcsTrustResponse` values: `full_quorum`, `fail_safe`, `not_trusted`.

`HaPhaseResponse` values: `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe`.

### `HaDecisionResponse`

`HaDecisionResponse` is tagged with field `kind` in `snake_case`.

| `kind` | Additional fields |
|---|---|
| `no_change` | none |
| `wait_for_postgres` | `start_requested`, `leader_member_id` |
| `wait_for_dcs_trust` | none |
| `attempt_leadership` | none |
| `follow_leader` | `leader_member_id` |
| `become_primary` | `promote` |
| `step_down` | `reason`, `release_leader_lease`, `clear_switchover`, `fence` |
| `recover_replica` | `strategy` |
| `fence_node` | none |
| `release_leader_lease` | `reason` |
| `enter_fail_safe` | `release_leader_lease` |

`step_down.reason` is tagged with `kind` and supports:

- `switchover`
- `foreign_leader_detected { leader_member_id }`

`recover_replica.strategy` is tagged with `kind` and supports:

- `rewind { leader_member_id }`
- `base_backup { leader_member_id }`
- `bootstrap`

`release_leader_lease.reason` supports:

- `fencing_complete`
- `postgres_unreachable`

## Error Mapping

| API error | HTTP status |
|---|---|
| `ApiError::BadRequest` | `400 Bad Request` |
| `ApiError::DcsStore` | `503 Service Unavailable` |
| `ApiError::Internal` | `500 Internal Server Error` |

Additional route-level error cases:

| Condition | Result |
|---|---|
| request-parse failure before routing | `400 Bad Request` with parser message |
| JSON parse failure on `POST /switchover` or `POST /fallback/heartbeat` | `400 Bad Request` with body `invalid json: <serde error>` |
| no debug snapshot subscriber for `GET /ha/state`, `GET /debug/snapshot`, or `GET /debug/verbose` | `503 Service Unavailable` with body `snapshot unavailable` |
| debug endpoint requested while `cfg.debug.enabled` is false | `404 Not Found` with body `not found` |
