# Debug Cluster Issues

This guide shows how to investigate incidents with `pgtm` first and raw `/state` only when you need protocol-level confirmation.

## Goal

Answer four questions quickly:

1. what does the cluster look like right now?
2. does the seed node trust its DCS view?
3. what authority and local role is the seed node publishing?
4. what node and member facts in `/state` explain that answer?

## Step 1: Start with `pgtm status -v`

```bash
pgtm -c config.toml status -v
```

Focus on:

- warnings
- `ROLE`
- `TRUST`
- `LEADER`
- `DECISION`
- `PROCESS`

That gives you the operator summary derived from a single seed `/state` document.

## Step 2: Interpret trust first

The seed node reports one of:

- `full_quorum`
- `degraded`
- `not_trusted`

If trust is degraded, restore DCS health or member freshness before expecting normal promotions or switchovers.

## Step 3: Inspect the raw seed document when needed

```bash
curl --fail --silent http://node-a:8080/state | jq .
```

The important sections are:

- `pg`
- `process`
- `dcs.trust`
- `dcs.leader`
- `dcs.members`
- `dcs.switchover`
- `ha.publication`
- `ha.role`
- `ha.world`
- `ha.planned_actions`

## Step 4: Use the worldview to explain the answer

When the summary is not enough, check:

- `ha.world.local`
- `ha.world.global`

Those sections expose the HA engine's current derived inputs, including lease state, switchover state, and peer eligibility.

## Common patterns

### No authoritative primary

Look at:

- `dcs.trust`
- `ha.publication.authority`
- `ha.world.global.lease`

### Switchover seems stuck

Look at:

- `dcs.switchover`
- `ha.role`
- `ha.planned_actions.coordination`
- replica readiness in `dcs.members`

### Local process work is stalled

Look at:

- `process`
- `ha.planned_actions.process`
- `ha.world.local.process`

## Next step

Once you know whether the blocker is trust, DCS membership, PostgreSQL health, or local process execution, move to the matching operational procedure instead of sampling more endpoints.
