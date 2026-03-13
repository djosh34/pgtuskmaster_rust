## Task: Replace The Hand-Rolled API Server With `axum` + `axum-server` + `tower` <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Replace the current hand-rolled HTTP server in `src/api/worker.rs` with a framework-managed API stack built on `axum`, `axum-server`, and `tower`/`tower-http`. The higher-order goal is to remove self-authored transport code, stop treating the API like a poll/sleep worker, and collapse the API implementation down to route handlers plus explicit middleware and TLS configuration. The resulting API must keep the public business surface small and boring while making transport/security behavior stricter, clearer, and easier to maintain.

This task is about the node HTTP API server only. The current business routes are simple:
- `GET /state`
- `POST /switchover`
- `DELETE /switchover`

The current complexity is mostly self-inflicted transport logic:
- custom accept loop with `step_once` + sleep
- manual TLS/plaintext peeking and handshake branching
- manual HTTP request parsing with `httparse`
- manual response framing
- request auth implemented inside the route worker instead of middleware

The target design is:
- `axum` routes and handlers for all API endpoints
- `axum-server` for serving with Rustls and future reload support
- `tower`/`tower-http` layers for middleware concerns
- no hand-written HTTP parser, no hand-written HTTP response formatter, no custom request routing, no API polling loop
- direct request handling: request arrives -> middleware/TLS checks run -> handler gathers latest state from local watchers -> response is serialized through `serde`

This rewrite must preserve the current product behavior where it still makes sense, but it must not preserve accidental transport complexity. This repo is greenfield with no backwards compatibility requirement. If the current server shape or config model exists only to support custom transport tricks, remove that shape instead of rebuilding it on top of a framework.

The framework choice is already decided by prior research for this task:
- use `axum`
- use `axum-server`
- use `tower` and `tower-http`

Do not spend time re-evaluating other frameworks inside implementation.

**Scope:**
- Replace the current API server implementation under `src/api/` and its runtime wiring in `src/runtime/node.rs`.
- Keep the business handlers for `/state` and `/switchover` centered on the existing state/controller logic where possible.
- Move bearer-token authorization into middleware/layers instead of handler-local branching.
- Keep TLS client certificate verification rooted only in configured CA material, never OS trust, and add an optional client-cert common-name allow-list.
- Add a dedicated cert reload endpoint for the API server TLS material and client-auth verifier material. PostgreSQL reload is explicitly out of scope.
- Remove the custom `step_once` API loop and any timeout/sleep structure that exists only because the API was forced into the generic worker pattern.
- Remove manual HTTP parsing/serialization and ensure framework-managed HTTP/1.1 and HTTP/2 support are enabled.
- Reuse existing `serde` request/response types instead of manual JSON byte handling or custom JSON wrappers.
- Delete obsolete transport code and stale config/docs/tests that only existed for the manual server.

Concrete route surface after the rewrite:
- `GET /state`
- `POST /switchover`
- `DELETE /switchover`
- `POST /reload/certs`

Concrete security requirements:
- API bearer auth is middleware-based.
- Client certificates are validated only against the configured API client CA, not the OS trust store.
- If an optional allow-list of client certificate common names is configured, clients whose common name is not in that allow-list must be rejected even when the cert chain is otherwise valid and anchored in the configured CA.
- The CN allow-list is an API server requirement only; broader SAN-based identity matching is out of scope for this task.

Concrete reload requirement:
- `POST /reload/certs` reloads API TLS identity, client-auth CA bundle, and client-cert common-name allow-list from the already configured runtime-config sources.
- It does not reload PostgreSQL TLS.
- It does not implement broad runtime-config reload.
- It must be protected by admin auth middleware.

Concrete architecture requirement:
- The API server is no longer a “step” worker.
- The runtime should spawn/await the server future directly.
- `GET /state` should gather fresh local snapshots by calling `.latest()` on the already-wired local watchers/subscribers at request time, then return the assembled `NodeState`.
- No periodic sleep should be involved in serving requests.

