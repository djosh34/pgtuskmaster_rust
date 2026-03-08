# HTTP API Reference

The HTTP API exposes the built-in control and observation routes implemented by the API worker in `src/api/worker.rs`, `src/api/controller.rs`, and `src/api/fallback.rs`.

## Worker Loop And Transport

`ApiWorkerCtx` fields: `listener`, `poll_interval`, `scope`, `member_id`, `config_subscriber`, `dcs_store`, `debug_snapshot_subscriber`, `tls_mode_override`, `tls_acceptor`, `role_tokens`, `require_client_cert`, `log`.

`ApiWorkerCtx::new` copies `scope` from `cfg.dcs.scope`, copies `member_id` from `cfg.cluster.member_id`, sets `poll_interval = 10 ms`, and initializes `debug_snapshot_subscriber`, `tls_mode_override`, `tls_acceptor`, and `role_tokens` to `None` with `require_client_cert = false`.

`api::worker::run` loops forever:

1. call `step_once`
2. log `api.step_once_failed` when `step_once` returns an error
3. return the error only when it is classified as fatal
4. sleep for `poll_interval`

Fatal step errors are messages containing:

- `api accept failed`
- `tls mode requires a configured tls acceptor`
- `api local_addr failed`

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

Parse failures before routing return `400 Bad Request` with the parser's message body and log `api.request_parse_failed`.

## Security Configuration

### Authentication

Authentication configuration normally comes from `cfg.api.security.auth` and may be overridden with `ApiWorkerCtx.configure_role_tokens(read_token, admin_token)`.

Auth config variants:

- `ApiAuthConfig::Disabled`
- `ApiAuthConfig::RoleTokens(ApiRoleTokensConfig { read_token, admin_token })`

Runtime token normalization:

- `normalize_runtime_token` trims configured runtime-config token strings
- blank runtime-config token strings become `None`
- `configure_role_tokens` uses `normalize_optional_token`, which rejects blank override strings with `WorkerError::Message("role token must not be empty when configured")`
- if both resolved tokens are absent, requests are allowed without authorization

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
| request targets a read route and bearer token matches `read_token` | request allowed |
| request targets an admin route and bearer token matches `read_token` | `403 Forbidden` with body `forbidden` |
| any other token mismatch | `401 Unauthorized` with body `unauthorized` |

Auth decision logs include:

- `api.method`
- `api.route_template`
- `api.auth.header_present`
- `api.auth.result`
- `api.auth.required_role`
- optional `api.request_id`

`x-request-id` is trimmed, empty values are ignored, and non-empty values are truncated to `128` characters.

### TLS

TLS configuration normally comes from `cfg.api.security.tls` and may be overridden with `ApiWorkerCtx.configure_tls(mode, server_config)`.

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
| `disabled` | connection stays plain TCP |
| `required` | always start a TLS handshake |
| `optional` | peek one byte for up to `10 ms`; if the byte is not `0x16`, or the peek times out, or returns `WouldBlock`, stay plain TCP; otherwise start a TLS handshake |

Missing TLS acceptor in optional or required mode returns `WorkerError::Message("tls mode requires a configured tls acceptor")`.

TLS handshake failures are logged as `api.tls_handshake_failed` and the connection is dropped without an HTTP response.

If `require_client_cert` is true and the accepted TLS stream has no peer certificate, the worker logs `api.tls_client_cert_missing` and drops the connection without an HTTP response.

## Route Table

| Method | Path | Role | Success status | Success payload | Notes |
|---|---|---|---|---|---|
| `POST` | `/switchover` | admin | `202 Accepted` | `AcceptedResponse` | writes serialized `SwitchoverRequest` JSON to `/{scope}/switchover` |
| `DELETE` | `/ha/switchover` | admin | `202 Accepted` | `AcceptedResponse` | clears switchover through `DcsHaWriter::clear_switchover` |
| `GET` | `/ha/state` | read | `200 OK` | `HaStateResponse` | returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/fallback/cluster` | read | `200 OK` | `FallbackClusterView` | `name` is copied from `cfg.cluster.name` |
| `POST` | `/fallback/heartbeat` | admin | `202 Accepted` | `AcceptedResponse` | request body is `FallbackHeartbeatInput` |
| `GET` | `/debug/snapshot` | read | `200 OK` | text from `format!("{:#?}", snapshot)` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/debug/verbose` | read | `200 OK` | JSON from `build_verbose_payload` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured; invalid `since` returns `400 Bad Request` with body `invalid since query parameter: <parse error>` |
| `GET` | `/debug/ui` | read | `200 OK` | HTML from `debug_ui_html()` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false |

