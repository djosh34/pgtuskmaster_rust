# pgtuskmasterctl

## Synopsis

**pgtuskmasterctl** \[*options*\] *command*

## Description

pgtuskmasterctl is a command line interface for administering PGTuskMaster high availability clusters. It communicates with the PGTuskMaster API server to query cluster state and manage switchover operations.

## Global Options

**--base-url** <uri>
: API server base URI. Default: `http://127.0.0.1:8080`

**--read-token** <token>
: Bearer token for read operations. Fallback: `PGTUSKMASTER_READ_TOKEN` environment variable

**--admin-token** <token>
: Bearer token for administrative operations. Fallback: `PGTUSKMASTER_ADMIN_TOKEN` environment variable

**--timeout-ms** <milliseconds>
: Request timeout in milliseconds. Default: `5000`

**--output** <format>
: Output format. Values: `json`, `text`. Default: `json`

## Commands

### ha state

Query the current high availability state of the cluster.

**API endpoint**: `GET /ha/state`

**Authentication**: Uses `--read-token`, falls back to `--admin-token` if read token absent

**Output fields**:
- `cluster_name` - Cluster identifier from configuration
- `scope` - DCS scope for this cluster
- `self_member_id` - Current node identifier
- `leader` - Current leader member ID, or null
- `switchover_requested_by` - Member ID that requested switchover, or null
- `member_count` - Total members in cluster view
- `dcs_trust` - DCS trust level: `full_quorum`, `fail_safe`, or `not_trusted`
- `ha_phase` - Current HA phase: `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, or `fail_safe`
- `ha_tick` - HA loop iteration counter
- `ha_decision` - Current automation decision with variant-specific fields
- `snapshot_sequence` - Internal snapshot version

### ha switchover clear

Clear an existing switchover request.

**API endpoint**: `DELETE /ha/switchover`

**Authentication**: Requires `--admin-token`

### ha switchover request --requested-by <member_id>

Submit a switchover request.

**API endpoint**: `POST /switchover`

**Authentication**: Requires `--admin-token`

**Request body**: `{"requested_by": "<member_id>"}`

## Output Formats

### json

Renders the full API response as pretty-printed JSON. For `ha state`, includes all fields from the `HaStateResponse` structure. For switchover commands, renders the `AcceptedResponse` with boolean `accepted` field.

### text

Renders human-readable key-value lines.

For `ha state` commands, format is:
```
cluster_name=<value>
scope=<value>
self_member_id=<value>
leader=<value> or <none>
switchover_requested_by=<value> or <none>
member_count=<value>
dcs_trust=<value>
ha_phase=<value>
ha_tick=<value>
ha_decision=<decision_type>(<parameters>)
snapshot_sequence=<value>
```

For switchover commands, format is:
```
accepted=<true|false>
```

## Exit Codes

`0`
: Success

`2`
: Command line usage error (invalid arguments)

`3`
: Transport error (connection refused, timeout)

`4`
: API status error (HTTP 5xx or unexpected status)

`5`
: Response decode error

## Environment Variables

**PGTUSKMASTER_READ_TOKEN**
: Default value for `--read-token`

**PGTUSKMASTER_ADMIN_TOKEN**
: Default value for `--admin-token`

## Examples

Query cluster state with default settings:
```
pgtuskmasterctl ha state
```

Query cluster state with custom API endpoint and text output:
```
pgtuskmasterctl --base-url https://cluster.example:8443 --output text ha state
```

Request switchover:
```
pgtuskmasterctl ha switchover request --requested-by node-b
```

Clear switchover request:
```
pgtuskmasterctl ha switchover clear
```

## See Also

pgtuskmaster(1)
