# HTTP API Reference

The PGTuskmaster HTTP API provides programmatic access to cluster state, high-availability operations, and diagnostic information. The API server listens on a TCP address configured in the runtime configuration.

## Base URL

```text
http(s)://<listen_addr>
```

The protocol depends on TLS configuration.

## Authentication and Authorization

### Bearer Token Authentication

When role tokens are configured, all requests must include an `Authorization` header with a Bearer token.

```text
Authorization: Bearer <token>
```

### Role-Based Access Control

The API distinguishes two authorization levels:

- **Read**: Grants access to state observation endpoints
- **Admin**: Grants access to control-plane endpoints

If no role tokens are configured, all requests are allowed.

#### Token Configuration
- **read_token**: Grants read access
- **admin_token**: Grants admin access (also grants read access)

#### Endpoint Roles

**Admin Endpoints**
- `POST /switchover`
- `POST /fallback/heartbeat`
- `DELETE /ha/switchover`

**Read Endpoints**
- All other endpoints listed in this reference

#### Authorization Outcomes

- Missing token when tokens are configured: `401 Unauthorized`
- Read token accessing admin endpoint: `403 Forbidden`
- Valid token for required role: `200 OK` or other appropriate status

### TLS Security

TLS mode is configured independently of authorization:

- **Disabled**: Plain HTTP only
- **Optional**: Accepts plain HTTP or HTTPS
- **Required**: HTTPS only

Optional client certificate verification can be enforced when TLS is enabled.

---

## Endpoints

### High Availability Control

#### Request Switchover

```text
POST /switchover
```

Initiates a planned leader switchover.

**Authorization**: Admin

**Request Body** (`SwitchoverRequestInput`)
```text
{
  "switchover_to": "<string>" | null
}
```

The request body may be empty for a generic switchover, or it may set `switchover_to` to request a specific eligible replica. Unknown fields are rejected.

**Response** (`AcceptedResponse`)
```text
{
  "accepted": true
}
```

**Status Codes**
- `202 Accepted`: Switchover request accepted and written to DCS
- `400 Bad Request`: Invalid JSON, unknown request fields, empty target, unknown target member, or ineligible target member
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: Read token used for admin endpoint
- `503 Service Unavailable`: DCS store error, or targeted validation could not load the current cluster snapshot

#### Cancel Switchover Request

```text
DELETE /ha/switchover
```

Removes a pending switchover request from DCS.

**Authorization**: Admin

**Response** (`AcceptedResponse`)
```text
{
  "accepted": true
}
```

**Status Codes**
- `202 Accepted`: Switchover request cleared
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: Read token used for admin endpoint
- `503 Service Unavailable`: DCS store error

#### Get HA State

```text
GET /ha/state
```

Retrieves current high-availability state.

**Authorization**: Read

This is the stable read surface that `pgtm status` uses for cluster discovery. The response now includes a stable member list with advertised peer API URLs so operators and automation can build a cluster-wide view without scraping debug endpoints or manually looping over nodes.

**Response** (`HaStateResponse`)
```text
{
  "cluster_name": "<string>",
  "scope": "<string>",
  "self_member_id": "<string>",
  "leader": "<string>" | null,
  "switchover_pending": <bool>,
  "switchover_to": "<string>" | null,
  "member_count": <number>,
  "members": [
    {
      "member_id": "<string>",
      "postgres_host": "<string>",
      "postgres_port": <number>,
      "api_url": "<string>" | null,
      "role": "<member_role_variant>",
      "sql": "<sql_status_variant>",
      "readiness": "<readiness_variant>",
      "timeline": <number> | null,
      "write_lsn": <number> | null,
      "replay_lsn": <number> | null,
      "updated_at_ms": <number>,
      "pg_version": <number>
    }
  ],
  "dcs_trust": "<trust_variant>",
  "authority": "<authority_variant>",
  "fence_cutoff": "<fence_cutoff_variant>" | null,
  "ha_role": "<role_variant>",
  "ha_tick": <number>,
  "planned_actions": ["<reconcile_action_variant>", "..."],
  "snapshot_sequence": <number>
}
```

