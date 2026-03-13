# Monitor via CLI Signals

This guide shows how to monitor a running cluster with `pgtm` and the raw `/state` endpoint.

## Goal

Track:

- primary authority changes
- trust degradation
- switchover intent
- replica readiness
- local process activity

## Step 1: Poll `pgtm status`

```bash
pgtm -c config.toml status
```

For structured output:

```bash
pgtm -c config.toml --json status > status.json
```

The status view exposes:

- `health`
- `warnings`
- `switchover`
- `nodes`

## Step 2: Watch continuously

```bash
pgtm -c config.toml status --watch
```

Use `-v --watch` during an incident when you want the seed node's local phase, decision, and process fields on every refresh.

## Step 3: Alert on high-signal conditions

Useful alerts include:

- `health=degraded`
- warnings containing `degraded_trust`
- warnings containing `no_primary`
- replicas that are not `ready`
- a pending switchover that does not clear

## Step 4: Read `/state` directly for deeper polling

```bash
curl --fail --silent http://node-a:8080/state | jq .
```

Useful fields for machine polling:

- `dcs.trust`
- `dcs.cache.leader_lease`
- `dcs.cache.switchover_intent`
- `dcs.cache.member_slots`
- `ha.publication.authority`
- `ha.role`
- `ha.planned_commands`
- `process`

## Step 5: Archive the current node state during incidents

```bash
stamp=$(date -u +%Y%m%dT%H%M%SZ)
curl --fail --silent http://node-a:8080/state > "/var/log/pgtuskmaster/state-${stamp}.json"
```

That file is the complete current observation surface for the node. There is no separate debug-history payload to capture anymore.
