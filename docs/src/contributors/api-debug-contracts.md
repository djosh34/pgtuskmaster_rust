# API and Debug Contracts

This chapter explains the “edges” of the system:

- the **Node API** (operator-facing HTTP surface), and
- the **debug snapshot** and debug views (contributor-facing observability surface).

As a contributor, you should treat these as explicit contracts:

- what external clients are allowed to do (write intents, read state)
- what internal state is exposed and how it is projected
- what is intentionally “best effort” (debug-only) vs stable.

## The Node API worker: where requests land

The server loop lives in `src/api/worker.rs`.

The runtime binds the listener during startup wiring (`runtime/node.rs`), then runs the API worker as one of the steady-state tasks.

The API worker is intentionally minimal:

- accept a TCP connection
- optionally negotiate TLS
- parse one HTTP request
- authorize it (if auth is configured)
- route it to a handler
- write an HTTP response.

This is not a general HTTP framework; it is a small purpose-built protocol implementation.

### TLS modes and client certificates

TLS behavior is configured in `RuntimeConfig` and interpreted by the API worker.

The current modes:

- **Disabled**: plaintext only.
- **Required**: the worker only accepts TLS connections.
- **Optional**: the worker peeks the first byte and treats the connection as TLS if it “looks like” a TLS ClientHello.

Client certificate requirements are separate from “TLS required”:

- when `require_client_cert` is set, the API worker rejects TLS connections that did not present a peer certificate.

The key contributor takeaway: “Optional TLS” is a compatibility feature, but it also means clients need to be explicit about whether they expect encryption.

### Authorization model (read vs admin)

Authorization happens inside the API worker, before routing.

If auth is configured as role tokens, requests are classified into:

- **Read endpoints**: require either a read token or an admin token.
- **Admin endpoints**: require the admin token; presenting only the read token returns **403 Forbidden** (not 401).

This “Forbidden vs Unauthorized” distinction is intentional: it allows clients to detect “you authenticated but lack privileges”.

Admin endpoints in the current routing table include:

- `POST /switchover`
- `DELETE /ha/switchover`
- `POST /fallback/heartbeat`.

Everything else is treated as read-only.

## Read contract: how state becomes API output

The Node API does not directly stitch together `pginfo`, `dcs`, `ha`, and `process` channels.

Instead, the runtime wires a `StateSubscriber<SystemSnapshot>` into the API worker. That snapshot is produced by the debug snapshot worker in `src/debug_api/worker.rs` and is built by `debug_api::snapshot::build_snapshot(...)`.

That design choice matters:

- there is a single owned “projection” for “what the system believes right now”
- debug views and operator reads can share the same composed snapshot
- contributors can evolve internal state shapes without making every HTTP handler understand every worker channel.

### `/ha/state`: stable-ish operational read

`GET /ha/state` returns a small `HaStateResponse` derived from the snapshot:

- cluster identity (cluster name, scope, self member id)
- current leader and switchover request (if any)
- DCS trust label
- HA phase and tick
- pending action count
- the snapshot sequence (useful for polling clients).

This endpoint is the best “simple operational signal” to rely on. It should remain small and understandable even as internal debug views become more detailed.

## Write contract: intent writes into DCS (switchover)

The most important operator write path today is the switchover intent.

### `POST /switchover` (intent write)

The handler is `api/controller.rs::post_switchover(...)`.

It:

- validates `requested_by` is non-empty
- encodes a `SwitchoverRequest` record as JSON
- writes it to the scoped key `/{scope}/switchover` via the DCS store.

This is an **intent** write, not an immediate execution command.

### How intent becomes action

The write path is:

1. API writes `/{scope}/switchover`
2. DCS worker observes the key via its watch stream and publishes it in `DcsCache`
3. HA consumes `DcsCache` and, if it is currently primary, transitions out of primary and emits:
   - demote
   - release leader lease
   - clear switchover key.

That end-to-end path is the pattern to follow for new “operator intents”:

- write a typed intent record to a stable key
- have the DCS worker own the read model (decoding and caching)
- have HA own the decision of “what to do about it”.

## Debug contract: snapshots and verbose projections

Debug endpoints are gated by config (`cfg.debug.enabled`).

The debug surfaces exist for contributors and test fixtures. They are allowed to change more than the operator-facing `/ha/state` contract, as long as the behavior is well described and tested.

### `GET /debug/snapshot`

Returns a pretty-printed `SystemSnapshot` debug representation.

This is primarily useful for humans, not for stable automation.

### `GET /debug/verbose`

Returns a structured JSON payload built by `src/debug_api/view.rs`:

- a “meta” section (schema version, timestamps, channel version, snapshot sequence)
- one section per subsystem (config, pginfo, dcs, process, ha)
- a bounded history of changes and a timeline stream.

The `since=<sequence>` query parameter allows clients (including the built-in debug UI) to poll incrementally.

Important semantic rule (to avoid “timeline noise”):

- timeline/change entries are recorded on *semantic* diffs, not just on monotonic `Version` churn
- for HA specifically, tick-only changes do not generate new timeline entries (the tick is still part of the snapshot, but it is not used as a “did something change?” detector)

### `GET /debug/ui`

Serves a small HTML/JS page that polls `/debug/verbose` and renders:

- current state sections
- timeline entries
- change events.

This is intentionally crude but extremely useful during e2e debugging.

## Client contract (what to rely on)

When you extend the API/debug surface, be explicit about what clients can rely on.

In the current codebase, a practical rule is:

- `/ha/state` is the stable operational contract; keep it simple and conservative.
- `/switchover` (and related admin endpoints) are stable “intent write” contracts; validate inputs and keep the DCS key/value formats deliberate.
- `/debug/*` endpoints are contributor/debug contracts:
  - allowed to evolve, but changes should be reflected in docs and test fixtures that consume them.

## Adjacent subsystem connections

This chapter is about the system’s edges. It connects directly to:

- [Worker Wiring and State Flow](./worker-wiring.md): explains how the API reads state (debug snapshot subscriber) and how that snapshot is composed.
- [HA Decision and Action Pipeline](./ha-pipeline.md): explains how the switchover intent turns into demote/release/clear actions.
- [DCS Data Model and Write Paths](../assurance/dcs-data-model.md): operator-facing explanation of the keys and semantics; contributor docs should stay aligned with it.
