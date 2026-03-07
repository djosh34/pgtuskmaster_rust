# Node API

The node API is the runtime's operational control plane. It is intentionally small: one state endpoint, one planned switchover write path, one switchover clear path, a narrow fallback surface, and optional debug routes. That small surface is deliberate. The API is meant to expose the current operator contract without turning the node into a second orchestration system.

The default listen address is `127.0.0.1:8080`. In a hardened deployment, bind it only where you need it, enable TLS, and require role tokens rather than exposing an open HTTP listener.

## Core routes

These are the routes real operator workflows depend on today:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/ha/state` | Read the current HA state projection | Read or admin |
| `POST` | `/switchover` | Submit planned switchover intent | Admin |
| `DELETE` | `/ha/switchover` | Clear a pending switchover request | Admin |

Two compatibility routes also exist:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/fallback/cluster` | Minimal cluster identity view | Read or admin |
| `POST` | `/fallback/heartbeat` | Compatibility heartbeat endpoint | Admin |

Treat the fallback routes as intentionally narrow. They are not a parallel management API and they do not carry the same operational detail as `/ha/state`.

## Reading HA state

`GET /ha/state` returns a typed JSON snapshot of the current HA-relevant view:

- cluster name, DCS scope, and this node's member ID
- current leader, if one is known
- `switchover_requested_by`, if intent is queued
- member count
- DCS trust state
- HA phase and HA decision
- HA tick and snapshot sequence number

Example:

```console
curl \
  --silent \
  --show-error \
  --header "Authorization: Bearer $PGTUSKMASTER_READ_TOKEN" \
  https://node-a.example.internal:8443/ha/state
```

The most important operational distinction is that `/ha/state` reports the runtime's current projection, not a promise that external conditions are already stable. For example:

- `ha_phase = "Primary"` means this node currently believes it owns or should own the primary posture
- `ha_decision = "attempt_leadership"` means the loop is still working through leadership acquisition rather than having finished every consequence already
- `switchover_requested_by` means intent exists, not that the switchover is complete

`GET /ha/state` is intentionally useful during degraded coordination windows. Tests in the repo explicitly require fail-safe observability through this route, so an operator should still be able to read the node's conservative phase even when DCS cleanup or lease management is slow.

## Status codes you should interpret carefully

Common responses have meaning:

- `200 OK`: the node has a usable snapshot and returned typed JSON
- `401 Unauthorized`: the route requires a role token and none or the wrong role was supplied
- `403 Forbidden`: the token is valid for a weaker role than the route requires
- `503 Service Unavailable`: the route exists, but the node cannot provide the underlying snapshot or a DCS-backed operation failed

That difference matters in incident response. `401` and `403` are security posture mismatches. `503` is a runtime or dependency problem. They lead to different fixes.

## Submitting a planned switchover

To ask the cluster to begin a planned switchover, send JSON with `requested_by`:

```console
curl \
  --silent \
  --show-error \
  --request POST \
  --header "Authorization: Bearer $PGTUSKMASTER_ADMIN_TOKEN" \
  --header "Content-Type: application/json" \
  --data '{"requested_by":"node-b"}' \
  https://node-a.example.internal:8443/switchover
```

Validation is intentionally strict:

- the body must parse as JSON
- unknown fields are rejected
- `requested_by` must be non-empty after trimming

On success the server returns `202 Accepted` with:

```text
{
  "accepted": true
}
```

Interpret `202` correctly. It confirms the intent was recorded in the DCS path for the current scope. It does not mean the cluster already demoted the current primary, selected a successor, or completed promotion. The current implementation also treats `requested_by` as audit metadata, not as an explicit successor selector, so the actual next primary still depends on what the HA loop sees when it reevaluates the cluster.

## Clearing a pending switchover

To cancel or clear queued switchover intent:

```console
curl \
  --silent \
  --show-error \
  --request DELETE \
  --header "Authorization: Bearer $PGTUSKMASTER_ADMIN_TOKEN" \
  https://node-a.example.internal:8443/ha/switchover
```

This also returns `202 Accepted` when the delete was recorded successfully. As with creation, accepted does not mean every node has already observed the new DCS state. It means the requested mutation succeeded on the API side and the HA loop will react on its next observations.

## Optional debug routes

When `debug.enabled = true`, the API enables three extra routes:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/debug/ui` | Browser UI for the verbose stream | Read or admin |
| `GET` | `/debug/verbose` | Structured debug payload, optionally filtered by `?since=<sequence>` | Read or admin |
| `GET` | `/debug/snapshot` | Snapshot dump kept for compatibility | Read or admin |

Operationally:

- `404 Not Found` means debug routes are disabled in config
- `503 Service Unavailable` means the route exists but the snapshot subscriber is unavailable
- `/debug/verbose` is the preferred machine-readable debug surface
- `/debug/ui` is just a thin browser view over the verbose feed
- `/debug/snapshot` remains for compatibility and should not be treated as the preferred day-2 interface

In the repo-owned quick-start configs, debug is intentionally enabled, so a `404` there usually means you are not running the config you thought you were.

## TLS and auth model

The API security model is explicit in `config_version = "v2"`:

- `api.security.tls.mode` controls whether the server runs without TLS, with optional TLS, or with required TLS
- a server identity is mandatory whenever TLS mode is `optional` or `required`
- `api.security.auth` is either `disabled` or `role_tokens`
- `role_tokens` may define a read token, an admin token, or both, but at least one token must be present when that mode is enabled

The role split is simple:

- read routes accept a read token or an admin token
- write routes require an admin token
- when auth is disabled, the routes are open by configuration rather than by accident

For operator-facing environments, prefer:

- `api.security.tls.mode = "required"`
- a configured server certificate and private key
- `api.security.auth.type = "role_tokens"`
- separate read and admin tokens managed outside the committed config files

That keeps both observation and control behind explicit credentials. If you also require client certificates, interpret TLS handshake failures separately from application-level authorization failures, because they fail before the route logic ever runs.

## How operators should use the API during incidents

Use the API in this order:

1. read `/ha/state` first to anchor the current phase, trust level, and pending intent
2. use `/debug/verbose` only when you need the richer timeline and debug routes are intentionally enabled
3. submit or clear switchover intent after you have verified (via `/ha/state`) that the cluster state is coherent enough for a planned transition

That order keeps the API aligned with its design. It is primarily an observation surface plus a narrow intent surface. The lifecycle and troubleshooting chapters explain whether the result you see is healthy, conservative, blocked, or genuinely broken.