**Field Details**
- `cluster_name`: Configured cluster name
- `scope`: DCS scope for this cluster
- `self_member_id`: Local member identifier
- `leader`: Current leader member ID if one exists
- `switchover_pending`: Whether a switchover request is currently pending
- `switchover_to`: Requested switchover target when the pending request is explicit, otherwise `null`
- `member_count`: Number of members in DCS cache
- `members`: Stable cluster member discovery data. This is the machine-readable list that `pgtm status` fans out from when it builds a cluster-wide view.
- `dcs_trust`: Trust level of DCS (see DcsTrustResponse variants)
- `authority`: Current operator-facing authority projection (see [HA State Semantics](ha-decisions.md))
- `fence_cutoff`: Present when the node is publishing a no-primary safety boundary that includes a lease epoch and committed LSN cutoff
- `ha_role`: Current local HA role intent (see [HA State Semantics](ha-decisions.md))
- `ha_tick`: HA decision loop counter
- `planned_actions`: Ordered reconcile actions the node plans to execute next
- `snapshot_sequence`: Monotonic snapshot version

Each `members[]` entry includes:

- `member_id`: DCS member identifier
- `postgres_host` and `postgres_port`: PostgreSQL endpoint currently published for that member
- `api_url`: Operator-reachable node API URL published by that member, or `null` when none is available
- `role`: Current DCS member role (`unknown`, `primary`, or `replica`)
- `sql`: Current SQL reachability (`unknown`, `healthy`, or `unreachable`)
- `readiness`: Current readiness state (`unknown`, `ready`, or `not_ready`)
- `timeline`: Current PostgreSQL timeline when known
- `write_lsn`: Current write LSN for a primary when known
- `replay_lsn`: Current replay LSN for a replica when known
- `updated_at_ms`: Timestamp of the published member record
- `pg_version`: Published PostgreSQL-state version for that member

When `switchover_to` is omitted, successor choice remains automatic. `POST /switchover` signals intent to switch over, and the HA engine picks the replacement primary from observed cluster state. When `switchover_to` is present, the API accepts only a known, fresh, healthy, ready replica and the HA engine holds non-target nodes back from acquiring leadership during that switchover.

**Status Codes**
- `200 OK`: State retrieved successfully
- `401 Unauthorized`: Missing or invalid token
- `503 Service Unavailable`: Snapshot subscriber unavailable

---

### Fallback Cluster

#### Get Fallback Cluster Name

```text
GET /fallback/cluster
```

Retrieves the configured fallback cluster name.

**Authorization**: Read

**Response**
```text
{
  "name": "<cluster_name>"
}
```

**Status Codes**
- `200 OK`: Cluster name returned
- `401 Unauthorized`: Missing or invalid token

#### Send Fallback Heartbeat

```text
POST /fallback/heartbeat
```

Records a heartbeat from a fallback cluster member.

**Authorization**: Admin

**Request Body** (`FallbackHeartbeatInput`)
```text
{
  "source": "<member_id>"
}
```

- `source`: Non-empty string identifying the heartbeat source

**Response** (`AcceptedResponse`)
```text
{
  "accepted": true
}
```

**Status Codes**
- `202 Accepted`: Heartbeat accepted
- `400 Bad Request`: Invalid JSON or empty `source` field
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: Read token used for admin endpoint

---

### Debug and Diagnostics

#### Get System Snapshot (Debug)

```text
GET /debug/snapshot
```

Returns a debug-formatted snapshot of all system state.

**Authorization**: Read

**Availability**: Only when `debug.enabled` is true in runtime configuration

**Response**: Plain text debug dump (not stable JSON)

**Status Codes**
- `200 OK`: Snapshot returned
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Debug endpoints disabled
- `503 Service Unavailable`: Snapshot subscriber unavailable

#### Get Verbose Debug Data

```text
GET /debug/verbose[?since=<sequence>]
```

Returns structured system state and recent changes.

**Authorization**: Read

**Availability**: Only when `debug.enabled` is true in runtime configuration

**Query Parameters**
- `since` (optional): Sequence number to filter changes/timeline events

