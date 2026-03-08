# HTTP API Reference

The HTTP API worker accepts at most one request per accepted connection, negotiates plain or TLS transport, authorizes the request, routes it, and writes one HTTP response.

## Worker And Transport Behavior

### `ApiWorkerCtx`

`ApiWorkerCtx` fields:

| Field |
|---|
| `listener` |
| `poll_interval` |
| `scope` |
| `member_id` |
| `config_subscriber` |
| `dcs_store` |
| `debug_snapshot_subscriber` |
| `tls_mode_override` |
| `tls_acceptor` |
| `role_tokens` |
| `require_client_cert` |
| `log` |

`ApiWorkerCtx::new`:

- copies `scope` from `cfg.dcs.scope`
- copies `member_id` from `cfg.cluster.member_id`
- sets `poll_interval` to `10 ms`
- sets `debug_snapshot_subscriber`, `tls_mode_override`, `tls_acceptor`, and `role_tokens` to `None`
- sets `require_client_cert` to `false`

### Worker Loop

`api::worker::run` loops forever. Each iteration:

1. calls `step_once`
2. logs `api.step_once_failed` on error
3. returns only fatal step errors
4. sleeps for `API_LOOP_POLL_INTERVAL`

Fatal step error messages:

| Message |
|---|
| `api accept failed` |
| `tls mode requires a configured tls acceptor` |
| `api local_addr failed` |

### Request Processing Constants

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

### Per-Connection Flow

`step_once`:

1. accepts at most one connection attempt per loop iteration
2. returns `Ok(())` when accept times out
3. logs `api.connection_accepted` with peer address and effective TLS mode after accept
4. negotiates plain or TLS transport
5. reads at most one HTTP request
6. authorizes the request
7. routes the request
8. writes one HTTP response
9. logs `api.response_sent`

Request-read timeout returns `Ok(())` without writing an HTTP response.

Parse failures before routing return `400 Bad Request` with the parser message body and log `api.request_parse_failed`.

## Authentication

Authentication resolves from `cfg.api.security.auth` unless overridden with `ApiWorkerCtx.configure_role_tokens(read_token, admin_token)`.

### Config Variants

| Variant |
|---|
| `ApiAuthConfig::Disabled` |
| `ApiAuthConfig::RoleTokens(ApiRoleTokensConfig { read_token, admin_token })` |

Runtime token normalization trims configured runtime-config token strings and converts blank strings to `None`.

`configure_role_tokens` rejects blank override strings with `WorkerError::Message("role token must not be empty when configured")`.

If both resolved tokens are absent, requests are allowed without authorization.

Authorization-header lookup is case-insensitive. Bearer extraction trims the full header value, requires the exact prefix `Bearer `, trims the remainder, and rejects an empty remainder.

### Route Roles

| Role | Routes |
|---|---|
| admin | `POST /switchover`, `POST /fallback/heartbeat`, `DELETE /ha/switchover` |
| read | all other supported routes |

### Authorization Outcomes

| Condition | Result |
|---|---|
| both resolved tokens absent | request allowed without authorization |
| token protection configured and no bearer token extracted | `401 Unauthorized` with body `unauthorized` |
| bearer token matches `admin_token` | request allowed |
| bearer token matches `read_token` and request is a read route | request allowed |
| bearer token matches `read_token` and request is an admin route | `403 Forbidden` with body `forbidden` |
| any other token mismatch | `401 Unauthorized` with body `unauthorized` |

Auth decision logs include:

| Field |
|---|
| `api.method` |
| `api.route_template` |
| `api.auth.header_present` |
| `api.auth.result` |
| `api.auth.required_role` |
| `api.request_id` |

`x-request-id` values are trimmed. Empty values are ignored. Non-empty values are truncated to `128` characters.

## TLS

TLS configuration resolves from `cfg.api.security.tls` unless overridden with `ApiWorkerCtx.configure_tls(mode, server_config)`.

### TLS Config Fields

| Field | Type |
|---|---|
| `mode` | `ApiTlsMode` |
| `identity` | `Option<TlsServerIdentityConfig>` |
| `client_auth` | `Option<TlsClientAuthConfig>` |

