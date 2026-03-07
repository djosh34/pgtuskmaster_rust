# API and Debug Contracts

This chapter explains the edges of the system:

- the Node API, which accepts operator intent and serves stable operational reads
- the debug snapshot pipeline, which produces contributor-facing read models and history

These surfaces are contracts. They decide what clients may write, what they may rely on when reading state, and which internal projections must remain coherent even while the implementation behind them changes.

## Where to start in code

The fastest entrypoints are all in `src/api/worker.rs`:

- `run(...)`: outer loop and fatal-vs-nonfatal handling
- `step_once(...)`: timed accept, one request cycle, response logging
- `authorize_request(...)`: role split and `401` vs `403`
- `route_request(...)`: endpoint ownership

For the projection side, open:

- `src/api/controller.rs`: stable operational read models and switchover intent writes
- `src/debug_api/worker.rs`: snapshot production and semantic change tracking
- `src/debug_api/snapshot.rs`: the raw composed `SystemSnapshot`
- `src/debug_api/view.rs`: `/debug/verbose` JSON projection

The API worker is intentionally hand-rolled. When you change TLS, auth, timeouts, routing, or error handling, you are editing explicit code paths rather than middleware defaults from a framework.

## Runtime wiring

`src/runtime/node.rs::run_workers(...)` binds the API listener, creates a dedicated DCS store handle for API writes, and wires in a `StateSubscriber<SystemSnapshot>` for read endpoints.

That means:

- intent writes go through the API worker's own DCS handle
- reads come from the composed debug snapshot, not from raw worker channels
- API handlers should stay small because the projection logic already lives elsewhere

## TLS and connection handling

TLS behavior is configured in `RuntimeConfig` and interpreted in `src/api/worker.rs`.

The current modes are:

- `Disabled`: plaintext only
- `Required`: the worker must negotiate TLS
- `Optional`: the worker peeks for a TLS client hello and decides per connection

Client-certificate requirements are a separate decision layered on top of TLS mode. If `require_client_cert` is enabled, a TLS connection without a peer certificate is rejected.

Because the worker is not using a framework, timeout behavior is also part of the contract:

- `step_once(...)` uses a short timed `accept()`
- request parsing is separately time-bounded
- nonfatal per-request failures are logged and the loop keeps serving future requests

## Authorization model

Authorization happens before routing.

If auth is configured as role tokens, requests are classified into read or admin endpoints:

- admin endpoints currently are `POST /switchover`, `DELETE /ha/switchover`, and `POST /fallback/heartbeat`
- read endpoints currently are `GET /ha/state`, `GET /fallback/cluster`, and the `GET /debug/*` routes

The exact split lives in `endpoint_role(...)`.

The `401` vs `403` distinction is intentional:

- `401 Unauthorized` means no acceptable token was presented
- `403 Forbidden` means the caller authenticated but lacks the required role

Treat that distinction as part of the client contract, not as an incidental status-code choice.

## Stable read contract: `/ha/state`

`GET /ha/state` is the small operational read model. It is projected in `src/api/controller.rs::get_ha_state(...)` from the composed `SystemSnapshot`.

The response includes:

- cluster identity
- current leader and switchover request, if any
- DCS trust
- HA phase and tick
- HA decision
- snapshot sequence

This endpoint should stay small and conservative. If you are tempted to add raw worker detail here, that field probably belongs in `/debug/verbose` instead.

## Intent write contract: switchover

The main operator write path is `POST /switchover`.

`src/api/controller.rs::post_switchover(...)`:

- validates `requested_by`
- encodes a typed `SwitchoverRequest`
- writes it to `/{scope}/switchover`

This is an intent write, not an immediate execution command.

The matching cleanup path is `DELETE /ha/switchover`, which clears the same key through `DcsHaWriter::clear_switchover(...)`.

The end-to-end control flow is:

1. API writes or clears the intent in DCS
2. DCS worker observes and republishes the request in `DcsCache`
3. HA reacts through the cached read model

That pattern is the safe way to add future operator intents. Do not make handlers reach into HA directly.

## Debug snapshot contract

Debug routes are gated by `cfg.debug.enabled` and are intended for contributors, tests, and debugging tools rather than for a minimal operator contract.

### `/debug/snapshot`

Returns a pretty-printed `SystemSnapshot` for humans.

### `/debug/verbose`

Returns JSON from `src/debug_api/view.rs::build_verbose_payload(...)` with:

- meta information such as schema version, generated timestamp, channel version, and sequence
- one section each for config, pginfo, dcs, process, and ha
- a small static API section listing exported endpoints
- bounded `changes` and `timeline` arrays

The `since=<sequence>` query parameter filters the returned history incrementally.

`src/debug_api/worker.rs` only records history on semantic diffs. For HA specifically, a tick-only change does not create new timeline noise even though the latest snapshot still contains the new tick value.

### `/debug/ui`

Serves the lightweight built-in debug UI that polls `/debug/verbose`. It is intentionally simple, but it is part of the contributor debugging workflow and should not drift from the JSON payload it expects.

## How to change this area safely

- Add or change endpoint ownership in `route_request(...)` and `endpoint_role(...)` together.
- Keep stable reads in controllers and keep debug projections in `src/debug_api/view.rs`; do not rebuild raw worker state inside handlers.
- Treat `401` vs `403` behavior as part of the client contract.
- Update `tests/bdd_api_http.rs` whenever you change a route, auth rule, or response shape.
- Decide explicitly whether a new field belongs in the stable `/ha/state` projection or only in `/debug/verbose`.

## Adjacent subsystem connections

- Read [Worker Wiring and State Flow](./worker-wiring.md) for how the debug snapshot subscriber reaches the API worker.
- Read [HA Decision and Action Pipeline](./ha-pipeline.md) for how switchover intent turns into demote, lease-release, and clear-request effects.
- Read [Testing System Deep Dive](./testing-system.md) for which tests prove stable HTTP contracts versus debug-only projections.

## Evidence pointers

- `src/api/worker.rs`
- `src/api/controller.rs`
- `src/api/fallback.rs`
- `src/debug_api/worker.rs`
- `src/debug_api/snapshot.rs`
- `src/debug_api/view.rs`
- `tests/bdd_api_http.rs`
