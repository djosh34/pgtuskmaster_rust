# HTTP API Reference

The node API now exposes one read surface and one control noun:

- `GET /state`
- `POST /switchover`
- `DELETE /switchover`

This is the entire public route surface.

## Base URL

```text
http(s)://<listen_addr>
```

The exact scheme depends on the node's TLS configuration.

## Authentication

When API tokens are configured, every request must send:

```text
Authorization: Bearer <token>
```

Read operations accept a read token or an admin token. Switchover operations require an admin token.

Authorization outcomes:

- missing token: `401 Unauthorized`
- insufficient role: `403 Forbidden`
- valid token: route-specific success or validation status

## `GET /state`

Returns one serializable `NodeState` document built from the node's current runtime state plus the public read-only `DcsView` snapshot that the local DCS worker publishes.

Authorization: read

Success status: `200 OK`

Unavailable status: `503 Service Unavailable`
The node has not finished wiring the required live state subscribers yet.

### Response shape

```text
{
  "cluster_name": "cluster-a",
  "scope": "cluster-a",
  "self_member_id": "node-a",
  "pg": {},
  "process": {},
  "dcs": {},
  "ha": {}
}
```

Top-level fields:

- `cluster_name`: configured cluster name
- `scope`: configured DCS scope
- `self_member_id`: local member identifier
- `pg`: current local PostgreSQL observation
- `process`: current local process-worker state
- `dcs`: current DCS trust plus the published member, leader, and switchover view
- `ha`: current HA publication, target role, worldview, and planned commands

### What `GET /state` is for

`GET /state` is intentionally verbose. It is the single raw observation surface for operators, tests, and `pgtm`.

The payload includes both facts and interpretation:

- raw local PostgreSQL state in `pg`
- raw DCS-backed cluster state in `dcs`
- the HA engine's derived understanding in `ha.world`
- the operator-facing authority projection in `ha.publication`

`pgtm status`, `pgtm primary`, and `pgtm replicas` all start from this single document. They do not fan out to peer APIs.

## `POST /switchover`

Requests a planned switchover through the node's typed DCS command surface.

Authorization: admin

Request body:

```text
{
  "switchover_to": "node-b"
}
```

`switchover_to` is optional. When omitted, the API records a generic switchover request and the HA loop chooses the successor from the observed DCS view.

Success status: `202 Accepted`

```text
{
  "accepted": true
}
```

Validation and failure statuses:

- `400 Bad Request`: malformed JSON, unknown target, self-target, degraded trust, request sent to a non-authoritative node, or ineligible target
- `401 Unauthorized`: missing or invalid token
- `403 Forbidden`: read token used for an admin route
- `503 Service Unavailable`: DCS command failed

## `DELETE /switchover`

Clears any pending switchover request through the node's typed DCS command surface.

Authorization: admin

Success status: `202 Accepted`

```text
{
  "accepted": true
}
```

Failure statuses:

- `401 Unauthorized`: missing or invalid token
- `403 Forbidden`: read token used for an admin route
- `503 Service Unavailable`: DCS command failed
