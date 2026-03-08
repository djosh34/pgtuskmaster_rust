```
# pgtuskmasterctl

## Name

pgtuskmasterctl — HA admin CLI for PGTuskMaster API

## Synopsis

**pgtuskmasterctl** \[*options*\] **ha** *subcommand*

## Description

pgtuskmasterctl is a command-line client for the PGTuskMaster high-availability management API. It provides operators with read and administrative access to cluster state and switchover controls.

## Global Options

**--base-url**=*url*
: API server base URL. Default: `http://127.0.0.1:8080`

**--read-token**=*token*
: Bearer token for read operations. Falls back to `PGTUSKMASTER_READ_TOKEN` environment variable.

**--admin-token**=*token*
: Bearer token for administrative operations. Falls back to `PGTUSKMASTER_ADMIN_TOKEN` environment variable.

**--timeout-ms**=*milliseconds*
: HTTP request timeout. Default: `5000`

**--output**=*format*
: Output format. Values: `json`, `text`. Default: `json`

## Commands

### ha state

Retrieve current high-availability state from the API.

**Syntax:**
pgtuskmasterctl ha state

**HTTP method:** GET /ha/state  
**Auth role:** read (or admin if read token absent)

**Output fields (json):**
- `cluster_name`
- `scope`
- `self_member_id`
- `leader`
- `switchover_requested_by`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_tick`
- `ha_decision`
- `snapshot_sequence`

**Output fields (text):**
```
cluster_name=...
scope=...
self_member_id=...
leader=... or <none>
switchover_requested_by=... or <none>
member_count=...
dcs_trust=...
ha_phase=...
ha_tick=...
ha_decision=...
snapshot_sequence=...
```

**dcs_trust values:** `full_quorum`, `fail_safe`, `not_trusted`

**ha_phase values:** `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe`

**ha_decision variants:**
- `no_change`
- `wait_for_postgres(start_requested=..., leader_member_id=...)`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader(leader_member_id=...)`
- `become_primary(promote=...)`
- `step_down(reason=..., release_leader_lease=..., clear_switchover=..., fence=...)`
- `recover_replica(strategy=...)`
- `fence_node`
- `release_leader_lease(reason=...)`
- `enter_fail_safe(release_leader_lease=...)`

### ha switchover clear

Clear any pending switchover request.

**Syntax:**
pgtuskmasterctl ha switchover clear

**HTTP method:** DELETE /ha/switchover  
**Auth role:** admin

**Output:**
- JSON: `{"accepted": true|false}`
- Text: `accepted=true` or `accepted=false`

### ha switchover request

Request a switchover to a new primary.

**Syntax:**
pgtuskmasterctl ha switchover request **--requested-by**=*id*

**HTTP method:** POST /switchover (note: path is `/switchover`, not `/ha/switchover`)  
**Auth role:** admin

**Options:**
**--requested-by**=*member_id* (required)

Request body: `{"requested_by": "member_id"}`

**Output:**
- JSON: `{"accepted": true|false}`
- Text: `accepted=true` or `accepted=false`

## Exit Codes

**0**: Success  
**2**: CLI usage error (invalid arguments or subcommand)  
**3**: Transport error (connection refused, timeout)  
**4**: Unexpected API status response  
**5**: Response decode failure

## Examples

View cluster state:
```
pgtuskmasterctl --base-url http://cluster.example:8080 ha state
```

Request switchover:
```
pgtuskmasterctl --admin-token $ADMIN_TOKEN ha switchover request --requested-by node-b
```

Clear switchover:
```
pgtuskmasterctl --admin-token $ADMIN_TOKEN ha switchover clear
```

Use text output:
```
pgtuskmasterctl --output text ha state
```
```