Concrete protocol requirement:
- Do not keep or recreate manual HTTP parsing.
- Do not keep or recreate manual response writing.
- Do not keep any custom TLS/plaintext client-hello peeking.
- Enable HTTP/1.1 and HTTP/2 through the chosen server stack instead of implementing protocol behavior by hand.

Concrete serialization requirement:
- Reuse `serde` types already present in `src/api/mod.rs` and `src/api/controller.rs` where applicable.
- New request/response bodies, if any, must also be normal `serde` structs/enums.
- Handlers should use framework JSON extractors/response types, not manual `serde_json::from_slice` and not manual `serde_json::to_vec`.

Concrete cleanup requirement:
- The hand-rolled server code should be removed, not left alongside the new stack “just in case”.
- The old API `step_once` tests that depend on a manual `TcpListener` + raw socket drive loop should be replaced with framework-appropriate integration tests.
- Remove dependencies that become transport leftovers, especially `httparse`, and remove direct `tokio-rustls` usage from the API path if the new server stack no longer needs it there.

Important config direction:
- The current `api.security.tls.mode` enum includes `disabled`, `optional`, and `required`.
- The existing `optional` mode is tied to custom same-port TLS/plaintext sniffing in `src/api/worker.rs`.
- Do not recreate that mixed-mode same-port behavior on top of `axum`.
- Because the repo has no backwards compatibility requirement and the goal is code reduction, this task should simplify the API TLS mode model rather than preserve a custom transport trick. The expected direction is:
  - keep `disabled`
  - keep a TLS-enabled mode
  - remove or hard-reject the mixed-mode `optional` API behavior if it would require custom peeking/sniffing again
- Update config parsing, validation, docs, and shipped config examples to match the simplified model.

Config addition required by this task:
- Extend the API client-auth config with an optional allow-list field for accepted client certificate common names.
- Recommended field shape:

```toml
[api.security.tls.client_auth]
client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }
require_client_cert = true
allowed_common_names = ["ops-admin", "pgtm-client"]
```

Validation rules required by this task:
- `allowed_common_names` must reject empty strings after trimming.
- If `allowed_common_names` is configured, `client_auth` must exist.
- If `allowed_common_names` is configured, the API server must require client certificates. Do not allow a config that defines an allow-list while also allowing unauthenticated clients.
- Keep using only the configured `client_ca` material as the trust root for mTLS client verification.

HTTP API contract for the new reload route:
- `POST /reload/certs`
- Authorization: admin
- Success: `200 OK`
- Response body:

```json
{
  "reloaded": true
}
```

- Failure cases:
  - `401 Unauthorized` for missing/invalid token
  - `403 Forbidden` for read token on admin route
  - `500 Internal Server Error` or `503 Service Unavailable` when reload fails; choose one status code family and document it clearly in code and docs rather than inventing ambiguous behavior

**Context from research:**
- Current API assembly happens in `src/runtime/node.rs`, which builds `ApiWorkerCtx`, wires the live state subscribers, configures TLS, and runs `crate::api::worker::run(api_ctx)`.
- `GET /state` already has the right data flow conceptually: it reads the latest local `watch` snapshots for `pg`, `process`, `dcs`, and `ha`, then wraps them into `NodeState` through `build_node_state(...)`.
- The route business logic is already small:
  - `src/api/controller.rs` contains `build_node_state`, `post_switchover`, and `delete_switchover`
  - `src/api/mod.rs` already owns `NodeState` and `AcceptedResponse`
- The present bloat is transport code in `src/api/worker.rs`, including:
  - raw listener accept loop
  - TLS/plaintext branching
  - manual request parsing with `httparse`
  - custom `HttpRequest` / `HttpResponse`
  - in-worker auth checks and route dispatch
