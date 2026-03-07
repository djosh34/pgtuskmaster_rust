# Node API

The node API is the runtime's operational control plane. It is intentionally small: one state endpoint, one planned switchover write path, one switchover clear path, a narrow fallback surface, and optional debug routes.

The default listen address is `127.0.0.1:8080`. In a secure deployment, bind it to the interface you actually need, enable TLS, and require role tokens rather than exposing an unauthenticated HTTP listener.

## Core routes

These are the routes the operator workflow depends on today:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/ha/state` | Read the current HA state projection | Read or admin |
| `POST` | `/switchover` | Submit a planned switchover request | Admin |
| `DELETE` | `/ha/switchover` | Clear a pending switchover request | Admin |

`GET /ha/state` stays useful even during degraded coordination windows. The implementation keeps it available so operators can still see phases such as fail-safe without depending on DCS cleanup succeeding first.

## Reading HA state

`GET /ha/state` returns a typed JSON snapshot of the current HA-relevant view:

- cluster name, DCS scope, and this node's member ID
- current leader, if known
- `switchover_requested_by`, if one is queued
- member count
- DCS trust state
- HA phase and HA decision
- snapshot sequence number

Example:

```console
curl \
  --silent \
  --show-error \
  --header "Authorization: Bearer $PGTUSKMASTER_READ_TOKEN" \
  https://node-a.example.internal:8443/ha/state
```

If API auth is disabled, the bearer token is not required. That may be convenient for a local lab, but it should not be your production posture.

## Submitting a planned switchover

To ask the cluster to begin a planned switchover, send a JSON body with `requested_by`:

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

Validation is strict:

- the body must parse as JSON
- unknown fields are rejected
- `requested_by` must be non-empty after trimming

On success the server returns `202 Accepted` with:

```text
{
  "accepted": true
}
```

That response confirms the intent was written. It does not mean the switchover has already completed, and `requested_by` is only an audit field in the current implementation rather than a target-member selector.

## Clearing a pending switchover

To cancel or clear a queued switchover intent:

```console
curl \
  --silent \
  --show-error \
  --request DELETE \
  --header "Authorization: Bearer $PGTUSKMASTER_ADMIN_TOKEN" \
  https://node-a.example.internal:8443/ha/switchover
```

The server also returns `202 Accepted` when the request is recorded successfully.

## Fallback routes

The runtime also exposes a minimal compatibility surface:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/fallback/cluster` | Minimal cluster identity view | Read or admin |
| `POST` | `/fallback/heartbeat` | Compatibility heartbeat endpoint | Admin |

These are intentionally narrow. They are not a second management API.

## Optional debug routes

When `debug.enabled = true`, the API enables three extra routes:

| Method | Path | Purpose | Required role |
| --- | --- | --- | --- |
| `GET` | `/debug/ui` | Browser UI for the verbose stream | Read or admin |
| `GET` | `/debug/verbose` | Structured debug payload, optionally filtered by `?since=<sequence>` | Read or admin |
| `GET` | `/debug/snapshot` | Snapshot dump kept for compatibility | Read or admin |

When debug is disabled, these routes return `404 Not Found`. When the runtime lacks the necessary snapshot subscriber, the read routes return `503 Service Unavailable`.

## TLS and auth model

The API security block is explicit in `config_version = "v2"`:

- `api.security.tls.mode` controls whether the server runs without TLS, with optional TLS, or with required TLS
- `api.security.tls.identity` is mandatory whenever TLS mode is `optional` or `required`
- `api.security.auth` is either `disabled` or `role_tokens`
- `role_tokens` may define a read token, an admin token, or both, but at least one token must be present

For operator-facing environments, prefer:

- `api.security.tls.mode = "required"`
- a configured server certificate and private key
- `api.security.auth.type = "role_tokens"`
- separate read and admin tokens stored outside the config file when possible

That keeps both state reads and write operations behind explicit credentials.
