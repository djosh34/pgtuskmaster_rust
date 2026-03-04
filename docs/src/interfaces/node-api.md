# Node API

The Node API is a compact operational interface for state visibility and planned control intent.

## Main endpoints

- `GET /ha/state`: current HA-relevant state projection
- `POST /switchover`: create planned switchover intent
- `DELETE /ha/switchover`: cancel or clear pending switchover intent

## Fallback endpoints

These endpoints exist for compatibility and minimal external health/identity workflows:

- `GET /fallback/cluster`: minimal cluster identity view
- `POST /fallback/heartbeat`: compatibility heartbeat (admin endpoint)

## Optional debug endpoints

When debug support is enabled in runtime configuration:

- `GET /debug/ui`
- `GET /debug/verbose` (optionally accepts `?since=<sequence>` to filter)
- `GET /debug/snapshot`

## Why this exists

The API is intentionally small to keep operational behavior explicit. It is designed around intent and state, not low-level procedure endpoints.

## Tradeoffs

A narrow API surface means fewer ad-hoc knobs. The benefit is clearer lifecycle behavior and smaller control-plane risk.

## When this matters in operations

For planned role changes, use API intent workflows instead of direct out-of-band coordination writes.
