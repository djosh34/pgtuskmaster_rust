# HTTP API deep summary

This support note is only raw factual context for `docs/src/reference/http-api.md`.
Prefer exact endpoint names, methods, auth roles, request/response types, and status behavior from the code. Do not invent features such as authentication schemes beyond bearer role tokens and optional TLS.

Public API surface from `src/api/worker.rs` route matching:

- `POST /switchover`
- `DELETE /ha/switchover`
- `GET /ha/state`
- `GET /fallback/cluster`
- `POST /fallback/heartbeat`
- `GET /debug/snapshot`
- `GET /debug/verbose`
- `GET /debug/ui`

Authentication and authorization behavior:

- `src/api/worker.rs` uses bearer-token auth when role tokens are configured.
- If both read/admin tokens are absent, requests are allowed.
- If tokens are configured and the request has no bearer token, the result is `401 Unauthorized`.
- If an endpoint requires admin and the request presents only the read token, the result is `403 Forbidden`.
- Admin endpoints are:
  - `POST /switchover`
  - `POST /fallback/heartbeat`
  - `DELETE /ha/switchover`
- All other listed routes are read endpoints.
- Requests use the `Authorization: Bearer <token>` header.
- Optional or required TLS can be configured; TLS is not itself the authorization model.

Generic HTTP status/error conventions:

- Unknown routes return `404 Not Found` with body `not found`.
- Invalid JSON on JSON-taking endpoints returns `400 Bad Request` with a message like `invalid json: ...`.
- `ApiError::BadRequest` maps to `400 Bad Request`.
- `ApiError::DcsStore` maps to `503 Service Unavailable`.
- `ApiError::Internal` maps to `500 Internal Server Error`.
- Some read routes can also return `503 Service Unavailable` when snapshot state is unavailable.

Endpoint details:

1. `GET /ha/state`
   - Returns `200 OK`.
   - Response type is `HaStateResponse`.
   - Fields are:
     - `cluster_name: String`
     - `scope: String`
     - `self_member_id: String`
     - `leader: Option<String>`
     - `switchover_requested_by: Option<String>`
     - `member_count: usize`
     - `dcs_trust: DcsTrustResponse`
     - `ha_phase: HaPhaseResponse`
     - `ha_tick: u64`
     - `ha_decision: HaDecisionResponse`
     - `snapshot_sequence: u64`
   - `DcsTrustResponse` values: `full_quorum`, `fail_safe`, `not_trusted`.
   - `HaPhaseResponse` values: `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe`.
   - `HaDecisionResponse` is a tagged enum with variants for no-change, waiting, leadership, follow, become-primary, step-down, replica recovery, fencing, leader-lease release, and fail-safe entry.
   - If no debug snapshot subscriber is attached, the route returns `503 Service Unavailable` with body `snapshot unavailable`.

2. `POST /switchover`
   - Requires admin authorization.
   - Request type is `SwitchoverRequestInput { requested_by: MemberId }`.
   - Unknown fields are denied by serde.
   - Empty or whitespace-only `requested_by` returns `400 Bad Request`.
   - On success, writes a typed switchover request to `/{scope}/switchover` in DCS and returns `202 Accepted` with `AcceptedResponse { accepted: true }`.

3. `DELETE /ha/switchover`
   - Requires admin authorization.
   - Clears the switchover request through the DCS HA writer.
   - Returns `202 Accepted` with `AcceptedResponse { accepted: true }` on success.

4. `GET /fallback/cluster`
   - Read authorization.
   - Returns `200 OK`.
   - Response type is `FallbackClusterView { name: String }`, where `name` is the configured cluster name.

5. `POST /fallback/heartbeat`
   - Requires admin authorization.
   - Request type is `FallbackHeartbeatInput { source: String }`.
   - Unknown fields are denied by serde.
   - Empty or whitespace-only `source` returns `400 Bad Request`.
   - Success returns `202 Accepted` with `AcceptedResponse { accepted: true }`.

6. `GET /debug/snapshot`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - If snapshot subscriber is missing, returns `503 Service Unavailable` with body `snapshot unavailable`.
   - On success returns `200 OK` and emits a pretty `Debug` string representation of the full system snapshot, not a stable JSON schema.

7. `GET /debug/verbose`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - Optional query parameter: `since=<u64>`.
   - Invalid `since` parsing returns `400 Bad Request`.
   - On success returns `200 OK` with a structured JSON payload assembled by `build_verbose_payload`.
   - That payload includes:
     - `meta`
     - `config`
     - `pginfo`
     - `dcs`
     - `process`
     - `ha`
     - `api`
     - `debug`
     - `changes`
     - `timeline`
   - The `api.endpoints` list in the payload is:
     - `/debug/snapshot`
     - `/debug/verbose`
     - `/debug/ui`
     - `/fallback/cluster`
     - `/switchover`
     - `/ha/state`
     - `/ha/switchover`

8. `GET /debug/ui`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - Returns `200 OK` with HTML content for the built-in debug UI.

Important docs caveats:

- `GET /debug/snapshot` returns debug-formatted text, not a documented JSON schema.
- The route list in `api.endpoints` is a helpful summary surface, but the authoritative behavior is the `route_request` match in `src/api/worker.rs`.
- There is no source-backed evidence in these files for additional auth mechanisms beyond bearer role tokens plus optional TLS/client-cert handling.