**Response** (`DebugVerbosePayload`)
```text
{
  "meta": {
    "schema_version": "v1",
    "generated_at_ms": <number>,
    "channel_updated_at_ms": <number>,
    "channel_version": <number>,
    "app_lifecycle": "<string>",
    "sequence": <number>
  },
  "config": {
    "version": <number>,
    "updated_at_ms": <number>,
    "cluster_name": "<string>",
    "member_id": "<string>",
    "scope": "<string>",
    "debug_enabled": <boolean>,
    "tls_enabled": <boolean>
  },
  "pginfo": { "..." : "see field list below" },
  "dcs": { "..." : "see field list below" },
  "process": { "..." : "see field list below" },
  "ha": { "..." : "see field list below" },
  "api": {
    "endpoints": [
      "/debug/snapshot",
      "/debug/verbose",
      "/debug/ui",
      "/fallback/cluster",
      "/switchover",
      "/ha/state",
      "/ha/switchover"
    ]
  },
  "debug": {
    "history_changes": <number>,
    "history_timeline": <number>,
    "last_sequence": <number>
  },
  "changes": [],
  "timeline": []
}
```

The structured sections are:

- `pginfo`: `version`, `updated_at_ms`, `variant`, `worker`, `sql`, `readiness`, `timeline`, `summary`
- `dcs`: `version`, `updated_at_ms`, `worker`, `trust`, `member_count`, `leader`, `has_switchover_request`
- `process`: `version`, `updated_at_ms`, `worker`, `state`, `running_job_id`, `last_outcome`
- `ha`: `version`, `updated_at_ms`, `worker`, `phase`, `tick`, `decision`, `decision_detail`, `planned_actions`
- `changes[]`: `sequence`, `at_ms`, `domain`, `previous_version`, `current_version`, `summary`
- `timeline[]`: `sequence`, `at_ms`, `category`, `message`

**Status Codes**
- `200 OK`: Verbose data returned
- `400 Bad Request`: Invalid `since` parameter
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Debug endpoints disabled
- `503 Service Unavailable`: Snapshot subscriber unavailable

#### Debug Web UI

```text
GET /debug/ui
```

Returns HTML for an interactive debug dashboard.

**Authorization**: Read

**Availability**: Only when `debug.enabled` is true in runtime configuration

**Response**: HTML content

**Status Codes**
- `200 OK`: UI page returned
- `401 Unauthorized`: Missing or invalid token
- `404 Not Found`: Debug endpoints disabled

---

## Data Types Reference

### DcsTrustResponse

Enumeration of DCS trust levels:

- `full_quorum`: Full member quorum established
- `fail_safe`: Operating in fail-safe mode
- `not_trusted`: DCS not trusted for HA decisions

### HaAuthorityResponse

Tagged union describing the operator-facing primary authority projection:

- `primary`: publishes a `member_id` plus the lease epoch `{ holder, generation }`
- `no_primary`: publishes a structured reason for withholding primary authority
- `unknown`: startup placeholder before a stronger projection is available

### FenceCutoffResponse

Safety boundary payload:

- `epoch`: the lease epoch the cutoff is tied to
- `committed_lsn`: the primary commit point the node must not outlive unsafely

### TargetRoleResponse

Tagged union describing local HA intent:

- `leader`
- `candidate`
- `follower`
- `fail_safe`
- `demoting_for_switchover`
- `fenced`
- `idle`

### ReconcileActionResponse

Tagged union describing the ordered next actions for the HA worker:

- `init_db`
- `base_backup`
- `pg_rewind`
- `start_primary`
- `start_replica`
- `promote`
- `demote`
- `acquire_lease`
- `release_lease`
- `publish`
- `clear_switchover`

---

## Error Responses

All endpoints may return error responses with consistent status codes:

| Status Code | Meaning | When Returned |
|--------------|---------|---------------|
| `400 Bad Request` | Invalid request format | Malformed JSON, invalid fields, bad query parameters |
| `401 Unauthorized` | Missing authentication | No bearer token when tokens are configured |
| `403 Forbidden` | Insufficient privileges | Read token accessing admin endpoint |
| `404 Not Found` | Endpoint not found | Unknown path or debug endpoints disabled |
| `500 Internal Server Error` | Internal failure | Unexpected processing errors |
| `503 Service Unavailable` | Dependency unavailable | DCS store error or snapshot subscriber missing |

Error response bodies contain plain text descriptions of the failure cause.

## TLS Configuration

TLS mode determines connection requirements:

- **Disabled**: Plain HTTP only; TLS handshake attempts fail
- **Optional**: Accepts both plain HTTP and HTTPS on same port
- **Required**: HTTPS only; plain HTTP connections are rejected

When TLS is enabled, client certificates can be required for mutual TLS authentication.
