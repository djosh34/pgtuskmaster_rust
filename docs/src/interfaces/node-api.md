# Node API

The Node API is a compact operational interface for state visibility and planned control intent.

## Main endpoints

- `GET /ha/state`: current HA-relevant state projection, including phase, tick, and decision label/detail
- `POST /switchover`: create planned switchover intent
- `DELETE /ha/switchover`: cancel or clear pending switchover intent

Recovery bootstrap is not exposed as a mutable node-API control plane; restore/bootstrap remains internal startup behavior and operator runbook territory.

## Internal ingest endpoints

These endpoints are used by pgtuskmaster-internal helpers (not intended for operators to call directly):

- `POST /events/wal`: ingest a WAL archive-and-recovery passthrough event emitted by `pgtuskmaster wal ...`.
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
