# pgtuskmasterctl - HA admin CLI for PGTuskMaster API

## Synopsis

`pgtuskmasterctl [GLOBAL OPTIONS] <COMMAND>`

## Description

pgtuskmasterctl provides administrative access to the PGTuskMaster API for high availability operations. The tool sends HTTP requests to a running PGTuskMaster node and renders responses.

## Global Options

* `--base-url <URL>`  
  API base URL. Default: `http://127.0.0.1:8080`

* `--read-token <TOKEN>`  
  Bearer token for read operations. Environment: `PGTUSKMASTER_READ_TOKEN`

* `--admin-token <TOKEN>`  
  Bearer token for admin operations. Environment: `PGTUSKMASTER_ADMIN_TOKEN`

* `--timeout-ms <MILLISECONDS>`  
  Request timeout. Default: `5000`

* `--output <FORMAT>`  
  Response format: `json` or `text`. Default: `json`

## Commands

### `ha`

High availability control command group.

#### `ha state`

Retrieve current HA state.

**Endpoint:** `GET /ha/state`  
**Auth:** Read token (falls back to admin token)

**Output fields:**
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

#### `ha switchover`

Switchover control command group.

##### `ha switchover clear`

Clear pending switchover request.

**Endpoint:** `DELETE /ha/switchover`  
**Auth:** Admin token

##### `ha switchover request --requested-by <MEMBER_ID>`

Request a switchover.

**Endpoint:** `POST /switchover`  
**Auth:** Admin token

**Required flag:**
- `--requested-by <MEMBER_ID>` Member ID requesting switchover

## Authentication

Read operations use the read token first, falling back to the admin token if no read token is provided. Admin operations require the admin token. Empty or whitespace-only token values are treated as absent.

## Output Formats

**json:** Serializes responses as formatted JSON.

**text:** Renders responses as line-delimited key-value pairs:
- State: `key=value`
- Accepted: `accepted=<bool>`

## Exit Codes

* `0` Success
* `2` Clap usage error (invalid command or arguments)
* `3` Transport error (network failure, connection refused)
* `4` API status error (unexpected HTTP status)
* `5` Decode error (malformed response)

## Command Hierarchy

```
pgtuskmasterctl
├── --base-url
├── --read-token
├── --admin-token
├── --timeout-ms
├── --output
└── ha
    ├── state
    └── switchover
        ├── clear
        └── request --requested-by
```