- `src/state/watch_state.rs` already provides the `StateSubscriber<T>::latest()` behavior needed by request handlers.
- The current TLS builder in `src/tls.rs` already uses only the configured CA bundle for client auth, not the OS trust store. Preserve that property when adapting it to the new server stack.
- Current API auth config lives under `api.security.auth` and currently resolves bearer tokens from config/secret sources. Keep the token source model, but enforce it through middleware instead of handler logic.
- `pgtm` and docs intentionally consume a single seed `/state`; the server rewrite should not change the API business surface of `/state` itself.

Likely code areas to change:
- `Cargo.toml`
  - add `axum`
  - add `axum-server`
  - add `tower`
  - add `tower-http` as needed
  - remove `httparse`
  - remove any API-only transport deps made obsolete by the new stack
- `src/api/mod.rs`
  - keep or extend shared API response/request types
  - add a typed `ReloadCertificatesResponse`
- `src/api/controller.rs`
  - keep business logic for state and switchover
  - add the reload action entry point or a small companion module if that keeps responsibilities cleaner
- `src/api/worker.rs`
  - remove the current hand-rolled transport implementation entirely or replace the file with a tiny server bootstrap that constructs the router and serves it
  - there should be no remaining manual HTTP parser, manual response formatter, or `step_once`
- `src/runtime/node.rs`
  - stop constructing/running the API as a poll worker
  - run the axum server future directly
  - wire the application state shared with handlers/middleware/reload path
- `src/tls.rs`
  - adapt certificate loading and verifier construction to the new server stack
  - add the common-name allow-list enforcement path
  - add tests for wrong-CA rejection and wrong-common-name rejection
  - add reload-friendly construction so the API listener can swap TLS config without process restart
- `src/config/schema.rs`
  - extend `TlsClientAuthConfig` with `allowed_common_names`
  - simplify/remove `ApiTlsMode::Optional` if implementation follows the expected direction
- `src/config/parser.rs`
  - validate the new allow-list rules
  - validate/simplify the API TLS mode model so the old mixed-mode API transport is gone
- `tests/bdd_api_http.rs`
  - replace manual socket/`step_once`-driven tests with direct server integration tests
  - cover auth middleware and route status codes through real HTTP requests
- `src/test_harness/tls.rs` and related TLS helpers
  - reuse the existing adversarial TLS fixtures for wrong-CA and CN-allow-list coverage
- `tests/cli_binary.rs` and any CLI/API integration tests
  - keep `GET /state` compatibility expectations intact
  - add coverage only where client-observable API behavior changes
- `docs/src/reference/http-api.md`
  - document the axum-backed route surface without transport implementation details
  - add `POST /reload/certs`
- `docs/src/reference/runtime-configuration.md`
  - document the new `allowed_common_names`
  - remove/update any mention of `api.security.tls.mode = "optional"` if that mode is deleted/rejected
- `docs/src/how-to/configure-tls.md` and `docs/src/how-to/configure-tls-security.md`
  - explain the mTLS CA restriction and optional CN allow-list clearly
  - document cert reload for the API server only
- shipped example configs under `docker/configs/` and any docs snippets that mention the old TLS mode or old API behavior

Implementation guidance:
- Prefer a shared application state struct for handlers that contains:
  - runtime config subscriber
  - state subscribers for `pg`, `process`, `dcs`, `ha`
  - DCS store handle for switchover operations
  - reloadable TLS/config material handle for the cert reload route
  - log handle if request/reload events still need application-event emission
- Route handlers should be thin:
  - `/state` reads subscribers and returns `Json<NodeState>`
  - `/switchover` handlers delegate to existing controller logic and return typed JSON responses/errors
  - `/reload/certs` invokes a reload service and returns `Json<ReloadCertificatesResponse>`
- Bearer auth should be layered by route role:
  - read routes accept read token or admin token
  - admin routes require admin token
  - route handlers should not manually branch on authorization header presence
- Keep error handling explicit and typed. Do not swallow reload or TLS errors.
- Prefer code deletion over adapter layers. The best outcome is much less API code than today.

