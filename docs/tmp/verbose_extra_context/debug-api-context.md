# Verbose extra context for docs/src/reference/debug-api.md

This note is intentionally exhaustive and source-first. It summarizes only what appears in the requested files and targeted route searches.

## What the debug API is

- The debug API is an optional read-only observability surface controlled by `debug.enabled` in runtime config.
- The main debug modules are `src/debug_api/snapshot.rs`, `src/debug_api/view.rs`, and `src/debug_api/worker.rs`.
- `snapshot.rs` defines the internal `SystemSnapshot` model. That snapshot captures the latest versioned state for app lifecycle, runtime config, PostgreSQL state, DCS state, process state, and HA state, plus generated time, a sequence number, and retained `changes` and `timeline` histories.
- `worker.rs` is the producer. It polls the state channels, computes compact summaries for each domain, detects meaningful changes, appends change/timeline events, trims retained history to a bounded ring, and publishes the current `SystemSnapshot`.
- `view.rs` is the JSON projection layer. It converts a `Versioned<SystemSnapshot>` into the stable verbose JSON payload used by `/debug/verbose`.

## Endpoint surface and routing

- Targeted route search in `src/api/worker.rs` shows these debug endpoints:
- `GET /debug/snapshot`
- `GET /debug/verbose`
- `GET /debug/ui`
- The same route search shows the regular HA/API endpoints that appear inside the debug payload's `api.endpoints` list:
- `/fallback/cluster`
- `/switchover`
- `/ha/state`
- `/ha/switchover`
- `view.rs` hardcodes the `api.endpoints` array in the verbose payload to:
- `/debug/snapshot`
- `/debug/verbose`
- `/debug/ui`
- `/fallback/cluster`
- `/switchover`
- `/ha/state`
- `/ha/switchover`

## Availability and auth

- `src/config/schema.rs` defines `DebugConfig { enabled: bool }`.
- The docker runtime example has `[debug] enabled = true`, so it is a real supported deployment shape, not test-only.
- Route search in `src/api/worker.rs` shows admin endpoints are only `POST /switchover`, `POST /fallback/heartbeat`, and `DELETE /ha/switchover`.
- That means the debug endpoints are read-role endpoints, not admin-role endpoints.
- Existing `docs/src/reference/http-api.md` already states that the debug endpoints are only available when `debug.enabled` is true and return `404 Not Found` when disabled.
- Existing HTTP API reference also states bearer-token auth rules and TLS behavior that apply to these endpoints too.

## Snapshot model details

- `SystemSnapshot` includes:
- `app`
- `config`
- `pg`
- `dcs`
- `process`
- `ha`
- `generated_at`
- `sequence`
- `changes`
- `timeline`
- Each subsystem state is stored as `Versioned<T>`, so the snapshot preserves both the value and the publishing metadata.
- `DebugDomain` variants are `App`, `Config`, `PgInfo`, `Dcs`, `Process`, and `Ha`.
- `DebugChangeEvent` stores `sequence`, timestamp, domain, previous version, current version, and human-readable summary.
- `DebugTimelineEntry` stores `sequence`, timestamp, domain, and message.

## History retention and incremental reads

- `src/debug_api/worker.rs` sets `DEFAULT_HISTORY_LIMIT` to `300`.
- The worker stores change and timeline events in `VecDeque`s and trims both queues back to `history_limit`.
- `/debug/verbose` supports incremental reads by sequence.
- `src/api/worker.rs` parses the `since` query parameter and passes it to `build_verbose_payload`.
- `view.rs` uses `since_sequence.unwrap_or(0)` as the cutoff and only emits `changes` and `timeline` rows whose `sequence` is greater than that cutoff.
- The verbose payload also includes:
- `debug.history_changes`
- `debug.history_timeline`
- `debug.last_sequence`
- That means clients can poll using `since=<last seen sequence>` and still understand how much retained history exists in memory.

## `/debug/verbose` JSON shape

- `DebugVerbosePayload` is the authoritative schema.
- Top-level sections are:
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
- `meta` includes:
- `schema_version` and it is currently `"v1"`
- `generated_at_ms`
- `channel_updated_at_ms`
- `channel_version`
- `app_lifecycle`
- `sequence`
- `config` includes cluster identity and two key booleans:
- `cluster_name`
- `member_id`
- `scope`
- `debug_enabled`
- `tls_enabled`
- `pginfo` normalizes PostgreSQL state into a compact projection:
- version/update metadata
- a `variant` string of `Unknown`, `Primary`, or `Replica`
- `worker`, `sql`, `readiness`
- optional `timeline`
- a compact human-readable `summary`
- `dcs` includes worker state, trust string, current member count, optional leader member id, and whether a switchover request exists.
- `process` includes worker state, whether the process worker is idle or running, the active job id when running, and the last outcome when idle.
- `ha` includes worker state, phase string, tick, decision label, optional decision detail, and the count of planned actions after lowering the decision into an effect plan.
- `api.endpoints` is a static list of surfaced routes.
- `debug` reports retained history lengths and the last sequence number.
- `changes` and `timeline` are arrays filtered by `since`.

## Change and timeline semantics

- The worker records initial baseline entries when it first observes the world.
- On later polls, it records events only when the compact signatures change.
- A targeted search shows there is a contract test specifically asserting that HA tick-only changes do not create extra debug history noise.
- That is useful operator context: `changes` is not every loop tick; it is a meaningful-change stream.

## `/debug/snapshot` behavior

- The route exists separately from `/debug/verbose`.
- The current HTTP API reference describes `/debug/snapshot` as a debug-formatted snapshot rather than the stable JSON view.
- The authoritative stable field list is therefore `/debug/verbose`, while `/debug/snapshot` is the raw snapshot-oriented diagnostic surface.
- Keep the page explicit that `/debug/verbose` is the endpoint to automate against for structured polling.

## `/debug/ui`

- Route search and nearby hits in `src/api/worker.rs` show `/debug/ui` is a built-in HTML page that fetches `/debug/verbose?since=...`, renders timeline and change tables, and updates from the same verbose payload.
- This matters for the reference page: `/debug/ui` is not a separate data schema. It is a browser-facing reader over `/debug/verbose`.

## Config and example deployment facts

- `docker/configs/cluster/node-a/runtime.toml` contains:
- `[api] listen_addr = "0.0.0.0:8080"`
- `[api.security] tls.mode = "disabled"`
- `[api.security.auth] type = "disabled"`
- `[debug] enabled = true`
- The example therefore exposes the debug routes over plain HTTP on port 8080 with no bearer token configured.
- In secured deployments, the same debug routes inherit the API listener TLS/auth posture instead of having a separate listener.
