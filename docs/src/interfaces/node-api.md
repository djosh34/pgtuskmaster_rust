# Node API

The Node API is a compact operational interface for state visibility and planned control intent.

## Main endpoints

- `GET /ha/state`: current HA-relevant state projection
- `POST /switchover`: create planned switchover intent
- `DELETE /ha/switchover`: cancel or clear pending switchover intent
- `POST /restore`: request a cluster restore takeover (admin)
- `GET /ha/restore`: restore request/status view (read)
- `DELETE /ha/restore`: clear restore request/status records (admin)

## Internal ingest endpoints

These endpoints are used by pgtuskmaster-internal helpers (not intended for operators to call directly):

- `POST /events/wal`: ingest a WAL archive/restore passthrough event emitted by `pgtuskmaster wal ...`.
  - Loopback-only enforcement: requests are rejected unless the peer IP is `127.0.0.1` / `::1`.
  - Auth role: Read (accepts `read` or `admin` token when auth is enabled).
  - Emits a structured log event with `event.name=backup.wal_passthrough` and attributes like `invocation_id`, `status_code`, WAL identifiers, and bounded stdout/stderr previews.

## Fallback endpoints

These endpoints exist for compatibility and minimal external health/identity workflows:

- `GET /fallback/cluster`: minimal cluster identity view
- `POST /fallback/heartbeat`: compatibility heartbeat (admin endpoint)

## Optional debug endpoints

When debug support is enabled in runtime configuration:

- `GET /debug/ui`
- `GET /debug/verbose` (optionally accepts `?since=<sequence>` to filter)
- `GET /debug/snapshot`

The debug verbose payload includes a bounded timeline/change stream that is intentionally *semantic*: it does not emit “tick-only churn” entries when the underlying state has not meaningfully changed.

## Why this exists

The API is intentionally small to keep operational behavior explicit. It is designed around intent and state, not low-level procedure endpoints.

## Tradeoffs

A narrow API surface means fewer ad-hoc knobs. The benefit is clearer lifecycle behavior and smaller control-plane risk.

## When this matters in operations

For planned role changes, use API intent workflows instead of direct out-of-band coordination writes.

## Restore takeover

### `POST /restore` (admin)

Requests a single-flight restore takeover for the cluster.

- Auth role: Admin
- Body (JSON):
  - `requested_by` (string, required)
  - `executor_member_id` (string, required): the member id that must execute the restore sequence
  - `reason` (string, optional)
  - `idempotency_token` (string, optional)

Response: `202 Accepted` with `{ "accepted": true, "restore_id": "<server generated>" }`.

If a restore request already exists, the endpoint returns `409 Conflict`.

### `GET /ha/restore` (read)

Returns a stable restore payload for polling:

- `request`: restore request record (or `null`)
- `status`: restore status record (or `null`)
- `derived`: small derived hints (e.g. `is_executor`, `heartbeat_stale`)

### `DELETE /ha/restore` (admin)

Deletes both the restore request and restore status keys in DCS.

Note: clearing intent does not forcibly terminate an in-flight `pgbackrest restore` job if it is already running on the executor.