**Expected outcome:**
- `src/api/worker.rs` is no longer a large hand-rolled HTTP/TLS server.
- There is no API `step_once` loop and no sleep-based request polling.
- The node API is served by `axum`/`axum-server` with framework-managed HTTP/1.1 and HTTP/2 support.
- `/state` still returns the same conceptual `NodeState`, built fresh from current local watcher snapshots at request time.
- Switchover routes still behave correctly, but auth is enforced by middleware instead of inline route logic.
- mTLS client verification trusts only the configured client CA, not OS roots.
- An optional client common-name allow-list can hard-reject otherwise valid client certs.
- The API has an authenticated cert reload endpoint for server/API TLS material only.
- Config/docs/tests match the simplified non-hand-rolled server model.
- The overall API implementation is materially smaller and easier to reason about than the current manual transport stack.

</description>

<acceptance_criteria>
- [ ] `Cargo.toml` adds `axum`, `axum-server`, and the required `tower`/`tower-http` crates for the new server stack, and removes `httparse` plus any API-only transport leftovers that become unused.
- [ ] `src/api/worker.rs` no longer contains a manual accept loop, manual request parser, manual response writer, custom `HttpRequest`/`HttpResponse` transport structs, TLS/plaintext peek logic, or an API `step_once` function.
- [ ] The runtime wiring in `src/runtime/node.rs` no longer treats the API as a poll/sleep worker; it serves the HTTP server directly as a server future.
- [ ] `GET /state` still assembles `NodeState` from the latest local `pg`, `process`, `dcs`, and `ha` subscribers at request time, with no periodic API polling loop involved.
- [ ] `POST /switchover` and `DELETE /switchover` continue to use typed `serde` request/response models and preserve their current business behavior.
- [ ] `POST /reload/certs` exists, is admin-protected, reloads API TLS identity plus client-auth CA/allow-list material, and explicitly does not reload PostgreSQL TLS.
- [ ] API auth is implemented as middleware/layers rather than handler-local auth branching.
- [ ] Read routes accept a read token or an admin token, and admin routes require an admin token.
- [ ] API TLS client verification trusts only the configured client CA material and does not fall back to OS-trusted roots for client-certificate acceptance.
- [ ] `src/config/schema.rs` extends the API client-auth config with an optional `allowed_common_names` list.
- [ ] `src/config/parser.rs` validates that `allowed_common_names` cannot contain empty values and cannot be configured without client-auth and required client certificates.
- [ ] Clients with a valid chain from the configured CA but a common name outside `allowed_common_names` are rejected.
- [ ] The old custom same-port mixed plaintext/TLS behavior is removed rather than reimplemented; config, validation, examples, and docs are updated accordingly.
- [ ] The server stack is configured to support HTTP/1.1 and HTTP/2 without custom HTTP protocol code.
- [ ] No API route manually parses JSON bodies with `serde_json::from_slice` or manually serializes JSON responses with `serde_json::to_vec`; framework JSON extractors/responses are used instead.
- [ ] Tests cover at least: successful `/state`, missing/invalid auth, read-token-vs-admin-token behavior, wrong client CA rejection, wrong common-name rejection, and successful API cert reload.
- [ ] `tests/bdd_api_http.rs` or equivalent integration coverage no longer depends on manually calling API `step_once` against a raw socket.
- [ ] `docs/src/reference/http-api.md` documents the final route surface including `POST /reload/certs`.
- [ ] `docs/src/reference/runtime-configuration.md`, `docs/src/how-to/configure-tls.md`, and `docs/src/how-to/configure-tls-security.md` document the new TLS/auth model, the common-name allow-list, and the API-only cert reload behavior.
- [ ] Shipped/example configs under `docker/configs/` and any repo docs snippets are updated so they do not advertise removed API TLS behavior.
- [ ] The final implementation is a real code-reduction refactor: transport/framework replacement should delete more hand-rolled code than it adds.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
