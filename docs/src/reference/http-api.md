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
{}
```

The request body is an empty object. Unknown fields are rejected.

**Response** (`AcceptedResponse`)
```text
{
  "accepted": true
}
```

**Status Codes**
- `202 Accepted`: Switchover request accepted and written to DCS
- `400 Bad Request`: Invalid JSON or unknown request fields
- `401 Unauthorized`: Missing or invalid token
- `403 Forbidden`: Read token used for admin endpoint
- `503 Service Unavailable`: DCS store error

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

**Response** (`HaStateResponse`)
```text
{
  "cluster_name": "<string>",
  "scope": "<string>",
  "self_member_id": "<string>",
  "leader": "<string>" | null,
  "switchover_pending": <bool>,
  "member_count": <number>,
  "dcs_trust": "<trust_variant>",
  "ha_phase": "<phase_variant>",
  "ha_tick": <number>,
  "ha_decision": <decision_variant>,
  "snapshot_sequence": <number>
}
```

**Field Details**
- `cluster_name`: Configured cluster name
- `scope`: DCS scope for this cluster
- `self_member_id`: Local member identifier
- `leader`: Current leader member ID if one exists
- `switchover_pending`: Whether a switchover request is currently pending
- `member_count`: Number of members in DCS cache
- `dcs_trust`: Trust level of DCS (see DcsTrustResponse variants)
- `ha_phase`: Current HA phase (see HaPhaseResponse variants)
- `ha_tick`: HA decision loop counter
- `ha_decision`: Current HA decision (see HaDecisionResponse variants)
- `snapshot_sequence`: Monotonic snapshot version

Successor choice remains automatic. `POST /switchover` signals intent to switch over, and the HA engine picks the replacement primary from observed cluster state.

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

### HaPhaseResponse

Enumeration of HA worker phases:

- `init`: Initial startup phase
- `waiting_postgres_reachable`: Waiting for PostgreSQL connectivity
- `waiting_dcs_trusted`: Waiting for DCS to become trusted
- `waiting_switchover_successor`: Waiting for switchover successor selection
- `replica`: Operating as replica
- `candidate_leader`: Candidate for leader election
- `primary`: Operating as primary
- `rewinding`: Rewinding state to align with leader
- `bootstrapping`: Initial cluster bootstrap
- `fencing`: Fencing previous primary
- `fail_safe`: Operating in fail-safe mode

### HaDecisionResponse

Tagged union of HA decisions:

- `no_change`: No state change required
- `wait_for_postgres`: Waiting for PostgreSQL with start/leaders fields
- `wait_for_dcs_trust`: Waiting for DCS trust restoration
- `attempt_leadership`: Attempting to become leader
- `follow_leader`: Following specified leader
- `become_primary`: Promoting to primary role
- `step_down`: Stepping down with reason and flags
- `recover_replica`: Recovering replica with strategy
- `fence_node`: Fencing node
- `release_leader_lease`: Releasing leader lease with reason
- `enter_fail_safe`: Entering fail-safe mode

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