`ApiTlsMode` values:

- `disabled`
- `optional`
- `required`

### Identity And Client Auth Fields

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

### Override Behavior

`configure_tls`:

- clears the acceptor in disabled mode
- requires a server config for optional or required mode

Calling `configure_tls` without a server config in optional or required mode returns `WorkerError::Message("tls mode optional/required requires a server tls config")`.

Effective TLS mode uses the override when present, otherwise `cfg.api.security.tls.mode`.

### Connection Handling By Effective TLS Mode

| Mode | Accept behavior |
|---|---|
| `disabled` | connection stays plain TCP |
| `required` | always starts a TLS handshake |
| `optional` | peeks one byte for up to `10 ms`; if the byte is not `0x16`, or the peek times out, or returns `WouldBlock`, the connection stays plain TCP; otherwise the worker starts a TLS handshake |

Missing TLS acceptor in optional or required mode returns `WorkerError::Message("tls mode requires a configured tls acceptor")`.

TLS handshake failures log `api.tls_handshake_failed` and drop the connection without an HTTP response.

If `require_client_cert` is `true` and the accepted TLS stream has no peer certificate, the worker logs `api.tls_client_cert_missing` and drops the connection without an HTTP response.

## Route Table

| Method | Path | Role | Success status | Success payload | Notes |
|---|---|---|---|---|---|
| `POST` | `/switchover` | admin | `202 Accepted` | `AcceptedResponse` | writes serialized `SwitchoverRequest` JSON to `/{scope}/switchover` after trimming leading and trailing `/` from `scope` |
| `DELETE` | `/ha/switchover` | admin | `202 Accepted` | `AcceptedResponse` | clears switchover through `DcsHaWriter::clear_switchover` |
| `GET` | `/ha/state` | read | `200 OK` | `HaStateResponse` | returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/fallback/cluster` | read | `200 OK` | `FallbackClusterView` | `name` copies `cfg.cluster.name` |
| `POST` | `/fallback/heartbeat` | admin | `202 Accepted` | `AcceptedResponse` | request body is `FallbackHeartbeatInput` |
| `GET` | `/debug/snapshot` | read | `200 OK` | text from `format!("{:#?}", snapshot)` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured |
| `GET` | `/debug/verbose` | read | `200 OK` | JSON from `build_verbose_payload` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; returns `503 Service Unavailable` with body `snapshot unavailable` when no snapshot subscriber is configured; invalid `since` returns `400 Bad Request` with body `invalid since query parameter: <parse error>` |
| `GET` | `/debug/ui` | read | `200 OK` | HTML from `debug_ui_html()` | returns `404 Not Found` with body `not found` when `cfg.debug.enabled` is false |

Unknown routes return `404 Not Found` with body `not found`.

Removed `/ha/leader` routes also return `404 Not Found` and do not mutate DCS state.

`tests/policy_e2e_api_only.rs` enforces post-start observation through `GET /ha/state` plus admin switchover requests instead of direct controller or worker steering after startup.

## Request Payloads And Query Parameters

### `SwitchoverRequestInput`

| Field | Type |
|---|---|
| `requested_by` | `MemberId` |

`#[serde(deny_unknown_fields)]`

Blank `requested_by` returns `400 Bad Request` with body `requested_by must be non-empty`.

Successful requests are serialized to DCS `SwitchoverRequest` JSON and written to `/{scope}/switchover` after trimming leading and trailing `/` from `scope`.

### `FallbackHeartbeatInput`

| Field | Type |
|---|---|
| `source` | `String` |

`#[serde(deny_unknown_fields)]`

Blank `source` returns `400 Bad Request` with body `source must be non-empty`.

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

`DcsTrustResponse` values:

- `full_quorum`
- `fail_safe`
- `not_trusted`

`HaPhaseResponse` values:

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

`step_down.reason` variants:

- `switchover`
- `foreign_leader_detected { leader_member_id }`

`recover_replica.strategy` variants:

- `rewind { leader_member_id }`
- `base_backup { leader_member_id }`
- `bootstrap`

`release_leader_lease.reason` variants:

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