Unknown routes return `404 Not Found` with body `not found`.

Removed `/ha/leader` routes also return `404 Not Found` and do not mutate DCS state. `tests/policy_e2e_api_only.rs` treats direct controller and worker steering as forbidden in HA end-to-end sources and allows observation through `GET /ha/state` plus admin switchover requests.

## Request Inputs

### `SwitchoverRequestInput`

| Field | Type |
|---|---|
| `requested_by` | `MemberId` |

Behavior:

- `#[serde(deny_unknown_fields)]`
- blank `requested_by` returns `400 Bad Request` with body `requested_by must be non-empty`

Successful requests are serialized to DCS `SwitchoverRequest` JSON and written to `/{scope}/switchover` after trimming leading and trailing `/` from `scope`.

### `FallbackHeartbeatInput`

| Field | Type |
|---|---|
| `source` | `String` |

Behavior:

- `#[serde(deny_unknown_fields)]`
- blank `source` returns `400 Bad Request` with body `source must be non-empty`

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

`step_down.reason` is tagged with `kind` and supports `switchover` and `foreign_leader_detected { leader_member_id }`.

`recover_replica.strategy` is tagged with `kind` and supports `rewind { leader_member_id }`, `base_backup { leader_member_id }`, and `bootstrap`.

`release_leader_lease.reason` supports `fencing_complete` and `postgres_unreachable`.

## Error Mapping

| API error | HTTP status |
|---|---|
| `ApiError::BadRequest` | `400 Bad Request` |
| `ApiError::DcsStore` | `503 Service Unavailable` |
| `ApiError::Internal` | `500 Internal Server Error` |

Additional route-level error cases:

| Condition | Result |
|---|---|
| request-parse failure before routing | `400 Bad Request` with the parser's message |
| JSON parse failure on `POST /switchover` or `POST /fallback/heartbeat` | `400 Bad Request` with body `invalid json: <serde error>` |
| no debug snapshot subscriber for `GET /ha/state`, `GET /debug/snapshot`, or `GET /debug/verbose` | `503 Service Unavailable` with body `snapshot unavailable` |
| debug endpoint requested while `cfg.debug.enabled` is false | `404 Not Found` with body `not found` |

## Verified Behaviors

### Switchover Surface

- `src/api/controller.rs` tests verify `SwitchoverRequestInput` denies unknown fields.
- `src/api/controller.rs` tests verify `post_switchover` writes a typed `SwitchoverRequest` record to `/{scope}/switchover`.
- `src/api/controller.rs` tests verify `post_switchover` rejects empty `requested_by`.
- `src/api/controller.rs` tests verify `delete_switchover` deletes `/{scope}/switchover`.

### Fallback Surface

- `src/api/fallback.rs` tests verify `FallbackHeartbeatInput` denies unknown fields.
- `src/api/fallback.rs` tests verify `GET /fallback/cluster` returns `cfg.cluster.name`.
- `src/api/fallback.rs` tests verify `POST /fallback/heartbeat` rejects an empty `source`.

### Route And Auth Coverage

- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify read-role tokens can read but cannot call admin routes.
- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify admin-role tokens can call admin routes.
- `src/api/worker.rs` tests verify auth decision logs do not leak bearer tokens.
- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify `GET /ha/state` returns typed JSON and returns `503` when the snapshot subscriber is missing.
- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify removed `/ha/leader` routes return `404` and do not mutate DCS keys.
- `src/api/worker.rs` tests verify legacy auth fallback can protect HA routes and that API role tokens take precedence over legacy tokens.

### Debug And TLS Coverage

- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify `GET /debug/verbose` returns structured output and honors the `since` filter.
- `src/api/worker.rs` tests verify `GET /debug/snapshot` remains available as a backward-compatible debug snapshot body.
- `src/api/worker.rs` and `tests/bdd_api_http.rs` verify debug routes return `404` when debug is disabled, `503` without a snapshot subscriber, and HTML for `GET /debug/ui`.
- `src/api/worker.rs` tests verify debug routes require authentication when tokens are configured.
- `src/api/worker.rs` tests verify disabled, optional, and required TLS modes for plain and TLS connections.
- `src/api/worker.rs` tests verify TLS and mTLS flows through the production rustls builder.
- `src/api/worker.rs` tests verify handshake failure on wrong CA, hostname mismatch, and expired certificates, plus rejection of untrusted or missing client certificates in mTLS mode.

### End-To-End Policy Coverage

- `tests/policy_e2e_api_only.rs` verifies HA end-to-end sources use `GET /ha/state` and admin switchover requests instead of direct controller or DCS steering after startup.
